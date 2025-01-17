use windows::{
    core::implement,
    Win32::{
        Foundation::{E_FAIL, RECT},
        UI::TextServices::{
            ITfCompositionSink, ITfContext, ITfContextComposition, ITfEditSession,
            ITfEditSession_Impl, ITfInsertAtSelection, GUID_PROP_ATTRIBUTE, TF_AE_NONE,
            TF_ANCHOR_END, TF_ANCHOR_START, TF_ES_READWRITE, TF_IAS_QUERYONLY, TF_SELECTION,
            TF_SELECTIONSTYLE, TF_ST_CORRECTION,
        },
    },
};
use windows_core::VARIANT;

use std::{cell::RefCell, mem::ManuallyDrop, rc::Rc};

use anyhow::Result;

use crate::{engine::state::IMEState, extension::StringExt as _, globals::GUID_DISPLAY_ATTRIBUTE};

use super::factory::TextServiceFactory;

#[implement(ITfEditSession)]
struct EditSession {
    callback: Rc<dyn Fn(u32) -> anyhow::Result<()>>,
}

// edit action will be performed within this function
pub fn edit_session(
    client_id: u32,
    context: ITfContext,
    callback: Rc<dyn Fn(u32) -> anyhow::Result<()>>,
) -> Result<()> {
    let session: ITfEditSession = EditSession::new(callback).into();

    let result = unsafe { context.RequestEditSession(client_id, &session, TF_ES_READWRITE) };

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::Error::new(e)),
    }
}

impl EditSession {
    pub fn new(callback: Rc<dyn Fn(u32) -> anyhow::Result<()>>) -> Self {
        EditSession { callback }
    }
}

impl ITfEditSession_Impl for EditSession_Impl {
    fn DoEditSession(&self, cookie: u32) -> windows::core::Result<()> {
        (self.callback)(cookie).map_err(|e| windows::core::Error::new(E_FAIL, e.to_string()))?;
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
                text_service.tid,
                text_service.context()?,
                Rc::new({
                    // unpadded is all you need!
                    let text = text.to_wide_16_unpadded();
                    let context = text_service.context::<ITfContext>()?;
                    let display_attribute_atom = text_service.display_attribute_atom.clone();
                    move |cookie| unsafe {
                        let range = composition.GetRange()?;

                        range.SetText(cookie, TF_ST_CORRECTION, &text)?;

                        let display_attribute = display_attribute_atom.get(&GUID_DISPLAY_ATTRIBUTE);
                        if let Some(display_attribute) = display_attribute {
                            let pvar = VARIANT::from(*display_attribute as i32);
                            let prop = context.GetProperty(&GUID_PROP_ATTRIBUTE)?;
                            prop.SetValue(cookie, &range, &pvar)?;
                        }

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

    pub fn set_cursor(&self, position: i32) -> Result<()> {
        let text_service = self.borrow()?;

        if let Some(composition) = text_service.borrow_composition()?.tip_composition.clone() {
            edit_session(
                text_service.tid,
                text_service.context()?,
                Rc::new({
                    let context = text_service.context::<ITfContext>()?;
                    move |cookie| unsafe {
                        let range = composition.GetRange()?;
                        range.Collapse(cookie, TF_ANCHOR_START)?;
                        range.ShiftEnd(cookie, position, std::ptr::null_mut(), std::ptr::null())?;
                        range.ShiftStart(
                            cookie,
                            position,
                            std::ptr::null_mut(),
                            std::ptr::null(),
                        )?;

                        let selection = TF_SELECTION {
                            range: ManuallyDrop::new(Some(range)),
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

    pub fn get_and_send_pos(&self) -> Result<()> {
        let text_service = self.borrow()?;
        let composition = text_service.borrow_composition()?;

        // I don't want to send, but edit_session is asynchronous, so I have to send to avoid blocking the main thread.
        if let Some(tip_composition) = composition.tip_composition.clone() {
            edit_session(
                text_service.tid,
                text_service.context()?,
                Rc::new({
                    let context = text_service.context::<ITfContext>()?;
                    let ipc_service = IMEState::get()?.ipc_service.clone();

                    move |cookie| unsafe {
                        let view = context.GetActiveView()?;
                        let range = tip_composition.GetRange()?;
                        let mut ipc_service = ipc_service.clone();

                        let mut rect = RECT::default();
                        let mut clipped = false.into();
                        view.GetTextExt(cookie, &range, &mut rect, &mut clipped)?;

                        ipc_service.set_window_position(rect.left, rect.bottom)?;
                        Ok(())
                    }
                }),
            )?
        } else {
            return Ok(());
        }

        Ok(())
    }
}
