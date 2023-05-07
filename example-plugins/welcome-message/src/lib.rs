use rush_pdk::*;

// rush_plugin_init runs on startup
#[plugin_fn]
pub fn rush_plugin_init(input: Json<InitHookParams>) -> FnResult<Json<()>> {
    // print welcome message to console
    let text = format!("Hello Rush {}", input.0.rush_version);
    output_text(&text);

    env::load_host_vars();
    output_text(&format!(
        "Your env vars:\n{:#?}",
        std::env::vars().collect::<Vec<_>>()
    ));

    Ok(Json(()))
}
