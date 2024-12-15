use windows::{
    core::{Result, GUID},
    Win32::{
        Foundation::{BOOL, LPARAM, WPARAM},
        UI::TextServices::{ITfContext, ITfKeyEventSink_Impl},
    },
};

use super::factory::TextServiceFactory_Impl;

// sink (aka event listener) for key events
impl ITfKeyEventSink_Impl for TextServiceFactory_Impl {
    fn OnTestKeyDown(
        &self,
        _pic: Option<&ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        // this function checks if the key event will be handled by "OnKeyUp" function
        // so we need to return TRUE if we want to handle the key event
        Ok(true.into())
    }

    fn OnTestKeyUp(
        &self,
        _pic: Option<&ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        // same as OnTestKeyDown
        Ok(true.into())
    }

    fn OnKeyDown(
        &self,
        _pic: Option<&ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        // this function is called when a key is pressed
        // we can handle key events here
        log::debug!("Key Down: {wparam:?}");
        Ok(true.into())
    }

    fn OnKeyUp(&self, _pic: Option<&ITfContext>, _wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        // this function is called when a key is released
        // but we handle key events in OnKeyDown function
        // so just return S_OK
        Ok(true.into())
    }

    fn OnPreservedKey(&self, _pic: Option<&ITfContext>, _rguid: *const GUID) -> Result<BOOL> {
        // this function is actually not used
        Ok(true.into())
    }

    fn OnSetFocus(&self, _fforeground: BOOL) -> Result<()> {
        Ok(())
    }
}
