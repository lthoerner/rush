//! Low-level bindings to functions provided by the Rush shell.
//!
//! These are used internally by the Rush PDK and use pointers to
//! shared memory as arguments instead of developer-friendly Rust types.

extern "C" {
    pub fn output_text(text: u64);
}
