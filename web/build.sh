#!/usr/bin/env bash
# Build the WASM core and drop it next to index.html, then serve.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown --lib
cp target/wasm32-unknown-unknown/release/tactus_leafgen.wasm web/leafgen.wasm
echo "ok -> web/leafgen.wasm ($(du -h web/leafgen.wasm | cut -f1))"
echo "serve with:  (cd web && python3 -m http.server 8000)  then open http://localhost:8000"
