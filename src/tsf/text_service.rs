use windows::{
    core::{Interface, Result},
    Win32::{
        Foundation::E_FAIL,
        UI::TextServices::{ITfTextInputProcessor, ITfThreadMgr},
    },
};

#[derive(Default)]
pub struct TextService {
    pub(super) tid: u32,
    pub(super) thread_mgr: Option<ITfThreadMgr>,
    pub(super) cookie: Option<u32>,
    pub(super) this: Option<ITfTextInputProcessor>,
}

impl TextService {
    pub fn this<I: Interface>(&self) -> Result<I> {
        if let Some(this) = self.this.as_ref() {
            this.cast()
        } else {
            Err(E_FAIL.into())
        }
    }

    pub fn thread_mgr(&self) -> Result<ITfThreadMgr> {
        self.thread_mgr.clone().ok_or(E_FAIL.into())
    }
}
