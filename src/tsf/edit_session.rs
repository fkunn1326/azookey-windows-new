use windows::{
    core::{implement, Result},
    Win32::{
        Foundation::E_FAIL,
        UI::TextServices::{ITfContext, ITfEditSession, ITfEditSession_Impl, TF_ES_READWRITE},
    },
};

use std::rc::Rc;

#[implement(ITfEditSession)]
struct EditSession {
    callback: Rc<dyn Fn(u32) -> anyhow::Result<()>>,
}

// edit action will be performed within this function
pub fn edit_session(
    client_id: u32,
    context: ITfContext,
    callback: Rc<dyn Fn(u32) -> anyhow::Result<()>>,
) -> Result<()> {
    let session: ITfEditSession = EditSession::new(callback).into();

    let result = unsafe { context.RequestEditSession(client_id, &session, TF_ES_READWRITE) };

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

impl EditSession {
    pub fn new(callback: Rc<dyn Fn(u32) -> anyhow::Result<()>>) -> Self {
        EditSession { callback }
    }
}

impl ITfEditSession_Impl for EditSession_Impl {
    fn DoEditSession(&self, cookie: u32) -> windows::core::Result<()> {
        (self.callback)(cookie).map_err(|e| windows::core::Error::new(E_FAIL, e.to_string()))?;
        Ok(())
    }
}
