# Rush Plugin Development Kit

> Write plugins that extend the behavior of the Rush shell

## Installation

```sh
cargo add https://github.com/Eyesonjune18/rush.git rush-pdk
```

## Overview

Plugins are WebAssembly (`wasm`) modules that can provide unique functionality to the Rush shell. The Rush PDK allows you to react to events broadcasted by Rush and affect the shell from your plugins.


Rust 🦀 compiled to one of the `wasm32-wasi` or `wasm32-unknown-unknown` targets is the officially supported method for writing plugins.

```sh
rustup target add wasm32-wasi
```

## Usage

Rush PDK provides a `plugin_fn` attribute which enables the Rush shell to run your event listeners. The name of the function you apply it to determines when it will be run.

```rs
use rush_pdk::*;

#[plugin_fn]
pub fn rush_plugin_init(init_info: Json<InitHookParams>) -> FnResult<Json<()>> {
    // print a welcome message on startup
    let msg = format!("Welcome to Rush v{}!", init_info.0.rush_version);
    output_text(msg);

    Ok(Json(()))
}
```
