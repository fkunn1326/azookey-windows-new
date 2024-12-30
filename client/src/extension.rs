use windows::{
    core::{GUID, HSTRING, PCWSTR},
    Win32::{
        System::Registry::{
            RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW, HKEY, KEY_WRITE,
            REG_OPTION_NON_VOLATILE, REG_SZ,
        },
        UI::Input::KeyboardAndMouse::{GetKeyState, VIRTUAL_KEY},
    },
};

use crate::check_win32_err;

// string extension
pub trait StringExt {
    fn to_wide_16(&self) -> Vec<u16>;
    fn to_wide_16_unpadded(&self) -> Vec<u16>;
    fn to_wide(&self) -> Vec<u8>;
}

impl StringExt for &str {
    fn to_wide_16(&self) -> Vec<u16> {
        self.encode_utf16().chain(Some(0)).collect()
    }

    fn to_wide_16_unpadded(&self) -> Vec<u16> {
        self.encode_utf16().collect()
    }

    fn to_wide(&self) -> Vec<u8> {
        self.encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .chain(Some(0))
            .collect()
    }
}

// guid extension
pub trait GUIDExt {
    fn to_string(&self) -> String;
}

impl GUIDExt for GUID {
    fn to_string(&self) -> String {
        format!(
            "{{{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}}}",
            self.data1,
            self.data2,
            self.data3,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7],
        )
    }
}

// registry extension

pub trait RegKey {
    fn create_subkey(&self, subkey: &str) -> windows::core::Result<HKEY>;
    fn set_string(&self, value_name: &str, value: &str) -> windows::core::Result<()>;
    fn delete_tree(&self, subkey: &str) -> windows::core::Result<()>;
    fn close(&self) -> windows::core::Result<()>;
}

impl RegKey for HKEY {
    fn create_subkey(&self, subkey_name: &str) -> windows::core::Result<HKEY> {
        let subkey_name_w = HSTRING::from(subkey_name);
        let mut subkey_handle: HKEY = HKEY::default();

        unsafe {
            let result = RegCreateKeyExW(
                *self,
                PCWSTR(subkey_name_w.as_ptr()),
                0,
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                None,
                &mut subkey_handle,
                None,
            );

            check_win32_err!(result, subkey_handle)
        }
    }

    fn set_string(&self, value_name: &str, value: &str) -> windows::core::Result<()> {
        let value_name_w = HSTRING::from(value_name);
        let value_w = value.to_wide();
        unsafe {
            let result = RegSetValueExW(
                *self,
                PCWSTR(value_name_w.as_ptr()),
                0,
                REG_SZ,
                Some(value_w.as_slice()),
            );

            check_win32_err!(result)
        }
    }

    fn delete_tree(&self, subkey: &str) -> windows::core::Result<()> {
        let subkey_w = HSTRING::from(subkey);
        unsafe {
            let result = RegDeleteTreeW(*self, PCWSTR(subkey_w.as_ptr()));

            check_win32_err!(result)
        }
    }

    fn close(&self) -> windows::core::Result<()> {
        unsafe {
            let result = RegCloseKey(*self);
            check_win32_err!(result)
        }
    }
}

#[allow(clippy::wrong_self_convention)]
pub trait VKeyExt {
    fn is_pressed(self) -> bool;
}

impl VKeyExt for VIRTUAL_KEY {
    fn is_pressed(self) -> bool {
        unsafe { GetKeyState(self.0 as i32) as u16 & 0x8000 != 0 }
    }
}
