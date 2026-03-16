#!/usr/bin/env bash
# Launch script for zundamon_vrc
# Can be used directly from the repo (dev mode) or via install.sh

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Determine binary location
if [ -f "$HOME/.local/bin/zundamon_vrc" ]; then
    BINARY="$HOME/.local/bin/zundamon_vrc"
elif [ -f "$SCRIPT_DIR/target/release/zundamon_vrc" ]; then
    BINARY="$SCRIPT_DIR/target/release/zundamon_vrc"
else
    echo "バイナリが見つかりません。"
    echo "install.sh を実行するか、cargo build --release でビルドしてください。"
    exit 1
fi

exec "$BINARY"
