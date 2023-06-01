#!/bin/bash
#    --edition=2021 src/main.rs \
#    --emit=dep-info,link \
# -C extra-filename=-9a3c5c9cc0837b06 \
# --out-dir "$2/target/wasm32-wasi/release/deps" \
# -L dependency=$2/target/wasm32-wasi/release/deps \
# -L dependency=$2/target/release/deps
rustc --crate-name hello_world_rs \
    --edition=2021 $1 \
    --error-format=json \
    --json=diagnostic-rendered-ansi,artifacts,future-incompat \
    --diagnostic-width=211 \
    --crate-type bin \
    -C opt-level=3 \
    -C embed-bitcode=no \
    -C metadata=9a3c5c9cc0837b06 \
    -o $3.wasm \
    --target wasm32-wasi
    