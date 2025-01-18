use windows::Win32::UI::TextServices::{ITfContext, ITfDocumentMgr, ITfThreadMgrEventSink_Impl};

use anyhow::Result;

use crate::engine::{client_action::ClientAction, composition::CompositionState};

use super::factory::TextServiceFactory_Impl;

impl ITfThreadMgrEventSink_Impl for TextServiceFactory_Impl {
    #[macros::anyhow]
    fn OnInitDocumentMgr(&self, _pdim: Option<&ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    #[macros::anyhow]
    fn OnUninitDocumentMgr(&self, _pdim: Option<&ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    #[macros::anyhow]
    fn OnSetFocus(
        &self,
        focus: Option<&ITfDocumentMgr>,
        _prevfocus: Option<&ITfDocumentMgr>,
    ) -> Result<()> {
        // if focus is changed, the composition will be terminated
        self.update_lang_bar()?;

        // if focus is changed, the text layout sink should be updated
        if let Some(focus) = focus {
            self.borrow_mut()?.advise_text_layout_sink(focus.clone())?;
        }

        let actions = vec![ClientAction::EndComposition];
        self.handle_action(&actions, CompositionState::None)?;

        Ok(())
    }

    #[macros::anyhow]
    fn OnPushContext(&self, _pic: Option<&ITfContext>) -> Result<()> {
        Ok(())
    }

    #[macros::anyhow]
    fn OnPopContext(&self, _pic: Option<&ITfContext>) -> Result<()> {
        Ok(())
    }
}
