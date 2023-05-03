use extism_pdk::*;
use rush_plugins_api::InitHookParams;

#[plugin_fn]
pub fn rush_plugin_init(input: Json<InitHookParams>) -> FnResult<Json<()>> {
    error!("Hello rush v{}!", input.0.rush_version);

    set_var!("a", "this is var a")?;

    let a = var::get("a")?.expect("variable 'a' set");
    let _a = String::from_utf8(a).expect("string from varible value");

    Ok(Json(()))
}
