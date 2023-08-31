use std::sync::{Arc, RwLock};

use serde_json::Value;
use wasmtime::{AsContextMut, Engine, Instance, Linker, Module, Store, Val, ValType};
use wasmtime_wasi::WasiCtxBuilder;

use super::{
    memory::{manager::CooperativeMemoryManager, WasmSpan},
    StoreData, WasmPluginContext,
};
use crate::errors::Result;
use crate::state::ShellState;

pub trait Plugin: Send {
    /// Returns the name of the plugin - this is usually the file it was loaded from
    fn name(&self) -> &str;
    /// Run a function defined by the plugin.
    /// Accepts a pre-serialized argument list of JSON data, so as to not cause
    /// unneccesary serialization.
    fn call_hook(&mut self, name: &str, arguments: &[&[u8]]) -> Option<Result<()>>;
    /// Run a function defined by the plugin and capture its return value.
    fn call_hook_with_return(&mut self, name: &str, arguments: &[&[u8]]) -> Option<Result<Value>>;
}

pub struct WasmPluginBuilder<'a> {
    name: Option<String>,
    bytes: &'a [u8],
    wasi: bool,
    engine: &'a Engine,
    state: Option<Arc<RwLock<ShellState>>>,
}

impl<'a> WasmPluginBuilder<'a> {
    pub fn new(engine: &'a Engine, bytes: &'a [u8]) -> Self {
        Self {
            name: None,
            bytes,
            wasi: false,
            engine,
            state: None,
        }
    }

    pub fn unnamed(mut self) -> Self {
        self.name = Some("<unnamed plugin>".to_string());
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn wasi(mut self, wasi: bool) -> Self {
        self.wasi = wasi;
        self
    }

    pub fn state(mut self, state: Arc<RwLock<ShellState>>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn build(self) -> Result<WasmPlugin> {
        let name = self.name.context("Plugin name not set")?;

        let module = Module::new(self.engine, self.bytes)?;
        let mut linker = Linker::new(self.engine);

        let context = WasmPluginContext {
            wasi: self.wasi.then(|| {
                WasiCtxBuilder::new()
                    .inherit_stdio()
                    .inherit_env()
                    .unwrap()
                    .build()
            }),
            shell: self.state.context("Shell state not provided to plugin")?,
            memory: None,
        };
        if self.wasi {
            wasmtime_wasi::add_to_linker(&mut linker, |ctx: &mut StoreData| {
                ctx.wasi().as_mut().unwrap()
            })?;
        }
        let mut store = Store::<StoreData>::new(self.engine, Box::new(context));
        let instance = linker.instantiate(&mut store, &module)?;
        let memory_manager = CooperativeMemoryManager::new(&mut store, &instance)?;
        store
            .data_mut()
            .set_memory_manager(Arc::new(memory_manager));

        Ok(WasmPlugin {
            name,
            instance: linker.instantiate(&mut store, &module)?,
            store,
        })
    }
}

pub struct WasmPlugin {
    name: String,
    instance: Instance,
    store: Store<StoreData>,
}

impl WasmPlugin {
    fn buffers_to_wasm(&mut self, values: &[&[u8]]) -> Vec<Val> {
        values
            .iter()
            .map(|buffer| {
                self.store
                    .data()
                    .memory()
                    .copy(self.store.as_context_mut(), buffer)
                    .into_raw()
                    .to_wasm()
            })
            .collect()
    }
}

impl Plugin for WasmPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn call_hook(&mut self, name: &str, arguments: &[&[u8]]) -> Option<Result<()>> {
        let hook = self.instance.get_func(&mut self.store, name)?;

        if hook.ty(&self.store).results().len() != 0 {
            return Some(Err(anyhow!("Hook {name} may not return a value")));
        }

        let arg_pointers = self.buffers_to_wasm(arguments);

        if let Err(err) = hook.call(&mut self.store, &arg_pointers, &mut []) {
            return Some(Err(err));
        }

        Some(Ok(()))
    }

    fn call_hook_with_return(&mut self, name: &str, arguments: &[&[u8]]) -> Option<Result<Value>> {
        let hook = self.instance.get_func(&mut self.store, name)?;

        let hook_type = hook.ty(&self.store);
        if hook_type.results().len() != 1 || hook_type.results().next() != Some(ValType::I64) {
            return Some(Err(anyhow!("Hook {name} must return an i64")));
        }

        let arg_pointers = self.buffers_to_wasm(arguments);

        let mut return_values = [Val::null()];
        if let Err(err) = hook.call(&mut self.store, &arg_pointers, &mut return_values) {
            return Some(Err(err));
        }

        let memory_span = WasmSpan::try_from_wasm(&return_values[0]).unwrap();
        let buffer = self
            .store
            .data()
            .memory()
            .view(self.store.as_context_mut(), memory_span)
            .as_ref()
            .to_vec();
        Some(serde_json::from_slice(&buffer).context("Hook returned invalid json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ShellState;
    use wasmtime::Engine;

    #[test]
    fn can_create_plugin_and_call_hook() {
        let engine = Engine::default();
        let mut plugin = WasmPluginBuilder::new(
            &engine,
            r#"
            (module
                (func $test (param) (result))
                (export "test" (func $test))
                (memory (export "memory") 1)
                (func $mem_alloc (param i32) (result i32)
                    unreachable
                )
                (func $mem_dealloc (param i32)
                    unreachable
                )
                (export "mem_alloc" (func $mem_alloc))
                (export "mem_dealloc" (func $mem_dealloc))
            )
            "#
            .as_bytes(),
        )
        .unnamed()
        .wasi(true)
        .state(ShellState::new().unwrap())
        .build()
        .unwrap();

        let result = plugin.call_hook("test", &[]);
        assert!(matches!(result, Some(Ok(()))));
    }
}
