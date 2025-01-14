use crate::tsf::factory::TextServiceFactory;

use windows::{
    core::{Interface, Result},
    Win32::UI::TextServices::{ITfLangBarItemButton, ITfLangBarItemMgr},
};

use super::{client_action::ClientAction, composition::CompositionState};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum InputMode {
    #[default]
    Latin,
    Kana,
}

impl TextServiceFactory {
    pub fn set_input_mode(&self, mode: InputMode) -> Result<()> {
        log::debug!("set input mode: {:?}", mode);
        {
            let mut text_service = self.borrow_mut()?;

            if text_service.mode == mode {
                return Ok(());
            }

            text_service.mode = mode;

            // change the icon of the language bar item
            let thread_mgr = text_service.thread_mgr()?;

            unsafe {
                thread_mgr
                    .cast::<ITfLangBarItemMgr>()?
                    .RemoveItem(&text_service.this::<ITfLangBarItemButton>()?)?;

                thread_mgr
                    .cast::<ITfLangBarItemMgr>()?
                    .AddItem(&text_service.this::<ITfLangBarItemButton>()?)?;
            };
        }

        // stop the composition
        let actions = vec![ClientAction::EndComposition];

        self.handle_action(&actions, CompositionState::None)?;

        Ok(())
    }
}
