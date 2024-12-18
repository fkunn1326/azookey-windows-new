mod engine;
mod extension;
mod globals;
mod macros;
mod register;
mod tsf;
mod utils;

use std::{ffi::c_void, sync::Mutex};

use globals::{DllModule, DLL_INSTANCE, GUID_TEXT_SERVICE};
use register::{CLSIDMgr, CategoryMgr, ProfileMgr};
use tsf::factory::TextServiceFactory;
use windows::{
    core::{IUnknown, Interface as _, GUID, HRESULT},
    Win32::{
        Foundation::{CLASS_E_CLASSNOTAVAILABLE, E_UNEXPECTED, HMODULE, S_FALSE},
        System::{Com::IClassFactory, Ole::SELFREG_E_CLASS, SystemServices::DLL_PROCESS_ATTACH},
    },
};
// -- Dll Export Functions --
// The IME DLL needs to implement the following four functions to operate as a COM server.

#[no_mangle]
pub extern "system" fn DllMain(
    hinst: HMODULE,
    fdw_reason: u32,
    _lpv_reserved: *mut c_void,
) -> bool {
    if fdw_reason != DLL_PROCESS_ATTACH {
        return true;
    }
    // use unwrap only in this function
    utils::log::setup_logger().unwrap();

    let result: anyhow::Result<()> = (|| {
        let mut dll_instance = DllModule::new();
        dll_instance.hinst = hinst;
        DLL_INSTANCE
            .set(Mutex::new(dll_instance))
            .map_err(|e| anyhow::anyhow!(format!("{:?}", e)))?;
        Ok(())
    })();

    log::debug!("DllMain");

    check_err!(result, true, false)
}

#[no_mangle]
/// # Safety
/// This function uses raw pointers
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    // Return a class factory to obtain the tsf TextService
    // This function will be called only once when applications request the TextService
    // So, You have to reopen the application to apply the changes in the TextService
    // https://zenn.dev/link/comments/d918e46723da80
    log::debug!("DllGetClassObject");

    let result: anyhow::Result<()> = (|| {
        let rclsid = unsafe { *rclsid };
        let riid = unsafe { *riid };
        let ppv = unsafe { &mut *ppv };

        if rclsid != GUID_TEXT_SERVICE {
            return Err(anyhow::anyhow!(CLASS_E_CLASSNOTAVAILABLE));
        }

        if riid != IClassFactory::IID {
            return Err(anyhow::anyhow!(E_UNEXPECTED));
        }

        *ppv = match riid {
            IUnknown::IID => std::mem::transmute::<IUnknown, *mut c_void>(IUnknown::from(
                TextServiceFactory::default(),
            )),
            IClassFactory::IID => std::mem::transmute::<IClassFactory, *mut c_void>(
                IClassFactory::from(TextServiceFactory::default()),
            ),
            _ => return Err(anyhow::anyhow!(E_UNEXPECTED)),
        };
        Ok(())
    })();

    check_err!(result)
}

#[no_mangle]
pub extern "system" fn DllRegisterServer() -> HRESULT {
    // Register the CLSID of the TextService
    // Called when the DLL is registered using regsvr32
    log::debug!("DllRegisterServer");

    let result: anyhow::Result<()> = (|| {
        let dll_path = DllModule::get_path()?;

        ProfileMgr::register(&dll_path)?;
        CLSIDMgr::register(&dll_path)?;
        CategoryMgr::register()?;

        Ok(())
    })();

    // to show the error, SELFREG_E_CLASS is needed
    check_err!(result, SELFREG_E_CLASS)
}

#[no_mangle]
pub extern "system" fn DllUnregisterServer() -> HRESULT {
    // Unregister the CLSID of the TextService
    // Called when the DLL is unregistered using regsvr32
    log::debug!("DllUnregisterServer");

    let result: anyhow::Result<()> = (|| {
        ProfileMgr::unregister()?;
        CLSIDMgr::unregister()?;
        CategoryMgr::unregister()?;

        Ok(())
    })();

    check_err!(result, SELFREG_E_CLASS)
}

#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    let result: anyhow::Result<()> = (|| {
        let dll_instance = DllModule::get()?;
        if dll_instance.can_unload() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(S_FALSE))
        }
    })();

    check_err!(result)
}
