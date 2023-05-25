#!/bin/sh

mkdir target
rm /target/*

# build wasm-faas
(cd wasm-faas; cargo build --release)
cp wasm-faas/target/debug/wasm-faas ./target
cp wasm-faas/target/release/wasm-faas ./target

# build examples

# build hello-world-as
(cd examples/hello-world-as; npm i ; npm run build)
cp examples/hello-world-as/build/hello-world.wasm ./target

# build hello-world-rs
(cd examples/hello-world-rs; cargo build --release --target wasm32-wasi)
cp examples/hello-world-rs/target/wasm32-wasi/release/hello-world-rs.wasm ./target

# build option-pricing-as
(cd examples/option-pricing-as; npm i ; npm run build)
cp examples/option-pricing-as/build/option-pricing.wasm ./target

# build sudoku-rs
(cd examples/sudoku-rs; cargo build --release --target wasm32-wasi)
cp examples/sudoku-rs/target/wasm32-wasi/release/sudoku-rs.wasm ./target

cp compile-rs.sh ./target
