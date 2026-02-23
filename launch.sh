#!/usr/bin/env bash
# Launch script for zundamon_vrc
# Resolves paths relative to this script's directory

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="$SCRIPT_DIR/target/release/zundamon_vrc"

if [ ! -f "$BINARY" ]; then
    echo "バイナリが見つかりません: $BINARY"
    echo "先にビルドしてください: cargo build --release"
    exit 1
fi

exec "$BINARY"
