use extism::Context;
use rush_plugins_api::InitHookParams;
use snafu::ResultExt;
use std::path::Path;

pub mod plugin;

pub struct PluginRegistry<'a> {
    pub plugins: Vec<plugin::RushPlugin<'a>>,
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
    ) -> Result<(), plugin::RushPluginError> {
        let mut plugin = plugin::RushPlugin::new(path, self.context)?;
        plugin.init(init_params)?;
        self.plugins.push(plugin);
        Ok(())
    }

    pub fn load(
        &mut self,
        path: &Path,
        init_params: &InitHookParams,
    ) -> Result<(), plugin::RushPluginError> {
        if path.is_file() {
            self.load_file(path, init_params)?;
        } else {
            let path_display = path.display().to_string();
            let subitems = path.read_dir().context(plugin::IoSnafu {
                name: &path_display,
            })?;

            for entry in subitems {
                let entry = entry.context(plugin::IoSnafu {
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
    use super::*;

    #[test]
    fn load_example_plugin() {
        let plugin_ctx = Context::new();
        let mut registry = PluginRegistry::new(&plugin_ctx);

        registry
            .load(
                Path::new("./example/target/wasm32-wasi/release/example.wasm"),
                &InitHookParams {
                    rush_version: "v1".to_string(),
                },
            )
            .unwrap();
    }
}
