use rush_pdk::*;

// rush_plugin_init runs on startup
#[plugin_fn]
pub fn rush_plugin_init(input: Json<InitHookParams>) -> FnResult<Json<()>> {
    // print welcome message to console
    let text = format!("Hello Rush {}", input.0.rush_version);
    output_text(&text);

    Ok(Json(()))
}
