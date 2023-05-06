# Example rush plugin

To build:

```rs
rustup target add wasm32-wasi
cargo build --release --target wasm32-wasi
```

Plugin will be located at: `./target/wasm32-wasi/release/example.wasm`
