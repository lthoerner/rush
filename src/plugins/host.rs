use std::sync::{mpsc, Arc, RwLock};
use std::thread::{spawn, JoinHandle};

use serde_json::Value;
use wasmtime::{Engine, WasmBacktraceDetails};

use crate::state::shell::ShellState;

use super::loader::RecursivePluginLoader;
use super::plugin::Plugin;

/// Handles running plugins in a seperate thread so as to not slow
/// down the responsiveness of the main shell thread.
pub struct PluginHost {
    tunnel: mpsc::Sender<HookEvent>,
}

impl PluginHost {
    /// Load plugins defined in the config and start running them in the background.
    pub fn new(state: Arc<RwLock<ShellState>>) -> Self {
        let plugin_engine = Engine::new(
            wasmtime::Config::new()
                .debug_info(true)
                .wasm_backtrace_details(WasmBacktraceDetails::Enable),
        )
        .unwrap();

        let plugin_loader = RecursivePluginLoader::new(plugin_engine, state);
        let plugins = plugin_loader
            .filter_map(|result| {
                if let Err(err) = &result {
                    eprintln!("rush: failed to load plugin: {err}");
                }
                result.ok()
            })
            .collect::<Vec<_>>();

        let mut host = Self {
            tunnel: Self::spawn_runner(plugins),
        };
        host.start();
        host
    }

    /// Spawn a new plugin runner thread that listens for events on the returned hook.
    fn spawn_runner(mut plugins: Vec<Box<dyn Plugin>>) -> mpsc::Sender<HookEvent> {
        let (tx, rx) = mpsc::channel::<HookEvent>();

        spawn(move || loop {
            while let Ok(event) = rx.recv() {
                let serialized_args = event
                    .hook_params
                    .iter()
                    .map(|arg| serde_json::to_vec(arg).expect("Failed to serialize hook argument"))
                    .collect::<Vec<_>>();
                let arg_slices = serialized_args
                    .iter()
                    .map(|arg| arg.as_slice())
                    .collect::<Vec<_>>();

                let mut crashed_plugins = Vec::new();
                let mut return_values = Vec::new();

                for (index, plugin) in plugins.iter_mut().enumerate() {
                    match &event.callback {
                        HookBroadcastCallback::Value(_) => {
                            let return_value =
                                plugin.call_hook_with_return(&event.hook_name, &arg_slices);

                            match return_value {
                                Some(Ok(payload)) => {
                                    return_values.push(PluginHookResponse {
                                        payload,
                                        plugin_name: plugin.name().to_owned(),
                                    });
                                }
                                Some(Err(err)) => {
                                    crashed_plugins.push((index, err));
                                }
                                None => {}
                            }
                        }
                        HookBroadcastCallback::Empty(_) => {
                            let return_value = plugin.call_hook(&event.hook_name, &arg_slices);

                            if let Some(Err(err)) = return_value {
                                crashed_plugins.push((index, err));
                            }
                        }
                    }
                }

                // scan through crashed plugins from biggest index to smallest index
                // so that remove() works properly
                while let Some((index, crash)) = crashed_plugins.pop() {
                    let plugin = plugins.remove(index);
                    eprintln!("rush: plugin {} crashed: {crash}", plugin.name());
                }

                match event.callback {
                    HookBroadcastCallback::Value(tx) => {
                        _ = tx.send(return_values);
                    }
                    HookBroadcastCallback::Empty(tx) => {
                        _ = tx.send(());
                    }
                }
            }
        });

        tx
    }

    fn run_hook(&mut self, hook: &str, arguments: Vec<Value>) -> JoinHandle<()> {
        let (tx, rx) = oneshot::channel::<()>();
        self.tunnel
            .send(HookEvent {
                hook_name: hook.to_owned(),
                hook_params: arguments,
                callback: HookBroadcastCallback::Empty(tx),
            })
            .unwrap();
        spawn(move || rx.recv().unwrap())
    }

    fn run_hook_with_return(
        &mut self,
        hook: &str,
        arguments: Vec<Value>,
    ) -> JoinHandle<Vec<PluginHookResponse>> {
        let (tx, rx) = oneshot::channel::<Vec<PluginHookResponse>>();
        self.tunnel
            .send(HookEvent {
                hook_name: hook.to_owned(),
                hook_params: arguments,
                callback: HookBroadcastCallback::Value(tx),
            })
            .unwrap();
        spawn(move || rx.recv().unwrap())
    }

    fn start(&mut self) -> JoinHandle<()> {
        self.run_hook("start", vec![])
    }
}

struct HookEvent {
    hook_name: String,
    hook_params: Vec<Value>,
    callback: HookBroadcastCallback,
}
struct PluginHookResponse {
    payload: Value,
    plugin_name: String,
}

enum HookBroadcastCallback {
    /// The hook is not expected to return anything
    Empty(oneshot::Sender<()>),
    /// A return value is expected from the hook
    Value(oneshot::Sender<Vec<PluginHookResponse>>),
}
