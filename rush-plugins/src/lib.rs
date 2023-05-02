use std::path::Path;

use extism::Context;

pub mod plugin;

#[derive(Default)]
pub struct PluginRegistry<'a> {
    pub plugins: Vec<plugin::RushPlugin<'a>>,
    pub context: Context,
}

impl<'a> PluginRegistry<'a> {
    pub fn load(&mut self, path: &Path) -> Result<(), plugin::RushPluginError> {
        let plugin = plugin::RushPlugin::<'a>::new(path, &self.context)?;
        self.plugins.push(plugin);
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
