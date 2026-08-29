#!/usr/bin/env bash
# Requires: rustup target add wasm32-unknown-unknown
set -euo pipefail
cargo build --release
cp target/wasm32-unknown-unknown/release/games.wasm www/games.wasm
ls -lh www/games.wasm
