use extism_pdk::*;
use rush_plugins_api::InitHookParams;

extern "C" {
    fn output_text(text_ptr: u64);
}

#[plugin_fn]
pub fn rush_plugin_init(input: Json<InitHookParams>) -> FnResult<Json<()>> {
    // create a string to give to Rush
    let text = format!("Hello Rush {}", input.0.rush_version);
    // move string to public memory
    let text_ptr = Memory::from_bytes(text);
    // send location of memory to shell
    // safety: we just created this memory address
    unsafe {
        output_text(text_ptr.offset);
    }

    Ok(Json(()))
}
