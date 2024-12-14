use windows::{
    core::Result,
    Win32::UI::TextServices::{
        ITfTextInputProcessorEx_Impl, ITfTextInputProcessor_Impl, ITfThreadMgr,
    },
};

use super::factory::TextServiceFactory_Impl;

impl ITfTextInputProcessor_Impl for TextServiceFactory_Impl {
    fn Activate(&self, ptim: Option<&ITfThreadMgr>, tid: u32) -> Result<()> {
        log::info!("Activated with tid: {tid}");
        let mut text_service = self.borrow_mut();

        text_service.tid = tid;
        if let Some(ptim) = ptim {
            text_service.thread_mgr = Some(ptim.clone());
        }

        Ok(())
    }

    fn Deactivate(&self) -> Result<()> {
        log::info!("Deactivated");
        let mut text_service = self.borrow_mut();

        text_service.tid = 0;
        text_service.thread_mgr = None;

        Ok(())
    }
}

impl ITfTextInputProcessorEx_Impl for TextServiceFactory_Impl {
    fn ActivateEx(&self, ptim: Option<&ITfThreadMgr>, tid: u32, _dwflags: u32) -> Result<()> {
        log::info!("Activated(Ex) with tid: {tid}");
        self.Activate(ptim, tid)
    }
}
