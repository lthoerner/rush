use api::InitHookParams;
use extism::{Context, Function, Plugin, ValType};
use rush_plugins_api as api;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Snafu)]
pub enum RushPluginError {
    #[snafu(display("invalid plugin format ({name}): {source}"))]
    Extism { source: extism::Error, name: String },
    #[snafu(display("i/o error ({name}): {source}"))]
    Io {
        source: std::io::Error,
        name: String,
    },
    #[snafu(display("message serialization/deserialization failed ({name}): {source}"))]
    Serde {
        source: serde_json::Error,
        name: String,
    },
}

pub struct RushPlugin<'a> {
    instance: extism::Plugin<'a>,
    name: String,
}

impl<'a> RushPlugin<'a> {
    /// Load a plugin without reading from a file.
    pub fn from_bytes(
        bytes: impl AsRef<[u8]>,
        context: &'a Context,
        name: String,
    ) -> Result<Self, extism::Error> {
        Ok(RushPlugin {
            instance: Plugin::new(context, bytes, [], true)?,
            name,
        })
    }

    /// Read and load a plugin from a file.
    ///
    /// # Panics
    ///
    /// - Panics if the path does not contain a valid file name (ex: `/`, `/path/to/file/..`)
    pub fn new(path: &Path, context: &'a Context) -> Result<Self, RushPluginError> {
        let path_display = path.display().to_string();

        let bytes = fs::read(path).context(IoSnafu {
            name: &path_display,
        })?;
        Self::from_bytes(&bytes, context, path_display.clone()).context(ExtismSnafu {
            name: &path_display,
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Call an exported function from the plugin.
    pub fn call_hook<T>(&mut self, hook: &str, data: &impl Serialize) -> Result<T, RushPluginError>
    where
        T: DeserializeOwned,
    {
        let hook_input = serde_json::to_vec(data).context(SerdeSnafu { name: &self.name })?;
        let output_bytes = self
            .instance
            .call(hook, &hook_input)
            .context(ExtismSnafu { name: &self.name })?;

        serde_json::from_slice::<T>(output_bytes).context(SerdeSnafu { name: &self.name })
    }

    /// Call an exported function from the plugin, returning `None` if it is not implemented.
    pub fn call_hook_if_exists(
        &mut self,
        hook: &str,
        data: &impl Serialize,
    ) -> Result<Option<impl Deserialize>, extism::Error> {
        if self.instance.has_function(hook) {
            Ok(Some(self.call_hook(hook, data)?))
        } else {
            Ok(None)
        }
    }

    /// Perform any initialization required by the plugin implementation.
    pub fn init(&mut self, params: &InitHookParams) -> Result<(), extism::Error> {
        self.call_hook_if_exists("rush_plugin_init", params)?;
        Ok(())
    }
}

impl Debug for RushPlugin<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RushPlugin")
            .field("name", &self.name)
            .finish()
    }
}
