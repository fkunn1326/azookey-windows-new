use windows::{
    core::Interface as _,
    Win32::UI::TextServices::{
        ITfContext, ITfContextView, ITfDocumentMgr, ITfSource, ITfTextLayoutSink,
        ITfTextLayoutSink_Impl, TfLayoutCode,
    },
};

use anyhow::Result;

use crate::engine::state::IMEState;

use super::{factory::TextServiceFactory_Impl, text_service::TextService};

impl ITfTextLayoutSink_Impl for TextServiceFactory_Impl {
    // This function is called when the text display position changes when the IME is enabled.
    // However, this function **will not be called** in Microsoft Store applications such as Notepad, so be careful.
    #[macros::anyhow]
    fn OnLayoutChange(
        &self,
        _pic: Option<&ITfContext>,
        _lcode: TfLayoutCode,
        _pview: Option<&ITfContextView>,
    ) -> Result<()> {
        self.get_and_send_pos()?;

        Ok(())
    }
}

impl TextService {
    pub fn advise_text_layout_sink(&mut self, doc_mgr: ITfDocumentMgr) -> Result<()> {
        if IMEState::get()?.context.is_some() {
            self.unadvise_text_layout_sink()?;
        }

        unsafe {
            let context = doc_mgr.GetTop()?;

            IMEState::get()?.context = Some(context.clone());

            let cookie = context
                .cast::<ITfSource>()?
                .AdviseSink(&ITfTextLayoutSink::IID, &self.this::<ITfTextLayoutSink>()?)?;

            IMEState::get()?
                .cookies
                .insert(ITfTextLayoutSink::IID, cookie);

            Ok(())
        }
    }

    pub fn unadvise_text_layout_sink(&mut self) -> Result<()> {
        unsafe {
            let mut state = IMEState::get()?;

            if let Some(context) = state.context.take() {
                if let Some(cookie) = state.cookies.remove(&ITfTextLayoutSink::IID) {
                    context.cast::<ITfSource>()?.UnadviseSink(cookie)?;
                }
            }

            Ok(())
        }
    }
}
