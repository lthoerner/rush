use extism::Context;
use rush_plugins_api::InitHookParams;
use snafu::ResultExt;
use std::path::Path;

pub mod plugin;

#[derive(Default)]
pub struct PluginRegistry<'a> {
    pub plugins: Vec<plugin::RushPlugin<'a>>,
    pub context: Context,
}

impl<'a> PluginRegistry<'a> {
    pub fn load_file(
        &'a mut self,
        path: &Path,
        init_params: &InitHookParams,
    ) -> Result<(), plugin::RushPluginError> {
        let mut plugin = plugin::RushPlugin::new(path, &self.context)?;
        plugin.init(init_params)?;
        self.plugins.push(plugin);
        Ok(())
    }

    pub fn load(
        &'a mut self,
        path: &Path,
        init_params: &InitHookParams,
    ) -> Result<(), plugin::RushPluginError> {
        if path.is_file() {
            self.load_file(path, init_params)?;
        } else {
            let path_display = path.display().to_string();

            for entry in path.read_dir().context(plugin::IoSnafu {
                name: &path_display,
            })? {
                let entry = entry.context(plugin::IoSnafu {
                    name: &path_display,
                })?;
                let path = entry.path();
                self.load(&path, init_params)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //let result = add(2, 2);
        //assert_eq!(result, 4);
    }
}
