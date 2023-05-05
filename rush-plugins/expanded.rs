#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod bindings {
    use crate::plugin::{HostBindings, NoOpHostBindings};
    use anyhow::{bail, Context};
    use extism::{CurrentPlugin, Function, UserData, Val, ValType};
    use lazy_static::lazy_static;
    use std::sync::Mutex;
    #[allow(missing_copy_implementations)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    /// Global bindings that plugins can call to interact with the shell
    pub struct HOST_BINDINGS {
        __private_field: (),
    }
    #[doc(hidden)]
    pub static HOST_BINDINGS: HOST_BINDINGS = HOST_BINDINGS {
        __private_field: (),
    };
    impl ::lazy_static::__Deref for HOST_BINDINGS {
        type Target = Mutex<Box<dyn HostBindings>>;
        fn deref(&self) -> &Mutex<Box<dyn HostBindings>> {
            #[inline(always)]
            fn __static_ref_initialize() -> Mutex<Box<dyn HostBindings>> {
                Mutex::new(Box::new(NoOpHostBindings))
            }
            #[inline(always)]
            fn __stability() -> &'static Mutex<Box<dyn HostBindings>> {
                static LAZY: ::lazy_static::lazy::Lazy<Mutex<Box<dyn HostBindings>>> = ::lazy_static::lazy::Lazy::INIT;
                LAZY.get(__static_ref_initialize)
            }
            __stability()
        }
    }
    impl ::lazy_static::LazyStatic for HOST_BINDINGS {
        fn initialize(lazy: &Self) {
            let _ = &**lazy;
        }
    }
    #[allow(missing_copy_implementations)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    pub struct OUTPUT_TEXT_FN {
        __private_field: (),
    }
    #[doc(hidden)]
    pub static OUTPUT_TEXT_FN: OUTPUT_TEXT_FN = OUTPUT_TEXT_FN {
        __private_field: (),
    };
    impl ::lazy_static::__Deref for OUTPUT_TEXT_FN {
        type Target = Function;
        fn deref(&self) -> &Function {
            #[inline(always)]
            fn __static_ref_initialize() -> Function {
                Function::new("output_text", [ValType::I64], [], None, output_text)
            }
            #[inline(always)]
            fn __stability() -> &'static Function {
                static LAZY: ::lazy_static::lazy::Lazy<Function> = ::lazy_static::lazy::Lazy::INIT;
                LAZY.get(__static_ref_initialize)
            }
            __stability()
        }
    }
    impl ::lazy_static::LazyStatic for OUTPUT_TEXT_FN {
        fn initialize(lazy: &Self) {
            let _ = &**lazy;
        }
    }
    pub fn output_text(
        plugin: &mut CurrentPlugin,
        args: &[Val],
        _ret: &mut [Val],
        _user_data: UserData,
    ) -> Result<(), anyhow::Error> {
        let mut bindings = HOST_BINDINGS.lock().unwrap();
        let arg = args.get(0).and_then(|p| p.i64()).context("Invalid argument type")?;
        let input = {
            let mem = plugin
                .memory
                .at_offset(arg as usize)
                .context("Invalid memory offset")?;
            plugin.memory.get_str(mem).context("Invalid string in memory")?.to_owned()
        };
        Ok(())
    }
}
pub mod plugin {
    use crate::bindings::{self, OUTPUT_TEXT_FN};
    use api::InitHookParams;
    use extism::{Context, CurrentPlugin, Function, Plugin, UserData, Val, ValType};
    use rush_plugins_api as api;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use snafu::{ResultExt, Snafu};
    use std::{fmt::Debug, fs, path::Path, sync::{Arc, Mutex}};
    /// Implementations of functions that plugins can use.
    pub trait HostBindings: Send {
        fn output_text(&mut self, plugin: &mut CurrentPlugin, input: String) {}
        /// Automatically called when a plugin misuses the bindings
        fn emit_warning(&mut self, plugin_name: &str, warning: &str) {}
    }
    /// A struct implementing [`HostBindings`] with only no-op methods.
    pub struct NoOpHostBindings;
    impl HostBindings for NoOpHostBindings {}
    #[snafu(visibility(pub(crate)))]
    pub enum RushPluginError {
        #[snafu(display("invalid plugin format ({name}): {source}"))]
        Extism { source: extism::Error, name: String },
        #[snafu(display("i/o error ({name}): {source}"))]
        Io { source: std::io::Error, name: String },
        #[snafu(
            display("message serialization/deserialization failed ({name}): {source}")
        )]
        Serde { source: serde_json::Error, name: String },
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for RushPluginError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                RushPluginError::Extism { source: __self_0, name: __self_1 } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "Extism",
                        "source",
                        __self_0,
                        "name",
                        &__self_1,
                    )
                }
                RushPluginError::Io { source: __self_0, name: __self_1 } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "Io",
                        "source",
                        __self_0,
                        "name",
                        &__self_1,
                    )
                }
                RushPluginError::Serde { source: __self_0, name: __self_1 } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "Serde",
                        "source",
                        __self_0,
                        "name",
                        &__self_1,
                    )
                }
            }
        }
    }
    ///SNAFU context selector for the `RushPluginError::Extism` variant
    pub(crate) struct ExtismSnafu<__T0> {
        #[allow(missing_docs)]
        pub(crate) name: __T0,
    }
    #[automatically_derived]
    impl<__T0: ::core::fmt::Debug> ::core::fmt::Debug for ExtismSnafu<__T0> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "ExtismSnafu",
                "name",
                &&self.name,
            )
        }
    }
    #[automatically_derived]
    impl<__T0: ::core::marker::Copy> ::core::marker::Copy for ExtismSnafu<__T0> {}
    #[automatically_derived]
    impl<__T0: ::core::clone::Clone> ::core::clone::Clone for ExtismSnafu<__T0> {
        #[inline]
        fn clone(&self) -> ExtismSnafu<__T0> {
            ExtismSnafu {
                name: ::core::clone::Clone::clone(&self.name),
            }
        }
    }
    impl<__T0> ::snafu::IntoError<RushPluginError> for ExtismSnafu<__T0>
    where
        RushPluginError: ::snafu::Error + ::snafu::ErrorCompat,
        __T0: ::core::convert::Into<String>,
    {
        type Source = extism::Error;
        #[track_caller]
        fn into_error(self, error: Self::Source) -> RushPluginError {
            let error: extism::Error = (|v| v)(error);
            RushPluginError::Extism {
                source: error,
                name: ::core::convert::Into::into(self.name),
            }
        }
    }
    ///SNAFU context selector for the `RushPluginError::Io` variant
    pub(crate) struct IoSnafu<__T0> {
        #[allow(missing_docs)]
        pub(crate) name: __T0,
    }
    #[automatically_derived]
    impl<__T0: ::core::fmt::Debug> ::core::fmt::Debug for IoSnafu<__T0> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "IoSnafu",
                "name",
                &&self.name,
            )
        }
    }
    #[automatically_derived]
    impl<__T0: ::core::marker::Copy> ::core::marker::Copy for IoSnafu<__T0> {}
    #[automatically_derived]
    impl<__T0: ::core::clone::Clone> ::core::clone::Clone for IoSnafu<__T0> {
        #[inline]
        fn clone(&self) -> IoSnafu<__T0> {
            IoSnafu {
                name: ::core::clone::Clone::clone(&self.name),
            }
        }
    }
    impl<__T0> ::snafu::IntoError<RushPluginError> for IoSnafu<__T0>
    where
        RushPluginError: ::snafu::Error + ::snafu::ErrorCompat,
        __T0: ::core::convert::Into<String>,
    {
        type Source = std::io::Error;
        #[track_caller]
        fn into_error(self, error: Self::Source) -> RushPluginError {
            let error: std::io::Error = (|v| v)(error);
            RushPluginError::Io {
                source: error,
                name: ::core::convert::Into::into(self.name),
            }
        }
    }
    ///SNAFU context selector for the `RushPluginError::Serde` variant
    pub(crate) struct SerdeSnafu<__T0> {
        #[allow(missing_docs)]
        pub(crate) name: __T0,
    }
    #[automatically_derived]
    impl<__T0: ::core::fmt::Debug> ::core::fmt::Debug for SerdeSnafu<__T0> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "SerdeSnafu",
                "name",
                &&self.name,
            )
        }
    }
    #[automatically_derived]
    impl<__T0: ::core::marker::Copy> ::core::marker::Copy for SerdeSnafu<__T0> {}
    #[automatically_derived]
    impl<__T0: ::core::clone::Clone> ::core::clone::Clone for SerdeSnafu<__T0> {
        #[inline]
        fn clone(&self) -> SerdeSnafu<__T0> {
            SerdeSnafu {
                name: ::core::clone::Clone::clone(&self.name),
            }
        }
    }
    impl<__T0> ::snafu::IntoError<RushPluginError> for SerdeSnafu<__T0>
    where
        RushPluginError: ::snafu::Error + ::snafu::ErrorCompat,
        __T0: ::core::convert::Into<String>,
    {
        type Source = serde_json::Error;
        #[track_caller]
        fn into_error(self, error: Self::Source) -> RushPluginError {
            let error: serde_json::Error = (|v| v)(error);
            RushPluginError::Serde {
                source: error,
                name: ::core::convert::Into::into(self.name),
            }
        }
    }
    #[allow(single_use_lifetimes)]
    impl ::core::fmt::Display for RushPluginError {
        fn fmt(
            &self,
            __snafu_display_formatter: &mut ::core::fmt::Formatter,
        ) -> ::core::fmt::Result {
            #[allow(unused_variables)]
            match *self {
                RushPluginError::Extism { ref name, ref source } => {
                    __snafu_display_formatter
                        .write_fmt(
                            format_args!(
                                "invalid plugin format ({0}): {1}", name, source
                            ),
                        )
                }
                RushPluginError::Io { ref name, ref source } => {
                    __snafu_display_formatter
                        .write_fmt(format_args!("i/o error ({0}): {1}", name, source))
                }
                RushPluginError::Serde { ref name, ref source } => {
                    __snafu_display_formatter
                        .write_fmt(
                            format_args!(
                                "message serialization/deserialization failed ({0}): {1}",
                                name, source
                            ),
                        )
                }
            }
        }
    }
    #[allow(single_use_lifetimes)]
    impl ::snafu::Error for RushPluginError
    where
        Self: ::core::fmt::Debug + ::core::fmt::Display,
    {
        fn description(&self) -> &str {
            match *self {
                RushPluginError::Extism { .. } => "RushPluginError :: Extism",
                RushPluginError::Io { .. } => "RushPluginError :: Io",
                RushPluginError::Serde { .. } => "RushPluginError :: Serde",
            }
        }
        fn cause(&self) -> ::core::option::Option<&dyn ::snafu::Error> {
            use ::snafu::AsErrorSource;
            match *self {
                RushPluginError::Extism { ref source, .. } => {
                    ::core::option::Option::Some(source.as_error_source())
                }
                RushPluginError::Io { ref source, .. } => {
                    ::core::option::Option::Some(source.as_error_source())
                }
                RushPluginError::Serde { ref source, .. } => {
                    ::core::option::Option::Some(source.as_error_source())
                }
            }
        }
        fn source(&self) -> ::core::option::Option<&(dyn ::snafu::Error + 'static)> {
            use ::snafu::AsErrorSource;
            match *self {
                RushPluginError::Extism { ref source, .. } => {
                    ::core::option::Option::Some(source.as_error_source())
                }
                RushPluginError::Io { ref source, .. } => {
                    ::core::option::Option::Some(source.as_error_source())
                }
                RushPluginError::Serde { ref source, .. } => {
                    ::core::option::Option::Some(source.as_error_source())
                }
            }
        }
    }
    #[allow(single_use_lifetimes)]
    impl ::snafu::ErrorCompat for RushPluginError {
        fn backtrace(&self) -> ::core::option::Option<&::snafu::Backtrace> {
            match *self {
                RushPluginError::Extism { .. } => ::core::option::Option::None,
                RushPluginError::Io { .. } => ::core::option::Option::None,
                RushPluginError::Serde { .. } => ::core::option::Option::None,
            }
        }
    }
    pub struct RushPlugin<'a> {
        instance: extism::Plugin<'a>,
        host_functions: Vec<Function>,
        name: String,
    }
    impl<'a> RushPlugin<'a> {
        /// Load a plugin without reading from a file.
        pub fn from_bytes(
            bytes: impl AsRef<[u8]>,
            context: &'a Context,
            name: String,
        ) -> Result<Self, extism::Error> {
            let output_text_fn = Function::new(
                "output_text",
                [ValType::I64],
                [],
                None,
                bindings::output_text,
            );
            Ok(RushPlugin {
                instance: Plugin::new(context, bytes, [&*OUTPUT_TEXT_FN], true)?,
                host_functions: <[_]>::into_vec(
                    #[rustc_box]
                    ::alloc::boxed::Box::new([output_text_fn]),
                ),
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
            let bytes = fs::read(path).context(IoSnafu { name: &path_display })?;
            Self::from_bytes(&bytes, context, path_display.clone())
                .context(ExtismSnafu { name: &path_display })
        }
        pub fn name(&self) -> &str {
            self.name.as_ref()
        }
        /// Call an exported function from the plugin.
        pub fn call_hook<T>(
            &mut self,
            hook: &str,
            data: &impl Serialize,
        ) -> Result<T, RushPluginError>
        where
            T: DeserializeOwned,
        {
            let hook_input = serde_json::to_vec(data)
                .context(SerdeSnafu { name: &self.name })?;
            let output_bytes = self
                .instance
                .call(hook, &hook_input)
                .context(ExtismSnafu { name: &self.name })?;
            serde_json::from_slice::<T>(output_bytes)
                .context(SerdeSnafu { name: &self.name })
        }
        /// Call an exported function from the plugin, returning `None` if it is not implemented.
        pub fn call_hook_if_exists(
            &mut self,
            hook: &str,
            data: &impl Serialize,
        ) -> Result<Option<impl Deserialize>, RushPluginError> {
            if self.instance.has_function(hook) {
                Ok(Some(self.call_hook(hook, data)?))
            } else {
                Ok(None)
            }
        }
        /// Perform any initialization required by the plugin implementation.
        pub fn init(&mut self, params: &InitHookParams) -> Result<(), RushPluginError> {
            self.call_hook_if_exists("rush_plugin_init", params)?;
            Ok(())
        }
    }
    impl Debug for RushPlugin<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("RushPlugin").field("name", &self.name).finish()
        }
    }
}
use extism::Context;
use plugin::{HostBindings, IoSnafu, RushPlugin, RushPluginError};
use rush_plugins_api::InitHookParams;
use snafu::ResultExt;
use std::{path::Path, sync::{Arc, Mutex}};
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
            let subitems = path.read_dir().context(IoSnafu { name: &path_display })?;
            for entry in subitems {
                let entry = entry.context(IoSnafu { name: &path_display })?;
                self.load(&entry.path(), init_params)?;
            }
        }
        Ok(())
    }
}
