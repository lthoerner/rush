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
                self.load(&entry.path(), init_params)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use extism::CurrentPlugin;

    use super::*;

    struct TestHostBindings;
    impl HostBindings for TestHostBindings {
        fn output_text(&mut self, _plugin: &mut CurrentPlugin, text: String) {
            println!("{text}");
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
