use crate::tsf::factory::TextServiceFactory;

use windows::{
    core::Interface,
    Win32::UI::TextServices::{ITfLangBarItemButton, ITfLangBarItemMgr},
};

use anyhow::Result;

use super::{client_action::ClientAction, composition::CompositionState, state::IMEState};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum InputMode {
    #[default]
    Latin,
    Kana,
}

impl TextServiceFactory {
    pub fn set_input_mode(&self, mode: InputMode) -> Result<()> {
        {
            let mut ime_state = IMEState::get()?;
            ime_state.input_mode = mode;

            // update the language bar
            self.update_lang_bar()?;
        }

        // stop the composition
        let actions = vec![ClientAction::EndComposition];

        self.handle_action(&actions, CompositionState::None)?;

        Ok(())
    }

    pub fn update_lang_bar(&self) -> Result<()> {
        // change the icon of the language bar item
        let text_service = self.borrow()?;
        let thread_mgr = text_service.thread_mgr()?;

        unsafe {
            thread_mgr
                .cast::<ITfLangBarItemMgr>()?
                .RemoveItem(&text_service.this::<ITfLangBarItemButton>()?)?;

            thread_mgr
                .cast::<ITfLangBarItemMgr>()?
                .AddItem(&text_service.this::<ITfLangBarItemButton>()?)?;
        };

        Ok(())
    }
}
