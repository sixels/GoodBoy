#!/bin/env sh

set -e

BUILD_TYPE=""
test "${1}" = "release" && BUILD_TYPE="release" || BUILD_TYPE="debug"

echo "Compiling for ${BUILD_TYPE}..."
if [[ ${BUILD_TYPE} == "release" ]]; then
    cargo build --target wasm32-unknown-unknown --release
else
    cargo build --target wasm32-unknown-unknown
fi

echo "Generating wasm bindings..."
wasm-bindgen \
    --target web \
    --no-typescript \
    --out-dir target/wasm_bindings \
    target/wasm32-unknown-unknown/${BUILD_TYPE}/goodboy.wasm

cp web/index.html target/wasm_bindings/index.html


echo 'All done! run `python3 -m http.server --directory target/wasm_bindings 8080` to start a server'
