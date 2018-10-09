#!/bin/sh

set -e


# Prepare out dir
out_dir="dist/"
mkdir -p $out_dir

# Compiling Rust code
echo "  [1/3] 🌀 Running 'cargo build'"
cargo build --target wasm32-unknown-unknown

# Running `wasm-bindgen`
echo "  [2/3] 🔗 Running 'wasm-bindgen'"
wasm-bindgen \
    --out-dir $out_dir \
    --no-typescript \
    --no-modules \
    target/wasm32-unknown-unknown/debug/cantucci.wasm

# Copying everything into dist
echo "  [3/3] 🎁 Copying files into '$out_dir'"
cp src/index.html dist/
cp src/index.js dist/
