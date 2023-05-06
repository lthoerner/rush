use rush_pdk::*;

#[plugin_fn]
pub fn rush_plugin_init(input: Json<InitHookParams>) -> FnResult<Json<()>> {
    // create a string to give to Rush
    let text = format!("Hello Rush {}", input.0.rush_version);
    output_text(text);

    Ok(Json(()))
}
