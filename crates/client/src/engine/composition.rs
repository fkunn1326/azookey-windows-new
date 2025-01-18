use std::cmp::{max, min};

use crate::{
    engine::user_action::UserAction,
    extension::VKeyExt as _,
    tsf::factory::{TextServiceFactory, TextServiceFactory_Impl},
};

use super::{
    client_action::{ClientAction, SetSelectionType},
    full_width::to_fullwidth,
    input_mode::InputMode,
    ipc_service::Candidates,
    state::IMEState,
    user_action::Navigation,
};
use windows::Win32::{
    Foundation::WPARAM,
    UI::{
        Input::KeyboardAndMouse::VK_CONTROL,
        TextServices::{ITfComposition, ITfCompositionSink_Impl, ITfContext},
    },
};

use anyhow::Result;

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
    pub suggestion: String,
    pub suggestions: Vec<String>,
    pub selection_index: i32,
    pub candidates: Candidates,
    pub state: CompositionState,
    pub tip_composition: Option<ITfComposition>,
}

impl ITfCompositionSink_Impl for TextServiceFactory_Impl {
    #[macros::anyhow]
    fn OnCompositionTerminated(
        &self,
        _ecwrite: u32,
        _pcomposition: Option<&ITfComposition>,
    ) -> Result<()> {
        // if user clicked outside the composition, the composition will be terminated

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
        let (composition, mode) = {
            let text_service = self.borrow()?;
            let composition = text_service.borrow_composition()?.clone();
            let mode = IMEState::get()?.input_mode.clone();
            (composition, mode)
        };

        let action = UserAction::try_from(wparam.0)?;

        let (transition, actions) = match composition.state {
            CompositionState::None => match action {
                UserAction::Input(char) if mode == InputMode::Kana => (
                    CompositionState::Composing,
                    vec![
                        ClientAction::StartComposition,
                        ClientAction::AppendText(char.to_string()),
                    ],
                ),
                UserAction::Number(number) if mode == InputMode::Kana => (
                    CompositionState::Composing,
                    vec![
                        ClientAction::StartComposition,
                        ClientAction::AppendText(number.to_string()),
                    ],
                ),
                UserAction::ToggleInputMode => (
                    CompositionState::None,
                    vec![match mode {
                        InputMode::Kana => ClientAction::SetIMEMode(InputMode::Latin),
                        InputMode::Latin => ClientAction::SetIMEMode(InputMode::Kana),
                    }],
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
                    if composition.suggestion.len() == 1 {
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
                    Navigation::Up => (
                        CompositionState::Composing,
                        vec![ClientAction::SetSelection(SetSelectionType::Up)],
                    ),
                    Navigation::Down => (
                        CompositionState::Composing,
                        vec![ClientAction::SetSelection(SetSelectionType::Down)],
                    ),
                },
                UserAction::ToggleInputMode => (
                    CompositionState::None,
                    vec![ClientAction::SetIMEMode(InputMode::Latin)],
                ),
                UserAction::Space | UserAction::Tab => (
                    CompositionState::Composing,
                    vec![ClientAction::SetSelection(SetSelectionType::Down)],
                ),
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

    pub fn handle_action(
        &self,
        actions: &[ClientAction],
        transition: CompositionState,
    ) -> Result<()> {
        #[allow(clippy::let_and_return)]
        let (composition, mode) = {
            let text_service = self.borrow()?;
            let composition = text_service.borrow_composition()?.clone();
            let mode = IMEState::get()?.input_mode.clone();
            (composition, mode)
        };

        let mut suggestion = composition.suggestion.clone();
        let mut candidates = composition.candidates.clone();
        let mut selection_index = composition.selection_index;
        let mut ipc_service = IMEState::get()?.ipc_service.clone();
        let mut transition = transition;

        for action in actions {
            match action {
                ClientAction::StartComposition => {
                    self.start_composition()?;
                    ipc_service.show_window()?;
                }
                ClientAction::EndComposition => {
                    self.set_text(&suggestion, "")?;
                    self.end_composition()?;
                    selection_index = 0;
                    suggestion.clear();
                    ipc_service.hide_window()?;
                    ipc_service.clear_text()?;
                }
                ClientAction::AppendText(text) => {
                    let text = match mode {
                        InputMode::Kana => to_fullwidth(text),
                        InputMode::Latin => text.to_string(),
                    };

                    candidates = ipc_service.append_text(text.clone())?;
                    let text = candidates.texts[selection_index as usize].clone();
                    let sub_text = candidates.sub_texts[selection_index as usize].clone();

                    suggestion = format!("{}{}", text, sub_text);

                    self.set_text(&text, &sub_text)?;
                    ipc_service.set_candidates(candidates.texts.clone())?;
                    ipc_service.set_selection(selection_index as i32)?;
                }
                ClientAction::RemoveText => {
                    candidates = ipc_service.remove_text()?;
                    let empty = "".to_string();
                    let text = candidates
                        .texts
                        .get(selection_index as usize)
                        .cloned()
                        .unwrap_or(empty.clone());
                    let sub_text = candidates
                        .sub_texts
                        .get(selection_index as usize)
                        .cloned()
                        .unwrap_or(empty);

                    suggestion = format!("{}{}", text, sub_text);

                    self.set_text(&text, &sub_text)?;
                    ipc_service.set_candidates(candidates.texts.clone())?;
                    ipc_service.set_selection(selection_index as i32)?;

                    if suggestion.is_empty() {
                        transition = CompositionState::None;
                        ipc_service.hide_window()?;
                    }
                }
                ClientAction::MoveCursor(_offset) => {
                    // TODO: I'll use azookey-kkc's composingText
                    // self.set_cursor(offset)?;
                }
                ClientAction::SetIMEMode(mode) => {
                    self.set_input_mode(mode.clone())?;
                    suggestion.clear();
                    ipc_service.clear_text()?;
                }
                ClientAction::SetSelection(selection) => {
                    let candidates = {
                        let text_service = self.borrow()?;
                        let composition = text_service.borrow_composition()?.clone();
                        let candidates = composition.candidates.clone();
                        candidates
                    };

                    let texts = candidates.texts.clone();
                    let sub_texts = candidates.sub_texts.clone();

                    selection_index = match selection {
                        SetSelectionType::Up => max(0, selection_index - 1),
                        SetSelectionType::Down => min(texts.len() as i32 - 1, selection_index + 1),
                        SetSelectionType::Number(number) => *number,
                    };

                    ipc_service.set_selection(selection_index as i32)?;
                    let text = texts[selection_index as usize].clone();
                    let sub_text = sub_texts[selection_index as usize].clone();

                    suggestion = format!("{}{}", text, sub_text);

                    self.set_text(&text, &sub_text)?;
                }
            }
        }

        let text_service = self.borrow()?;
        let mut composition = text_service.borrow_mut_composition()?;
        composition.suggestion = suggestion.clone();
        composition.state = transition;
        composition.selection_index = selection_index;
        composition.candidates = candidates;

        Ok(())
    }
}
