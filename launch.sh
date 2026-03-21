#!/usr/bin/env bash
# Launch script for ZunduxTTS
# Can be used directly from the repo (dev mode) or via install.sh

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Determine binary location
if [ -f "$HOME/.local/bin/zundux_tts" ]; then
    BINARY="$HOME/.local/bin/zundux_tts"
elif [ -f "$SCRIPT_DIR/target/release/zundux_tts" ]; then
    BINARY="$SCRIPT_DIR/target/release/zundux_tts"
else
    echo "バイナリが見つかりません。"
    echo "install.sh を実行するか、cargo build --release でビルドしてください。"
    exit 1
fi

# Force X11 backend for IME (Japanese input) support on Wayland
# (WINIT_UNIX_BACKEND was removed in winit 0.29+; unsetting WAYLAND_DISPLAY forces X11)
unset WAYLAND_DISPLAY

# Set XMODIFIERS if not already set (detect running IME)
if [ -z "$XMODIFIERS" ]; then
    if pgrep -x fcitx5 >/dev/null 2>&1 || pgrep -x fcitx >/dev/null 2>&1; then
        export XMODIFIERS=@im=fcitx
    elif pgrep -x ibus-daemon >/dev/null 2>&1; then
        export XMODIFIERS=@im=ibus
    fi
fi

exec "$BINARY"
