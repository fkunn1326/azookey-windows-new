use windows::{
    core::{IUnknown, Interface as _, Result, BSTR, GUID, PCWSTR},
    Win32::{
        Foundation::{BOOL, E_FAIL, E_INVALIDARG, POINT, RECT},
        System::Ole::CONNECT_E_CANNOTCONNECT,
        UI::{
            TextServices::{
                ITfLangBarItemButton_Impl, ITfLangBarItemSink, ITfLangBarItem_Impl, ITfMenu,
                ITfSource_Impl, TfLBIClick, GUID_LBI_INPUTMODE, TF_LANGBARITEMINFO,
                TF_LBI_STYLE_BTN_BUTTON,
            },
            WindowsAndMessaging::{LoadImageW, HICON, IMAGE_ICON, LR_DEFAULTCOLOR},
        },
    },
};

use crate::{
    engine::input_mode::InputMode,
    globals::{DllModule, GUID_TEXT_SERVICE, TEXTSERVICE_LANGBARITEMSINK_COOKIE},
};

use super::factory::TextServiceFactory_Impl;

const INFO: TF_LANGBARITEMINFO = TF_LANGBARITEMINFO {
    clsidService: GUID_TEXT_SERVICE,
    guidItem: GUID_LBI_INPUTMODE,
    dwStyle: TF_LBI_STYLE_BTN_BUTTON,
    ulSort: 0,
    szDescription: [0; 32],
};

// you need to implement these three interfaces to create a language bar item
// if not, you will get E_FAIL error in ITfLangBarItemMgr::AddItem

impl ITfLangBarItem_Impl for TextServiceFactory_Impl {
    fn GetInfo(&self, p_info: *mut TF_LANGBARITEMINFO) -> Result<()> {
        unsafe {
            *p_info = INFO;
        }
        Ok(())
    }

    fn GetStatus(&self) -> Result<u32> {
        Ok(0)
    }

    fn Show(&self, _f_show: BOOL) -> Result<()> {
        Ok(())
    }

    // this will be shown as a tooltip when you hover the language bar item
    fn GetTooltipString(&self) -> Result<BSTR> {
        Ok(BSTR::default())
    }
}

impl ITfLangBarItemButton_Impl for TextServiceFactory_Impl {
    fn OnClick(&self, _click: TfLBIClick, _pt: &POINT, _prcarea: *const RECT) -> Result<()> {
        let mode = {
            let text_service = self.borrow()?;
            match text_service.mode {
                InputMode::Latin => InputMode::Kana,
                InputMode::Kana => InputMode::Latin,
            }
        };

        self.set_input_mode(mode)?;

        Ok(())
    }

    // this method should not be called
    fn InitMenu(&self, _pmenu: Option<&ITfMenu>) -> Result<()> {
        Ok(())
    }

    // this method should not be called
    fn OnMenuSelect(&self, _w_id: u32) -> Result<()> {
        Ok(())
    }

    fn GetIcon(&self) -> Result<HICON> {
        let dll_module = DllModule::get().map_err(|e| {
            log::error!("Failed to get DllModule {:?}", e);
            E_FAIL
        })?;

        let icon_id = match self.borrow()?.mode {
            InputMode::Kana => 102,
            InputMode::Latin => 103,
        };

        unsafe {
            let handle = LoadImageW(
                dll_module.hinst,
                PCWSTR(icon_id as *mut u16),
                IMAGE_ICON,
                0,
                0,
                LR_DEFAULTCOLOR,
            )?;

            Ok(HICON(handle.0))
        }
    }

    fn GetText(&self) -> Result<BSTR> {
        Ok(BSTR::default())
    }
}

impl ITfSource_Impl for TextServiceFactory_Impl {
    fn AdviseSink(&self, riid: *const GUID, punk: Option<&IUnknown>) -> windows::core::Result<u32> {
        let riid = unsafe { *riid };

        if riid != ITfLangBarItemSink::IID {
            return Err(E_INVALIDARG.into());
        }

        if punk.is_none() {
            return Err(E_INVALIDARG.into());
        }

        Ok(TEXTSERVICE_LANGBARITEMSINK_COOKIE)
    }

    fn UnadviseSink(&self, dw_cookie: u32) -> windows::core::Result<()> {
        if dw_cookie != TEXTSERVICE_LANGBARITEMSINK_COOKIE {
            return Err(CONNECT_E_CANNOTCONNECT.into());
        }

        Ok(())
    }
}
