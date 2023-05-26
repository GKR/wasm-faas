# wasm-faas

A simple WebAssembly-powered serverless platform written in Rust.

## Compile Rust with rustc (wasm32-wasi)

**target wasm32-wasi**
```
rustc --crate-name hello_world_rs \
    --edition=2021 src/main.rs \
    --error-format=json \
    --json=diagnostic-rendered-ansi,artifacts,future-incompat \
    --diagnostic-width=211 \
    --crate-type bin \
    --emit=dep-info,link \
    -C opt-level=3 \
    -C embed-bitcode=no \
    -C metadata=9a3c5c9cc0837b06 \
    -C extra-filename=-9a3c5c9cc0837b06 \
    --out-dir /home/gunnar/wasm-faas-gkr/examples/hello-world-rs/target/wasm32-wasi/release/deps \
    --target wasm32-wasi \
    -L dependency=/home/gunnar/wasm/wasm-faas-gkr/examples/hello-world-rs/target/wasm32-wasi/release/deps \
    -L dependency=/home/gunnar/wasm/wasm-faas-gkr/examples/hello-world-rs/target/release/deps
```

**target normal**
```
rustc --crate-name hello_world_rs \
    --edition=2021 src/main.rs \
    --error-format=json \
    --json=diagnostic-rendered-ansi,artifacts,future-incompat \
    --diagnostic-width=211 \
    --crate-type bin \
    --emit=dep-info,link \
    -C opt-level=3 \
    -C embed-bitcode=no \
    -C metadata=b6e3176b59525a0d \
    -C extra-filename=-b6e3176b59525a0d \
    --out-dir /home/gunnar/wasm/wasm-faas-gkr/examples/hello-world-rs/target/release/deps \
    -L dependency=/home/gunnar/wasm/wasm-faas-gkr/examples/hello-world-rs/target/release/deps
```

Copyright 2022 Colin Eberhardt, MIT licence.
