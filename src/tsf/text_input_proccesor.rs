use std::collections::HashMap;

use crate::globals::GUID_DISPLAY_ATTRIBUTE;

use super::factory::TextServiceFactory_Impl;
use windows::{
    core::{Interface as _, Result},
    Win32::{
        Foundation::{BOOL, E_FAIL},
        System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
        UI::TextServices::{
            CLSID_TF_CategoryMgr, ITfCategoryMgr, ITfKeyEventSink, ITfKeystrokeMgr, ITfSource,
            ITfTextInputProcessorEx_Impl, ITfTextInputProcessor_Impl, ITfThreadMgr,
            ITfThreadMgrEventSink,
        },
    },
};

impl ITfTextInputProcessor_Impl for TextServiceFactory_Impl {
    fn Activate(&self, ptim: Option<&ITfThreadMgr>, tid: u32) -> Result<()> {
        log::debug!("Activated with tid: {tid}");
        let mut text_service = self.borrow_mut()?;

        text_service.tid = tid;
        let thread_mgr = ptim.ok_or(E_FAIL)?;
        text_service.thread_mgr = Some(thread_mgr.clone());

        // initialize key event sink
        log::debug!("AdviseKeyEventSink");

        unsafe {
            thread_mgr.cast::<ITfKeystrokeMgr>()?.AdviseKeyEventSink(
                tid,
                &text_service.this::<ITfKeyEventSink>()?,
                BOOL::from(true),
            )?;
        };

        // initialize thread manager event sink
        log::debug!("AdviseThreadMgrEventSink");
        unsafe {
            let cookie = thread_mgr.cast::<ITfSource>()?.AdviseSink(
                &ITfThreadMgrEventSink::IID,
                &text_service.this::<ITfThreadMgrEventSink>()?,
            )?;
            text_service.cookie = Some(cookie);
        };

        // initialize display attribute
        let atom_map = unsafe {
            let mut map = HashMap::new();
            let category_mgr: ITfCategoryMgr =
                CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;

            let atom = category_mgr.RegisterGUID(&GUID_DISPLAY_ATTRIBUTE)?;
            map.insert(GUID_DISPLAY_ATTRIBUTE, atom);
            map
        };

        text_service.display_attribute_atom = atom_map;

        log::debug!("AdviseKeyEventSink success");

        Ok(())
    }

    fn Deactivate(&self) -> Result<()> {
        log::debug!("Deactivated");
        {
            let text_service = self.borrow()?;
            let thread_mgr = text_service.thread_mgr()?;

            // end composition
            self.end_composition()?;
            let mut composition = text_service.borrow_mut_composition()?;

            if composition.tip_composition.is_some() {
                composition.tip_composition = None;
            }

            // remove key event sink
            log::debug!("UnadviseKeyEventSink");
            unsafe {
                thread_mgr
                    .cast::<ITfKeystrokeMgr>()?
                    .UnadviseKeyEventSink(text_service.tid)?;
            };
        }

        let mut text_service = self.borrow_mut()?;
        let thread_mgr = text_service.thread_mgr()?;

        // remove thread manager event sink
        log::debug!("UnadviseThreadMgrEventSink");
        unsafe {
            if let Some(cookie) = text_service.cookie {
                thread_mgr.cast::<ITfSource>()?.UnadviseSink(cookie)?;
                text_service.cookie = None;
            }
        };

        // clear display attribute
        text_service.display_attribute_atom.clear();

        text_service.tid = 0;
        text_service.thread_mgr = None;

        log::debug!("UnadviseKeyEventSink success");
        Ok(())
    }
}

impl ITfTextInputProcessorEx_Impl for TextServiceFactory_Impl {
    fn ActivateEx(&self, ptim: Option<&ITfThreadMgr>, tid: u32, _dwflags: u32) -> Result<()> {
        // called when the text service is activated
        // if this function is implemented, the Activate() function won't be called
        // so we need to call the Activate function manually
        log::debug!("Activated(Ex) with tid: {tid}");
        self.Activate(ptim, tid)?;
        Ok(())
    }
}
