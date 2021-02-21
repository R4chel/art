#!/usr/bin/env bash
cd $(dirname $0)
set -ex
outdir=dist
mkdir -p $outdir
cargo +nightly build --target wasm32-unknown-unknown
wasm-bindgen --target web target/wasm32-unknown-unknown/debug/art.wasm --out-dir $outdir
cp index.html $outdir/index.html
