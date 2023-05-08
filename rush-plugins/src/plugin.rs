use crate::bindings::{
    self, ENV_DELETE_FN, ENV_GET_FN, ENV_SET_FN, ENV_VARS_FN, FS_IS_EXECUTABLE_FN, OUTPUT_TEXT_FN,
};
use api::InitHookParams;
use extism::{
    manifest::Wasm, Context, CurrentPlugin, Function, Manifest, Plugin, UserData, Val, ValType,
};
use rush_plugins_api as api;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::{
    collections::HashMap,
    fmt::Debug,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

/// Implementations of functions that plugins can use.
#[allow(unused_variables)]
pub trait HostBindings: Send {
    fn output_text(&mut self, plugin: &mut CurrentPlugin, input: String) {}

    // Functions for modifying the host's environment variables.
    // Each plugin has its own, isolated set so these can be used to sync between the host and plugin.

    fn env_get(&mut self, plugin: &mut CurrentPlugin, var_name: String) -> Option<String> {
        None
    }
    fn env_set(&mut self, plugin: &mut CurrentPlugin, var_name: String, var_value: String) {}
    fn env_delete(&mut self, plugin: &mut CurrentPlugin, var_name: String) {}
    /// Get all environment variables as a JSON object.
    fn env_vars(&mut self, plugin: &mut CurrentPlugin) -> HashMap<String, String> {
        HashMap::new()
    }

    fn fs_is_executable(&mut self, plugin: &mut CurrentPlugin, path: String) -> bool {
        false
    }
}

/// A struct implementing [`HostBindings`] with only no-op methods.
pub struct NoOpHostBindings;
impl HostBindings for NoOpHostBindings {}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
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
        bytes: impl Into<Vec<u8>>,
        context: &'a Context,
        name: String,
    ) -> Result<Self, extism::Error> {
        let manifest = Manifest::new([Wasm::data(bytes)]).with_allowed_path("/", "/");
        Ok(RushPlugin {
            instance: Plugin::new_with_manifest(
                context,
                &manifest,
                [
                    &*OUTPUT_TEXT_FN,
                    &*ENV_GET_FN,
                    &*ENV_SET_FN,
                    &*ENV_DELETE_FN,
                    &*ENV_VARS_FN,
                    &*FS_IS_EXECUTABLE_FN,
                ],
                true,
            )?,
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
        Self::from_bytes(bytes, context, path_display.clone()).context(ExtismSnafu {
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
    pub fn call_hook_if_exists<T>(
        &mut self,
        hook: &str,
        data: &impl Serialize,
    ) -> Result<Option<T>, RushPluginError>
    where
        T: DeserializeOwned,
    {
        if self.instance.has_function(hook) {
            Ok(Some(self.call_hook::<T>(hook, data)?))
        } else {
            Ok(None)
        }
    }

    // Following methods are a comprehensive list of plugin hooks.

    /// Perform any initialization required by the plugin implementation.
    pub fn init(&mut self, params: &InitHookParams) -> Result<(), RushPluginError> {
        self.call_hook_if_exists::<()>("rush_plugin_init", params)?;
        Ok(())
    }

    /// Perform any deinitialization required by the plugin implementation.
    pub fn deinit(&mut self) -> Result<(), RushPluginError> {
        self.call_hook_if_exists::<()>("rush_plugin_deinit", &())?;
        Ok(())
    }

    /// Ask the plugin for completion suggestions.
    pub fn request_autocomplete(
        &mut self,
        line_buffer: &str,
    ) -> Result<Option<String>, RushPluginError> {
        let suggestion: Option<String> = self
            .call_hook_if_exists("provide_autocomplete", &line_buffer)?
            .flatten();
        Ok(suggestion)
    }
}

impl Debug for RushPlugin<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RushPlugin")
            .field("name", &self.name)
            .finish()
    }
}
