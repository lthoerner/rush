use crate::plugin::{HostBindings, NoOpHostBindings};
use anyhow::bail;
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

pub fn output_text(
    plugin: &mut CurrentPlugin,
    args: &[Val],
    _ret: &mut [Val],
    _user_data: UserData,
) -> Result<(), anyhow::Error> {
    let mut bindings = HOST_BINDINGS.lock().unwrap();
    if let Some(Some(arg)) = args.get(0).map(|p| p.i64()) {
        let mem = plugin.memory.at_offset(arg as usize).unwrap();
        let input = plugin.memory.get_str(mem).unwrap().to_owned();
        bindings.output_text(plugin, input)
    } else {
        bail!("Invalid bindings - expected output_text(i64) -> ()");
    }

    Ok(())
}
