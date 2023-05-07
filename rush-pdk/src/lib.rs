pub mod raw;

use concat_idents::concat_idents;
use std::collections::HashMap;

pub use extism_pdk::{self, *};
pub use rush_plugins_api::{self, *};

macro_rules! __binding_internal {
    // stage 3; generate function
    ($(#[$attr:meta])* ($($prefix:ident)?) $name:ident ($($arg:ident : $ty:ty),*) -> (box) $ret:ty; $($t:tt)*) => {
        $(#[$attr])*
        pub fn $name($($arg: $ty),*) -> $ret {
            $(
                let $arg = ::extism_pdk::Memory::from_bytes($arg);
            )*
            let ret_value = unsafe {
                ::concat_idents::concat_idents!(raw_name = $($prefix, _,)? $name {
                    crate::raw::raw_name($($arg.offset),*)
                })
            };
            <$ret as ::extism_pdk::FromBytes>::from_bytes(
                    ::extism_pdk::Memory::find(ret_value).unwrap().to_vec()
                ).unwrap()
        }

        bindings!($($t)*);
    };
    ($(#[$attr:meta])* ($($prefix:ident)?) $name:ident ($($arg:ident : $ty:ty),*) -> () $ret:ty; $($t:tt)*) => {
        $(#[$attr])*
        pub fn $name($($arg: $ty),*) -> $ret {
            $(
                let $arg = ::extism_pdk::Memory::from_bytes($arg);
            )*
            unsafe {
                ::concat_idents::concat_idents!(raw_name = $($prefix, _,)? $name {
                    crate::raw::raw_name($($arg.offset),*)
                })
            }
        }

        bindings!($($t)*);
    };
}

macro_rules! __binding_solve_prefix {
    // stage 2; seperate prefix from generated function name
    // This allows functions to be put in seperate modules (e.g. `env`)
    // without having repetative names (e.g. `env::env_set` becomes `env::set`).
    // The syntax for this: `prefix::name` looks for the raw function `prefix_name`
    // and creates `name`.
    ($(#[$attr:meta])* $prefix:ident :: $name:ident $($t:tt)*) => {
        __binding_internal!($(#[$attr])* ($prefix) $name $($t)*);
    };
    ($(#[$attr:meta])* $name:ident $($t:tt)*) => {
        __binding_internal!($(#[$attr])* () $name $($t)*);
    };
}

/// Add type safety to a raw binding.
///
/// The host application can provide "bindings" to the plugin, which are functions that the plugin can call.
/// Only numbers can be passed to or returned from bindings, but this does give the opportunity to pass
/// memory pointers for the host to dereference. This macro creates a translation layer to handle allocating,
/// sending the pointer, and then deallocating the memory and provides type safety so that plugin developers can
/// only send the correct pointers.
macro_rules! bindings {
    // stage 1; detect if the binding return value needs to be converted from a "box" (pointer) type
    // Box return types are u64s that contain a memory address to the actual return value.
    // This is neccesary to communicate anything other than numbers.
    ($(#[$attr:meta])* $name:ident $(:: $name2:ident)? ($($arg:ident : $ty:ty),*) -> box $ret:ty; $($t:tt)*) => {
        __binding_solve_prefix!($(#[$attr])* $name $(:: $name2)* ($($arg: $ty),*) -> (box) $ret; $($t)*);
    };
    ($(#[$attr:meta])* $name:ident $(:: $name2:ident)? ($($arg:ident : $ty:ty),*) -> $ret:ty; $($t:tt)*) => {
        __binding_solve_prefix!($(#[$attr])* $name $(:: $name2)* ($($arg: $ty),*) -> () $ret; $($t)*);
    };
    () => ()
}

bindings! {
    output_text(text: &str) -> ();
}

/// Send and revieve the host's environment variables.
///
/// Each plugin has its own, isolated environment variables, but it may be neccesary to access the application's
/// (e.g. to get the PATH variable). This module contains function for sending and retrieving the
/// application's environment variables.
pub mod env {
    use super::*;

    bindings! {
        /// Get a single environment variable from the application
        env::get(key: &str) -> box Json<Option<String>>;
        /// Set a single environment variable for the application
        env::set(key: &str, value: &str) -> ();
        /// Delete a single environment variable from the application
        env::delete(key: &str) -> ();
        /// Get a map of all environment variables from the application
        env::vars() -> box Json<HashMap<String, String>>;
    }

    /// Replace the plugin's environment variables with the application's.
    ///
    /// ```rs
    /// rusk_pdk::env::load_host_vars();
    /// std::env::var("PATH").unwrap();
    /// ```
    pub fn load_host_vars() {
        let host_env = self::vars().0;
        let old_env_vars = std::env::vars().map(|(name, _)| name);

        // remove environment variables that won't be replaced by the host
        for var in old_env_vars {
            if !host_env.contains_key(&var) {
                std::env::remove_var(var);
            }
        }

        for (key, value) in host_env {
            std::env::set_var(key, value);
        }
    }

    /// Send all of the plugin's environment variables to the application's environment.
    pub fn send_host_vars() {
        for (key, value) in std::env::vars() {
            self::set(&key, &value);
        }
    }
}
