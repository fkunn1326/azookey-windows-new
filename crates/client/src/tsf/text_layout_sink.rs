use windows::{
    core::Interface as _,
    Win32::UI::TextServices::{
        ITfContext, ITfContextView, ITfSource, ITfTextLayoutSink, ITfTextLayoutSink_Impl,
        TfLayoutCode,
    },
};

use anyhow::Result;

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
    pub fn advise_text_layout_sink(&mut self) -> Result<()> {
        unsafe {
            let doc_mgr = self.thread_mgr()?.GetFocus()?;
            let context = doc_mgr.GetBase()?;

            let cookie = context
                .cast::<ITfSource>()?
                .AdviseSink(&ITfTextLayoutSink::IID, &self.this::<ITfTextLayoutSink>()?)?;

            self.cookies.insert(ITfTextLayoutSink::IID, cookie);

            Ok(())
        }
    }

    pub fn unadvise_text_layout_sink(&mut self) -> Result<()> {
        unsafe {
            if let Some(cookie) = self.cookies.remove(&ITfTextLayoutSink::IID) {
                let doc_mgr = self.thread_mgr()?.GetFocus()?;
                let context = doc_mgr.GetBase()?;
                context.cast::<ITfSource>()?.UnadviseSink(cookie)?;
            }

            Ok(())
        }
    }
}
