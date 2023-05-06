#![feature(trace_macros)]

pub mod raw;

pub use extism_pdk::*;
pub use rush_plugins_api::*;

macro_rules! __binding_internal {
 ($name:ident ($($arg:ident : $ty:ty),*) -> (ptr) $ret:ty; $($t:tt)*) => {
        pub fn $name($($arg: $ty),*) -> $ret {
            $(
                let $arg = ::extism_pdk::Memory::from_bytes($arg);
            )*
            let ret_value = unsafe {
                crate::raw::$name($($arg.offset),*)
            };
            <$ret as ::extism_pdk::FromBytes>::from_bytes(
                    Memory::find(ret_value).unwrap().to_vec()
                ).unwrap()
        }

        binding!($($t)*);
    };
    ($name:ident ($($arg:ident : $ty:ty),*) -> () $ret:ty; $($t:tt)*) => {
        pub fn $name($($arg: $ty),*) -> $ret {
            $(
                let $arg = ::extism_pdk::Memory::from_bytes($arg);
            )*
            unsafe {
                crate::raw::$name($($arg.offset),*)
            }
        }

        binding!($($t)*);
    };
}

macro_rules! binding {
    ($name:ident ($($arg:ident : $ty:ty),*) -> ptr $ret:ty; $($t:tt)*) => {
        __binding_internal!($name($($arg: $ty),*) -> (ptr) $ret; $($t)*);
    };
    ($name:ident ($($arg:ident : $ty:ty),*) -> $ret:ty; $($t:tt)*) => {
        __binding_internal!($name($($arg: $ty),*) -> () $ret; $($t)*);
    };
    () => ()
}

binding! {
    output_text(text: String) -> ();
    // modify_text(text: String) -> ptr String;
}
