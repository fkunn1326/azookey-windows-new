use std::{
    cell::Cell,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};
use windows::{
    core::{implement, BSTR, GUID},
    Win32::UI::TextServices::{
        IEnumTfDisplayAttributeInfo, IEnumTfDisplayAttributeInfo_Impl, ITfDisplayAttributeInfo,
        ITfDisplayAttributeInfo_Impl, ITfDisplayAttributeProvider_Impl, TF_DISPLAYATTRIBUTE,
    },
};

use anyhow::Result;

use crate::globals::{DISPLAY_ATTRIBUTE, GUID_DISPLAY_ATTRIBUTE};

use super::factory::TextServiceFactory_Impl;

// class for display attribute (color, bold, underline, etc.)
impl ITfDisplayAttributeProvider_Impl for TextServiceFactory_Impl {
    #[macros::anyhow]
    fn EnumDisplayAttributeInfo(&self) -> windows::core::Result<IEnumTfDisplayAttributeInfo> {
        let enum_info = EnumDisplayAttributeInfo::new();
        Ok(enum_info.into())
    }

    #[macros::anyhow]
    fn GetDisplayAttributeInfo(
        &self,
        guid: *const windows_core::GUID,
    ) -> windows::core::Result<ITfDisplayAttributeInfo> {
        let guid = unsafe { *guid };
        let attributes = EnumDisplayAttributeInfo::new();
        for attribute in attributes.attributes {
            if attribute.guid == guid {
                return Ok(attribute.into());
            }
        }
        anyhow::bail!("Display attribute not found");
    }
}

#[derive(Clone)]
#[implement(ITfDisplayAttributeInfo)]
pub struct DisplayAttributeInfo {
    pub guid: GUID,
    attribute: Cell<TF_DISPLAYATTRIBUTE>,
    attribute_backup: TF_DISPLAYATTRIBUTE,
}

impl DisplayAttributeInfo {
    pub fn new(guid: GUID, attribute: TF_DISPLAYATTRIBUTE) -> Self {
        DisplayAttributeInfo {
            guid,
            attribute: Cell::new(attribute),
            attribute_backup: attribute,
        }
    }
}

impl ITfDisplayAttributeInfo_Impl for DisplayAttributeInfo_Impl {
    #[macros::anyhow]
    fn GetAttributeInfo(&self, pda: *mut TF_DISPLAYATTRIBUTE) -> Result<()> {
        unsafe {
            *pda = self.attribute.get();
        }
        Ok(())
    }

    #[macros::anyhow]
    fn GetGUID(&self) -> Result<GUID> {
        Ok(self.guid)
    }

    #[macros::anyhow]
    fn Reset(&self) -> Result<()> {
        self.attribute.set(self.attribute_backup);
        Ok(())
    }

    #[macros::anyhow]
    fn GetDescription(&self) -> Result<BSTR> {
        Ok(BSTR::default())
    }

    #[macros::anyhow]
    fn SetAttributeInfo(&self, pda: *const TF_DISPLAYATTRIBUTE) -> Result<()> {
        unsafe {
            self.attribute.set(*pda);
        }
        Ok(())
    }
}

#[implement(IEnumTfDisplayAttributeInfo)]
pub struct EnumDisplayAttributeInfo {
    pub attributes: Vec<DisplayAttributeInfo>,
    index: AtomicUsize,
}

#[allow(clippy::new_without_default)]
impl EnumDisplayAttributeInfo {
    pub fn new() -> Self {
        let attributes = vec![DisplayAttributeInfo::new(
            GUID_DISPLAY_ATTRIBUTE,
            DISPLAY_ATTRIBUTE,
        )];

        EnumDisplayAttributeInfo {
            attributes,
            index: AtomicUsize::new(0),
        }
    }
}

impl IEnumTfDisplayAttributeInfo_Impl for EnumDisplayAttributeInfo_Impl {
    #[macros::anyhow]
    fn Clone(&self) -> Result<IEnumTfDisplayAttributeInfo> {
        let clone = EnumDisplayAttributeInfo::new();
        clone.index.store(self.index.load(Relaxed), Relaxed);
        Ok(clone.into())
    }

    #[macros::anyhow]
    fn Next(
        &self,
        ulcount: u32,
        rginfo: *mut Option<ITfDisplayAttributeInfo>,
        pcfetched: *mut u32,
    ) -> Result<()> {
        unsafe {
            if ulcount == 0 {
                return Ok(());
            }

            let mut fetched = 0;
            let mut index = self.index.load(Relaxed);

            while fetched < ulcount && index < self.attributes.len() {
                let attribute = match self.attributes.get(index).cloned() {
                    Some(attr) => attr,
                    None => anyhow::bail!("Display attribute not found"),
                };
                *rginfo = Some(attribute.into());
                fetched += 1;
                index += 1;
            }

            self.index.store(index, Relaxed);
            *pcfetched = fetched;
        }
        Ok(())
    }

    #[macros::anyhow]
    fn Reset(&self) -> Result<()> {
        self.index.store(0, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    #[macros::anyhow]
    fn Skip(&self, ulcount: u32) -> Result<()> {
        let index = self.index.load(Relaxed) + ulcount as usize;
        self.index.store(index, Relaxed);
        Ok(())
    }
}
