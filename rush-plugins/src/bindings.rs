use crate::plugin::{HostBindings, NoOpHostBindings};
use anyhow::{bail, Context};
use extism::{CurrentPlugin, Function, UserData, Val, ValType};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    /// Global bindings that plugins can call to interact with the shell
    pub static ref HOST_BINDINGS: Mutex<Box<dyn HostBindings>> =
        Mutex::new(Box::new(NoOpHostBindings));

    pub static ref OUTPUT_TEXT_FN: Function = Function::new(
        "output_text",
        [ValType::I64],
        [],
        None,
        output_text,
    );
}

macro_rules! load_string {
    ($plugin:expr, $offset:expr) => {{
        let mem = $plugin
            .memory
            .at_offset($offset as usize)
            .context("Invalid memory offset")?;
        $plugin
            .memory
            .get_str(mem)
            .context("Invalid string in memory")?
            .to_owned()
    }};
}

#[rustfmt::skip]
macro_rules! get_arg {
    ($args:expr, $index:expr, $ty:ident) => {{
        let Some(arg): Option<$ty> = $args
            .get($index)
            .and_then(|p| p.$ty()) else {
                bail!("Expected a `{}` at argument {}", stringify!($ty), $index);
            };
        arg
    }};
}

pub fn output_text(
    plugin: &mut CurrentPlugin,
    args: &[Val],
    _ret: &mut [Val],
    _user_data: UserData,
) -> Result<(), anyhow::Error> {
    let arg = get_arg!(args, 0, i64);
    let input = load_string!(plugin, arg);

    let mut bindings = HOST_BINDINGS.lock().unwrap();
    bindings.output_text(plugin, input);

    Ok(())
}
