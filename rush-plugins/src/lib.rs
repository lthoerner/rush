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
}

impl<'a> PluginRegistry<'a> {
    pub fn new(plugin_context: &'a Context) -> Self {
        Self {
            context: plugin_context,
            plugins: Vec::new(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use extism::CurrentPlugin;
    use std::collections::HashMap;

    struct TestHostBindings;
    impl HostBindings for TestHostBindings {
        fn output_text(&mut self, _plugin: &mut CurrentPlugin, text: String) {
            println!("{text}");
        }
        fn env_vars(&mut self, plugin: &mut CurrentPlugin) -> HashMap<String, String> {
            HashMap::from([
                ("PATH".to_string(), "/usr/bin:/bin".to_string()),
                ("RUSH_TEST".to_string(), "1".to_string()),
            ])
        }
    }

    #[ctor::ctor]
    fn set_test_bindings() {
        *bindings::HOST_BINDINGS.lock().unwrap() = Box::new(TestHostBindings);
    }

    #[test]
    fn load_example_plugin() {
        let plugin_ctx = Context::new();
        let mut registry = PluginRegistry::new(&plugin_ctx);

        registry
            .load(
                Path::new("../example-plugins/welcome-message/target/wasm32-wasi/release/welcome_message.wasm"),
                &InitHookParams {
                    rush_version: "v1".to_string(),
                },
            )
            .unwrap();
    }
}
