use windows::{
    core::{w, GUID},
    Win32::{
        Globalization::LocaleNameToLCID,
        System::{
            Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
            Registry::HKEY_CLASSES_ROOT,
        },
        UI::{
            Input::KeyboardAndMouse::HKL,
            TextServices::{
                CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles, ITfCategoryMgr,
                ITfInputProcessorProfileMgr, GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
                GUID_TFCAT_TIPCAP_COMLESS, GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
                GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT, GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
                GUID_TFCAT_TIPCAP_UIELEMENTENABLED, GUID_TFCAT_TIP_KEYBOARD,
            },
        },
    },
};

use crate::{
    extension::{GUIDExt as _, RegKey as _, StringExt as _},
    globals::{CLSID_PREFIX, GUID_PROFILE, GUID_TEXT_SERVICE, INPROC_SUFFIX, SERVICE_NAME},
};

// register the information (lang, icon, name, path_of_dll) of the TextService
pub struct ProfileMgr;

impl ProfileMgr {
    pub fn register(dll_path: &str) -> anyhow::Result<()> {
        unsafe {
            let profiles: ITfInputProcessorProfileMgr =
                CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;

            let langid: u16 = LocaleNameToLCID(w!("ja-JP"), 0).try_into()?;

            Ok(profiles.RegisterProfile(
                &GUID_TEXT_SERVICE,
                langid,
                &GUID_PROFILE,
                SERVICE_NAME.to_wide_16().as_slice(),
                dll_path.to_wide_16().as_slice(),
                0,
                HKL::default(),
                0,
                true,
                0,
            )?)
        }
    }

    pub fn unregister() -> anyhow::Result<()> {
        unsafe {
            let profiles: ITfInputProcessorProfileMgr =
                CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;

            let langid: u16 = LocaleNameToLCID(w!("ja-JP"), 0).try_into()?;

            Ok(profiles.UnregisterProfile(&GUID_TEXT_SERVICE, langid, &GUID_PROFILE, 0)?)
        }
    }
}

// register the CLSID(unique id for textservice) of the TextService
pub struct CLSIDMgr;
impl CLSIDMgr {
    pub fn register(dll_path: &str) -> anyhow::Result<()> {
        let clsid_key = CLSID_PREFIX.to_owned() + &GUID_TEXT_SERVICE.to_string();
        let inproc_key = clsid_key.clone() + INPROC_SUFFIX;

        let hkey = HKEY_CLASSES_ROOT.create_subkey(&clsid_key)?;
        hkey.set_string("", SERVICE_NAME)?;
        hkey.close()?;

        let inproc_hkey = HKEY_CLASSES_ROOT.create_subkey(&inproc_key)?;
        inproc_hkey.set_string("", dll_path)?;
        inproc_hkey.set_string("ThreadingModel", "Apartment")?;
        inproc_hkey.close()?;

        Ok(())
    }

    pub fn unregister() -> anyhow::Result<()> {
        let clsid_key = CLSID_PREFIX.to_owned() + &GUID_TEXT_SERVICE.to_string();
        let inproc_key = clsid_key.clone() + INPROC_SUFFIX;

        HKEY_CLASSES_ROOT.delete_tree(&clsid_key)?;
        HKEY_CLASSES_ROOT.delete_tree(&inproc_key)?;

        Ok(())
    }
}

// register the category (entitlements of textservice) of the TextService
pub struct CategoryMgr;
impl CategoryMgr {
    const CATEGORIES: [GUID; 7] = [
        GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
        GUID_TFCAT_TIPCAP_COMLESS,
        GUID_TFCAT_TIPCAP_INPUTMODECOMPARTMENT,
        GUID_TFCAT_TIPCAP_UIELEMENTENABLED,
        GUID_TFCAT_TIP_KEYBOARD,
        GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
        GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
    ];

    pub fn register() -> anyhow::Result<()> {
        unsafe {
            let catmgr: ITfCategoryMgr =
                CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;

            for cat in Self::CATEGORIES.iter() {
                catmgr.RegisterCategory(&GUID_TEXT_SERVICE, cat, &GUID_TEXT_SERVICE)?;
            }

            Ok(())
        }
    }

    pub fn unregister() -> anyhow::Result<()> {
        unsafe {
            let catmgr: ITfCategoryMgr =
                CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;

            for cat in Self::CATEGORIES.iter() {
                catmgr.UnregisterCategory(&GUID_TEXT_SERVICE, cat, &GUID_TEXT_SERVICE)?;
            }

            Ok(())
        }
    }
}
