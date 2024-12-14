mod extension;
mod globals;
mod macros;
mod register;
mod utils;

use std::{ffi::c_void, sync::Mutex};

use globals::{DllModule, DLL_INSTANCE};
use register::{CLSIDMgr, CategoryMgr, ProfileMgr};
use windows::{
    core::HRESULT,
    Win32::{
        Foundation::{HMODULE, S_FALSE, S_OK},
        System::{Ole::SELFREG_E_CLASS, SystemServices::DLL_PROCESS_ATTACH},
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

    log::info!("DllMain");

    check_err!(result, true, false)
}

#[no_mangle]
pub extern "system" fn DllGetClassObject() -> HRESULT {
    // Return a class factory to obtain the tsf TextService
    S_OK
}

#[no_mangle]
pub extern "system" fn DllRegisterServer() -> HRESULT {
    // Register the CLSID of the TextService
    // Called when the DLL is registered using regsvr32
    log::info!("DllRegisterServer");

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
    log::info!("DllUnregisterServer");

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
