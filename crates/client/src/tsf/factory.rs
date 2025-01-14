use std::{
    cell::{Ref, RefCell, RefMut},
    ffi::c_void,
};

use windows::{
    core::{implement, AsImpl, IUnknown, Interface, Result, GUID},
    Win32::{
        Foundation::{BOOL, E_FAIL, E_NOINTERFACE},
        System::Com::{IClassFactory, IClassFactory_Impl},
        UI::TextServices::{
            ITfCompositionSink, ITfDisplayAttributeProvider, ITfKeyEventSink, ITfLangBarItem,
            ITfLangBarItemButton, ITfSource, ITfTextInputProcessor, ITfTextInputProcessorEx,
            ITfThreadMgrEventSink,
        },
    },
};

use crate::{globals::DllModule, handle_result};

use super::text_service::TextService;

#[derive(Default)]
#[implement(
    IClassFactory,
    ITfTextInputProcessor,
    ITfTextInputProcessorEx,
    ITfKeyEventSink,
    ITfThreadMgrEventSink,
    ITfCompositionSink,
    ITfDisplayAttributeProvider,
    ITfLangBarItem,
    ITfLangBarItemButton,
    ITfSource
)]
pub struct TextServiceFactory {
    text_service: RefCell<TextService>,
}

impl IClassFactory_Impl for TextServiceFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Option<&IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut c_void,
    ) -> Result<()> {
        let riid = unsafe { *riid };
        let ppvobject = unsafe { &mut *ppvobject };

        *ppvobject = std::ptr::null_mut();

        if punkouter.is_some() {
            return Err(E_NOINTERFACE.into());
        }

        unsafe {
            *ppvobject = match riid {
                ITfTextInputProcessor::IID => {
                    std::mem::transmute::<ITfTextInputProcessor, *mut c_void>(
                        TextServiceFactory::create::<ITfTextInputProcessor>()?,
                    )
                }
                ITfTextInputProcessorEx::IID => {
                    std::mem::transmute::<ITfTextInputProcessorEx, *mut c_void>(
                        TextServiceFactory::create::<ITfTextInputProcessorEx>()?,
                    )
                }
                _ => return Err(E_NOINTERFACE.into()),
            };
        }

        Ok(())
    }

    fn LockServer(&self, flock: BOOL) -> Result<()> {
        let result: anyhow::Result<()> = (|| {
            let mut dll_instance = DllModule::get()?;
            if flock.as_bool() {
                dll_instance.lock();
            } else {
                dll_instance.unlock();
            }

            Ok(())
        })();

        handle_result!(result)
    }
}

impl TextServiceFactory {
    pub fn create<I: Interface>() -> Result<I> {
        let factory = Self {
            text_service: RefCell::new(TextService::default()),
        };

        let this = ITfTextInputProcessor::from(factory);
        let factory = unsafe { this.as_impl() };
        factory.borrow_mut()?.this = Some(this.clone());

        unsafe { factory.cast::<I>() }
    }

    pub fn borrow_mut(&self) -> Result<RefMut<TextService>> {
        Ok(self.text_service.try_borrow_mut().map_err(|e| {
            log::error!("Failed to borrow_mut {:#}", e);
            E_FAIL
        })?)
    }

    pub fn borrow(&self) -> Result<Ref<TextService>> {
        Ok(self.text_service.try_borrow().map_err(|e| {
            log::error!("Failed to borrow {:#}", e);
            E_FAIL
        })?)
    }
}
