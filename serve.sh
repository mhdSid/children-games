#!/usr/bin/env bash
# file:// can't fetch a .wasm — serve the folder instead.
cd "$(dirname "$0")/www" && python3 -m http.server 8080
