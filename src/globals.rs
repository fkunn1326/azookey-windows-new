use std::sync::{Mutex, MutexGuard, OnceLock};

use windows::{
    core::GUID,
    Win32::{
        Foundation::{HMODULE, MAX_PATH},
        System::LibraryLoader::GetModuleFileNameW,
    },
};

pub const CLSID_PREFIX: &str = "CLSID\\";
pub const INPROC_SUFFIX: &str = "\\InProcServer32";

pub const SERVICE_NAME: &str = "Azookey";

// ffdefe79-2fc2-11ef-b16b-94e70b2c378c
pub const GUID_TEXT_SERVICE: GUID = GUID::from_u128(0xffdefe79_2fc2_11ef_b16b_94e70b2c378c);
// ffdefe7a-2fc2-11ef-b16b-94e70b2c378c
pub const GUID_PROFILE: GUID = GUID::from_u128(0xffdefe7a_2fc2_11ef_b16b_94e70b2c378c);

// DisplayAttribute用のGUID3つ
pub const GUID_DISPLAY_ATTRIBUTE_INPUT: GUID =
    GUID::from_u128(0xffdefe7b_2fc2_11ef_b16b_94e70b2c378c);
pub const GUID_DISPLAY_ATTRIBUTE_CONVERTED: GUID =
    GUID::from_u128(0xffdefe7c_2fc2_11ef_b16b_94e70b2c378c);
pub const GUID_DISPLAY_ATTRIBUTE_FOCUSED: GUID =
    GUID::from_u128(0xffdefe7d_2fc2_11ef_b16b_94e70b2c378c);

// これはなんだろう..?
// https://github.com/microsoft/Windows-classic-samples/blob/main/Samples/Win7Samples/winui/input/tsf/textservice/textservice-step04/LanguageBar.cpp#L23
pub const TEXTSERVICE_LANGBARITEMSINK_COOKIE: u32 = 0x0fab0fab;

pub static DLL_INSTANCE: OnceLock<Mutex<DllModule>> = OnceLock::new();

unsafe impl Sync for DllModule {}
unsafe impl Send for DllModule {}

#[derive(Debug)]
pub struct DllModule {
    ref_count: u32,
    ref_lock: u32,
    pub hinst: HMODULE,
}

impl DllModule {
    pub fn new() -> Self {
        Self {
            ref_count: 0,
            ref_lock: 0,
            hinst: HMODULE::default(),
        }
    }

    pub fn get<'a>() -> anyhow::Result<MutexGuard<'a, DllModule>> {
        DLL_INSTANCE
            .get()
            .ok_or_else(|| anyhow::anyhow!("DllModule is not initialized"))?
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    pub fn get_path() -> anyhow::Result<String> {
        let path = {
            let dll_instance = DllModule::get()?.hinst;

            let mut buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
            let length = unsafe { GetModuleFileNameW(dll_instance, &mut buffer) };

            String::from_utf16_lossy(&buffer[..length as usize])
        };
        Ok(path)
    }

    pub fn lock(&mut self) -> u32 {
        self.ref_lock += 1;
        self.ref_lock
    }

    pub fn unlock(&mut self) -> u32 {
        self.ref_lock -= 1;
        self.ref_lock
    }

    pub fn can_unload(&self) -> bool {
        self.ref_count == 0 && self.ref_lock == 0
    }
}
