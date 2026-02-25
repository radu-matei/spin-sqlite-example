#!/usr/bin/env bash
set -e

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <database.db>"
    exit 1
fi

DB="$1"

cargo build --target wasm32-wasip1 --release --manifest-path api/Cargo.toml

WASM=api/target/wasm32-wasip1/release/api.wasm

echo "$DB" | wizer --allow-wasi --wasm-bulk-memory true \
    --dir . \
    -o "${WASM}.init" \
    "$WASM"

mv "${WASM}.init" "$WASM"

if command -v wasm-opt &> /dev/null; then
    wasm-opt -O3 --enable-bulk-memory-opt -o "$WASM" "$WASM"
fi

echo -n "Pre-initialized component size: "
ls -lh "$WASM" | awk '{print $5}'
