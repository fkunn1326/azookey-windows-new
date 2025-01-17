use std::sync::{LazyLock, Mutex, MutexGuard};

use super::{input_mode::InputMode, ipc_service::IPCService};

#[derive(Default, Debug)]
pub struct IMEState {
    pub ipc_service: IPCService,
    pub input_mode: InputMode,
}

pub static IME_STATE: LazyLock<Mutex<IMEState>> = LazyLock::new(|| Mutex::new(IMEState::default()));

impl IMEState {
    pub fn get() -> anyhow::Result<MutexGuard<'static, IMEState>> {
        match IME_STATE.try_lock() {
            Ok(guard) => Ok(guard),
            Err(e) => anyhow::bail!("Failed to lock state: {:?}", e),
        }
    }
}
