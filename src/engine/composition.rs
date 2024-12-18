use crate::{
    engine::user_action::UserAction,
    extension::StringExt,
    tsf::{
        edit_session::edit_session,
        factory::{TextServiceFactory, TextServiceFactory_Impl},
    },
};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum CompositionState {
    #[default]
    None,
    Composing,
    Previewing,
    Selecting,
}

#[derive(Default, Clone, Debug)]
pub struct Composition {
    pub spelling: String,
    pub suggestions: Vec<String>,
    pub state: CompositionState,
    pub tip_composition: Option<ITfComposition>,
}

use std::{cell::RefCell, mem::ManuallyDrop, rc::Rc};

use windows::{
    core::Result,
    Win32::{
        Foundation::WPARAM,
        UI::TextServices::{
            ITfComposition, ITfCompositionSink, ITfCompositionSink_Impl, ITfContext,
            ITfContextComposition, ITfInsertAtSelection, TF_AE_NONE, TF_ANCHOR_END,
            TF_IAS_QUERYONLY, TF_SELECTION, TF_SELECTIONSTYLE,
        },
    },
};

impl ITfCompositionSink_Impl for TextServiceFactory_Impl {
    fn OnCompositionTerminated(
        &self,
        _ecwrite: u32,
        _pcomposition: Option<&windows::Win32::UI::TextServices::ITfComposition>,
    ) -> windows_core::Result<()> {
        Ok(())
    }
}

impl TextServiceFactory {
    pub fn start_composition(&self) -> Result<()> {
        log::debug!("start_composition");
        let composition = Rc::new(RefCell::new(None));

        {
            let text_service = self.borrow()?;
            let context = text_service.context()?;
            let context_composition = text_service.context::<ITfContextComposition>()?;
            let sink = text_service.this::<ITfCompositionSink>()?;
            let insert = text_service.context::<ITfInsertAtSelection>()?;

            edit_session(
                text_service.tid,
                context,
                Rc::new({
                    let composition_ref = Rc::clone(&composition);
                    move |cookie| unsafe {
                        let range = insert.InsertTextAtSelection(cookie, TF_IAS_QUERYONLY, &[])?;
                        let composition =
                            context_composition.StartComposition(cookie, &range, &sink)?;

                        *composition_ref.borrow_mut() = Some(composition);
                        Ok(())
                    }
                }),
            )?;
        }

        self.borrow_mut()?.borrow_mut_composition()?.tip_composition = composition.borrow().clone();
        log::debug!("Composition started {composition:?}");

        Ok(())
    }

    pub fn end_composition(&self) -> Result<()> {
        log::debug!("end_composition");
        let text_service = self.borrow()?;

        if let Some(composition) = text_service.borrow_composition()?.tip_composition.clone() {
            edit_session(
                text_service.tid,
                text_service.context()?,
                Rc::new({
                    move |cookie| unsafe {
                        composition.EndComposition(cookie)?;
                        Ok(())
                    }
                }),
            )?
        } else {
            log::warn!("Composition is not started");
        }

        Ok(())
    }

    pub fn set_text(&self, text: &str) -> Result<()> {
        let text_service = self.borrow()?;

        if let Some(composition) = text_service.borrow_composition()?.tip_composition.clone() {
            edit_session(
                self.borrow()?.tid,
                self.borrow()?.context()?,
                Rc::new({
                    // unpadded is all you need!
                    let text = text.to_wide_16_unpadded();
                    let context = self.borrow()?.context::<ITfContext>()?;
                    move |cookie| unsafe {
                        let range = composition.GetRange()?;

                        range.SetText(cookie, 0, &text)?;
                        range.Collapse(cookie, TF_ANCHOR_END)?;
                        let selection = TF_SELECTION {
                            range: ManuallyDrop::new(Some(range.clone())),
                            style: TF_SELECTIONSTYLE {
                                ase: TF_AE_NONE,
                                fInterimChar: false.into(),
                            },
                        };

                        context.SetSelection(cookie, &[selection])?;

                        Ok(())
                    }
                }),
            )?
        } else {
            log::warn!("Composition is not started");
        }

        Ok(())
    }

    pub fn handle_key(&self, context: Option<&ITfContext>, wparam: WPARAM) -> Result<bool> {
        {
            if let Some(context) = context {
                self.borrow_mut()?.context = Some(context.clone());
            } else {
                return Ok(false);
            };
        }

        #[allow(clippy::let_and_return)]
        let mut composition = {
            let text_service = self.borrow()?;
            let composition = text_service.borrow_composition()?.clone();
            composition
        };
        let action = UserAction::from(wparam.0);

        let transition = match composition.state {
            CompositionState::None => match action {
                UserAction::Input(_char) => {
                    self.start_composition()?;
                    composition.spelling.push('a');

                    CompositionState::Composing
                }
                UserAction::Number(number) => {
                    self.start_composition()?;
                    composition.spelling.push_str(&number.to_string());

                    CompositionState::Composing
                }
                _ => {
                    return Ok(false);
                }
            },
            CompositionState::Composing => match action {
                UserAction::Input(char) => {
                    composition.spelling.push(char);

                    CompositionState::Composing
                }
                UserAction::Number(number) => {
                    composition.spelling.push_str(&number.to_string());

                    CompositionState::Composing
                }
                UserAction::Backspace => {
                    composition.spelling.pop();

                    if composition.spelling.is_empty() {
                        self.end_composition()?;
                        CompositionState::None
                    } else {
                        CompositionState::Composing
                    }
                }
                UserAction::Enter => {
                    self.end_composition()?;
                    CompositionState::None
                }
                UserAction::Escape => {
                    self.end_composition()?;
                    CompositionState::None
                }
                _ => {
                    return Ok(false);
                }
            },
            _ => {
                return Ok(false);
            }
        };

        let spell = {
            if transition == CompositionState::None {
                "".to_string()
            } else {
                composition.spelling.clone()
            }
        };

        if !spell.is_empty() {
            self.set_text(&spell)?;
        };

        let text_service = self.borrow()?;
        let mut composition = text_service.borrow_mut_composition()?;
        composition.spelling = spell.clone();
        composition.state = transition;

        Ok(true)
    }
}
