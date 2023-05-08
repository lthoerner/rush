mod bindings;
pub mod plugin;

use extism::Context;
use plugin::{HostBindings, IoSnafu, RushPlugin, RushPluginError};
use rush_plugins_api::InitHookParams;
use snafu::ResultExt;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

pub use bindings::HOST_BINDINGS;

pub struct PluginRegistry<'a> {
    pub plugins: Vec<RushPlugin<'a>>,
    pub context: &'a Context,
    pub report_error: Box<dyn FnMut(&str, String)>,
}

impl<'a> PluginRegistry<'a> {
    pub fn new(plugin_context: &'a Context, report_error: Box<dyn FnMut(&str, String)>) -> Self {
        Self {
            context: plugin_context,
            plugins: Vec::new(),
            report_error,
        }
    }

    /// Load a plugin from a file.
    pub fn load_file(
        &mut self,
        path: &Path,
        init_params: &InitHookParams,
    ) -> Result<(), RushPluginError> {
        let mut plugin = RushPlugin::new(path, self.context)?;
        plugin.init(init_params)?;
        self.plugins.push(plugin);
        Ok(())
    }

    /// Load all plugins from a file or directory.
    pub fn load(
        &mut self,
        path: &Path,
        init_params: &InitHookParams,
    ) -> Result<(), RushPluginError> {
        if path.is_file() {
            self.load_file(path, init_params)?;
        } else {
            let path_display = path.display().to_string();
            let subitems = path.read_dir().context(IoSnafu {
                name: &path_display,
            })?;

            for entry in subitems {
                let entry = entry.context(IoSnafu {
                    name: &path_display,
                })?;

                // if it's a file not ending in .wasm (i.e. not a plugin), skip it
                if !entry.file_name().to_string_lossy().ends_with(".wasm")
                    && entry
                        .file_type()
                        .context(IoSnafu {
                            name: &path_display,
                        })?
                        .is_file()
                {
                    continue;
                }

                self.load(&entry.path(), init_params)?;
            }
        }

        Ok(())
    }

    /// Perform any deinitialization required by the plugin implementations, removing them from the registry.
    pub fn deinit_plugins(&mut self) {
        for mut plugin in self.plugins.drain(..) {
            _ = plugin.deinit();
        }
    }

    // TODO: might be good to add a "priority" system where plugins can specify how good their completions are
    pub fn request_autocomplete(&mut self, line_buffer: &str) -> Option<String> {
        // note: this always runs the hook on every plugin even if one of them gives a completion early
        // not sure if that's good or bad since the plugin could use this for side effects
        self.plugins
            .iter_mut()
            .map(|plugin| plugin.request_autocomplete(line_buffer))
            // collect()ing is neccesary to drop the borrow of self.plugin, allowing us to run the remove() method
            .collect::<Vec<_>>()
            .into_iter()
            .enumerate()
            .filter_map(|(index, result)| match result {
                Ok(completion) => completion,
                Err(err) => {
                    (self.report_error)(self.plugins[index].name(), err.to_string());
                    self.plugins.remove(index);
                    None
                }
            })
            // collect to make sure all errored plugins are disabled
            .collect::<Vec<_>>()
            .into_iter()
            .next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use extism::CurrentPlugin;
    use std::collections::HashMap;

    struct TestHostBindings {
        env_vars: HashMap<String, String>,
    }

    impl HostBindings for TestHostBindings {
        fn output_text(&mut self, _plugin: &mut CurrentPlugin, text: String) {
            println!("{text}");
        }
        fn env_vars(&mut self, _plugin: &mut CurrentPlugin) -> HashMap<String, String> {
            self.env_vars.clone()
        }
        fn env_get(&mut self, _plugin: &mut CurrentPlugin, var_name: String) -> Option<String> {
            self.env_vars.get(&var_name).cloned()
        }
        fn fs_is_executable(&mut self, _plugin: &mut CurrentPlugin, path: String) -> bool {
            true
        }
    }

    #[ctor::ctor]
    fn set_test_bindings() {
        *bindings::HOST_BINDINGS.lock().unwrap() = Box::new(TestHostBindings {
            env_vars: HashMap::from([
                ("PATH".to_string(), "/usr/bin:/bin".to_string()),
                ("RUSH_TEST".to_string(), "1".to_string()),
            ]),
        });
    }

    #[test]
    fn load_example_plugin() {
        let plugin_ctx = Context::new();
        let mut registry = PluginRegistry::new(
            &plugin_ctx,
            Box::new(|plugin, err| {
                panic!("Error in plugin {}: {}", plugin, err);
            }),
        );

        registry
            .load(
                Path::new("../example-plugins/welcome-message/target/wasm32-wasi/release/welcome_message.wasm"),
                &InitHookParams {
                    rush_version: "v1".to_string(),
                },
            )
            .unwrap();
    }

    #[test]
    fn autocomplete() {
        let plugin_ctx = Context::new();
        let mut registry = PluginRegistry::new(
            &plugin_ctx,
            Box::new(|plugin, err| {
                panic!("Error in plugin {}: {}", plugin, err);
            }),
        );

        registry
            .load(
                Path::new("../example-plugins/path-autocomplete/target/wasm32-wasi/release/path_autocomplete.wasm"),
                &InitHookParams {
                    rush_version: "v1".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            registry.request_autocomplete("realp").as_deref(),
            Some("ath")
        );
    }
}
