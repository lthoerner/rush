# PATH Autocomplete Plugin

Recieve autocomplete suggestions for external commands in your PATH

## Building

```rs
rustup target add wasm32-wasi
cargo build --release --target wasm32-wasi
```

Plugin will be located at: `./target/wasm32-wasi/release/path_autocomplete.wasm`
