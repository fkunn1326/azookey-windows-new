use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use windows::{
    core::{Interface, Result, GUID},
    Win32::{
        Foundation::E_FAIL,
        UI::TextServices::{ITfContext, ITfTextInputProcessor, ITfThreadMgr},
    },
};

use crate::engine::{composition::Composition, input_mode::InputMode};

#[derive(Default, Debug)]
pub struct TextService {
    pub tid: u32,
    pub thread_mgr: Option<ITfThreadMgr>,
    pub cookies: HashMap<GUID, u32>,
    pub context: Option<ITfContext>,
    pub composition: RefCell<Composition>,
    pub display_attribute_atom: HashMap<GUID, u32>,
    pub mode: InputMode,
    pub this: Option<ITfTextInputProcessor>,
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

    pub fn context<I: Interface>(&self) -> Result<I> {
        if let Some(context) = self.context.as_ref() {
            context.cast()
        } else {
            Err(E_FAIL.into())
        }
    }

    pub fn borrow_composition(&self) -> Result<Ref<Composition>> {
        Ok(self.composition.try_borrow().map_err(|e| {
            log::error!("Failed to borrow composition: {:#}", e);
            E_FAIL
        })?)
    }

    pub fn borrow_mut_composition(&self) -> Result<RefMut<Composition>> {
        Ok(self.composition.try_borrow_mut().map_err(|e| {
            log::error!("Failed to write composition: {:#}", e);
            E_FAIL
        })?)
    }
}
