use crate::{
    engine::user_action::UserAction, extension::VKeyExt as _, tsf::factory::{TextServiceFactory, TextServiceFactory_Impl}
};

use windows::{
    core::Result,
    Win32::{
        Foundation::WPARAM,
        UI::{Input::KeyboardAndMouse::VK_CONTROL, TextServices::{ITfComposition, ITfCompositionSink_Impl, ITfContext}}
    }
};

use super::{client_action::ClientAction, user_action::Navigation};

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

impl ITfCompositionSink_Impl for TextServiceFactory_Impl {
    fn OnCompositionTerminated(
        &self,
        _ecwrite: u32,
        _pcomposition: Option<&ITfComposition>,
    ) -> Result<()> {
        // if user clicked outside the composition, the composition will be terminated
        log::debug!("composition terminated");

        let actions = vec![ClientAction::EndComposition];
        self.handle_action(&actions, CompositionState::None)?;

        Ok(())
    }
}

impl TextServiceFactory {
    pub fn handle_key(&self, context: Option<&ITfContext>, wparam: WPARAM) -> Result<bool> {
        if let Some(context) = context {
            self.borrow_mut()?.context = Some(context.clone());
        } else {
            return Ok(false);
        };

        // check shortcut keys
        if VK_CONTROL.is_pressed() {
            return Ok(false);
        }

        #[allow(clippy::let_and_return)]
        let composition = {
            let text_service = self.borrow()?;
            let composition = text_service.borrow_composition()?.clone();
            composition
        };
        let action = UserAction::from(wparam.0);

        let (transition, actions) = match composition.state {
            CompositionState::None => match action {
                UserAction::Input(char) => (
                    CompositionState::Composing,
                    vec![
                        ClientAction::StartComposition,
                        ClientAction::AppendText(char.to_string()),
                    ],
                ),
                UserAction::Number(number) => (
                    CompositionState::Composing,
                    vec![
                        ClientAction::StartComposition,
                        ClientAction::AppendText(number.to_string()),
                    ],
                ),
                _ => {
                    return Ok(false);
                }
            },
            CompositionState::Composing => match action {
                UserAction::Input(char) => (
                    CompositionState::Composing,
                    vec![ClientAction::AppendText(char.to_string())],
                ),
                UserAction::Number(number) => (
                    CompositionState::Composing,
                    vec![ClientAction::AppendText(number.to_string())],
                ),
                UserAction::Backspace => {
                    if composition.spelling.len() == 1 {
                        (
                            CompositionState::None,
                            vec![ClientAction::RemoveText, ClientAction::EndComposition],
                        )
                    } else {
                        (CompositionState::Composing, vec![ClientAction::RemoveText])
                    }
                }
                UserAction::Enter => (CompositionState::None, vec![ClientAction::EndComposition]),
                UserAction::Escape => (
                    CompositionState::None,
                    vec![ClientAction::RemoveText, ClientAction::EndComposition],
                ),
                UserAction::Navigation(direction) => match direction {
                    Navigation::Right => (
                        CompositionState::Composing,
                        vec![ClientAction::MoveCursor(1)],
                    ),
                    Navigation::Left => (
                        CompositionState::Composing,
                        vec![ClientAction::MoveCursor(-1)],
                    ),
                    _ => (CompositionState::Composing, vec![]),
                },
                _ => {
                    return Ok(false);
                }
            },
            _ => {
                return Ok(false);
            }
        };

        self.handle_action(&actions, transition)?;

        Ok(true)
    }

    pub fn handle_action(&self, actions: &[ClientAction], transition: CompositionState) -> Result<()> {
        #[allow(clippy::let_and_return)]
        let composition = {
            let text_service = self.borrow()?;
            let composition = text_service.borrow_composition()?.clone();
            composition
        };

        let mut spell = composition.spelling.clone();

        for action in actions {
            match action {
                ClientAction::StartComposition => {
                    self.start_composition()?;
                }
                ClientAction::EndComposition => {
                    self.end_composition()?;
                    spell.clear();
                }
                ClientAction::AppendText(text) => {
                    spell.push_str(text);
                    self.set_text(&spell)?;
                }
                ClientAction::RemoveText => {
                    spell.pop();
                    self.set_text(&spell)?;
                }
                ClientAction::MoveCursor(_offset) => {
                    // TODO: I'll use azookey-kkc's composingText
                    // self.set_cursor(offset)?;
                }
            }
        }

        let text_service = self.borrow()?;
        let mut composition = text_service.borrow_mut_composition()?;
        composition.spelling = spell.clone();
        composition.state = transition;

        Ok(())
    }
}
