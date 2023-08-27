use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use wasmtime::{Caller, Store};
use wasmtime_wasi::WasiCtx;

use crate::state::shell::ShellState;

use self::memory::manager::WasmMemoryManager;

pub mod host;
pub mod loader;
pub mod memory;
pub mod plugin;

pub type StoreData = Box<dyn PluginContext>;

/// Allows access to the shell and plugin state from inside code exposed to a plugin
pub trait PluginContext: Send + Sync {
    fn memory(&self) -> Arc<dyn WasmMemoryManager>;
    fn set_memory_manager(&mut self, manager: Arc<dyn WasmMemoryManager>);
    fn wasi(&mut self) -> &mut Option<WasiCtx>;
    fn shell(&self) -> RwLockReadGuard<'_, ShellState>;
    fn shell_mut(&mut self) -> RwLockWriteGuard<'_, ShellState>;
}

pub struct WasmPluginContext {
    wasi: Option<WasiCtx>,
    shell: Arc<RwLock<ShellState>>,
    memory: Option<Arc<dyn WasmMemoryManager>>,
}

impl PluginContext for WasmPluginContext {
    fn memory(&self) -> Arc<dyn WasmMemoryManager> {
        self.memory
            .as_ref()
            .expect("Cannot access plugin memory before instantiation is complete")
            .clone()
    }
    fn set_memory_manager(&mut self, manager: Arc<dyn WasmMemoryManager>) {
        self.memory = Some(manager);
    }
    fn wasi(&mut self) -> &mut Option<WasiCtx> {
        &mut self.wasi
    }
    fn shell(&self) -> RwLockReadGuard<'_, ShellState> {
        self.shell.read().unwrap()
    }

    fn shell_mut(&mut self) -> RwLockWriteGuard<'_, ShellState> {
        self.shell.write().unwrap()
    }
}
