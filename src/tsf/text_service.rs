use windows::Win32::UI::TextServices::{ITfTextInputProcessor, ITfThreadMgr};

#[derive(Default)]
pub struct TextService {
    pub(super) tid: u32,
    pub(super) thread_mgr: Option<ITfThreadMgr>,
    pub(super) this: Option<ITfTextInputProcessor>,
}
