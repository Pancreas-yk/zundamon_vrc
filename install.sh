#!/usr/bin/env bash
set -euo pipefail

# ===== ずんだもん VRC インストーラー (Arch/Manjaro) =====

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; }

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
INSTALL_BIN="$HOME/.local/bin"
INSTALL_APPS="$HOME/.local/share/applications"
CONFIG_DIR="$HOME/.config/zundamon_vrc"

# ---------- Step 1: 依存パッケージのインストール ----------
info "依存パッケージを確認中..."

PACKAGES=(base-devel rust docker pulseaudio noto-fonts-cjk yt-dlp ffmpeg)
MISSING=()

for pkg in "${PACKAGES[@]}"; do
    if ! pacman -Qi "$pkg" &>/dev/null; then
        MISSING+=("$pkg")
    fi
done

if [ ${#MISSING[@]} -gt 0 ]; then
    info "以下のパッケージをインストールします: ${MISSING[*]}"
    sudo pacman -S --needed --noconfirm "${MISSING[@]}"
else
    info "すべての依存パッケージがインストール済みです"
fi

# ---------- Step 2: GPU検出 & VOICEVOXイメージ選択 ----------
VOICEVOX_IMAGE="voicevox/voicevox_engine:latest"
GPU_FLAGS=""

if lspci | grep -qi nvidia; then
    echo ""
    echo -e "${BOLD}NVIDIA GPUが検出されました。GPU版VOICEVOXを使いますか？${NC}"
    echo "GPU版は合成速度が大幅に速くなります。"
    echo "  1) GPU版 (nvidia-container-toolkit が必要です)"
    echo "  2) CPU版"
    echo ""
    read -rp "選択 [1/2] (デフォルト: 1): " gpu_choice
    gpu_choice="${gpu_choice:-1}"

    if [ "$gpu_choice" = "1" ]; then
        VOICEVOX_IMAGE="voicevox/voicevox_engine:nvidia-latest"
        GPU_FLAGS="--gpus all"

        # nvidia-container-toolkit のインストール
        if ! pacman -Qi nvidia-container-toolkit &>/dev/null; then
            info "nvidia-container-toolkit をインストール中..."
            # Try pacman first, fall back to AUR
            if pacman -Si nvidia-container-toolkit &>/dev/null; then
                sudo pacman -S --needed --noconfirm nvidia-container-toolkit
            else
                warn "nvidia-container-toolkit が公式リポジトリにありません"
                warn "AUR からインストールしてください: yay -S nvidia-container-toolkit"
                echo ""
                read -rp "続行しますか？ (y/N): " cont
                if [ "$cont" != "y" ] && [ "$cont" != "Y" ]; then
                    error "インストールを中断しました"
                    exit 1
                fi
            fi
        fi
        info "GPU版VOICEVOXを使用します"
    else
        info "CPU版VOICEVOXを使用します"
    fi
else
    info "NVIDIA GPUが検出されませんでした。CPU版VOICEVOXを使用します"
fi

# ---------- Step 3: Docker設定 ----------
info "Dockerを設定中..."

# Enable and start docker service
if ! systemctl is-active --quiet docker; then
    sudo systemctl enable --now docker
    info "Dockerサービスを起動しました"
fi

# Add user to docker group
if ! groups "$USER" | grep -q '\bdocker\b'; then
    sudo usermod -aG docker "$USER"
    NEED_RELOGIN=true
    warn "ユーザーをdockerグループに追加しました（再ログインが必要です）"
else
    NEED_RELOGIN=false
fi

# Pull VOICEVOX image
info "VOICEVOXイメージをダウンロード中... (時間がかかる場合があります)"
if [ "$NEED_RELOGIN" = true ]; then
    sudo docker pull "$VOICEVOX_IMAGE"
else
    docker pull "$VOICEVOX_IMAGE"
fi

# ---------- Step 4: ビルド ----------
info "アプリケーションをビルド中..."
cd "$SCRIPT_DIR"
cargo build --release

# ---------- Step 5: インストール ----------
info "インストール中..."

mkdir -p "$INSTALL_BIN"
mkdir -p "$INSTALL_APPS"
mkdir -p "$CONFIG_DIR"

# Copy binary
cp "$SCRIPT_DIR/target/release/zundamon_vrc" "$INSTALL_BIN/zundamon_vrc"
info "バイナリをインストール: $INSTALL_BIN/zundamon_vrc"

# Build docker run command
DOCKER_CMD="docker run --rm ${GPU_FLAGS:+$GPU_FLAGS }-p 50021:50021 $VOICEVOX_IMAGE"

# Create launch script
cat > "$INSTALL_BIN/zundamon_vrc_launch.sh" << 'LAUNCHER_EOF'
#!/usr/bin/env bash
VOICEVOX_CONTAINER="zundamon-voicevox"
LAUNCHER_EOF

# Append image and GPU flags (with variable expansion)
cat >> "$INSTALL_BIN/zundamon_vrc_launch.sh" << LAUNCHER_DYNAMIC_EOF
VOICEVOX_IMAGE="$VOICEVOX_IMAGE"
GPU_FLAGS="$GPU_FLAGS"
LAUNCHER_DYNAMIC_EOF

cat >> "$INSTALL_BIN/zundamon_vrc_launch.sh" << 'LAUNCHER_EOF'

# Start VOICEVOX if not running
if ! docker ps --format '{{.Names}}' | grep -q "^${VOICEVOX_CONTAINER}$"; then
    docker run -d --rm --name "$VOICEVOX_CONTAINER" ${GPU_FLAGS:+$GPU_FLAGS} -p 50021:50021 "$VOICEVOX_IMAGE" >/dev/null 2>&1
fi

# Run the app
"$HOME/.local/bin/zundamon_vrc"

# Stop VOICEVOX on exit
docker stop "$VOICEVOX_CONTAINER" 2>/dev/null || true
LAUNCHER_EOF

chmod +x "$INSTALL_BIN/zundamon_vrc_launch.sh"
info "ランチャーをインストール: $INSTALL_BIN/zundamon_vrc_launch.sh"

# Create .desktop file
cat > "$INSTALL_APPS/zundamon_vrc.desktop" << DESKTOP_EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=ずんだもん VRC
Comment=VOICEVOX TTS for VRChat via virtual microphone
Exec=$INSTALL_BIN/zundamon_vrc_launch.sh
Icon=audio-input-microphone
Terminal=false
Categories=AudioVideo;Audio;
DESKTOP_EOF

info "デスクトップエントリをインストール: $INSTALL_APPS/zundamon_vrc.desktop"

# Update config with VOICEVOX Docker settings
if [ -f "$CONFIG_DIR/config.toml" ]; then
    # Update existing config
    if grep -q 'voicevox_path' "$CONFIG_DIR/config.toml"; then
        sed -i "s|^voicevox_path.*|voicevox_path = \"$DOCKER_CMD\"|" "$CONFIG_DIR/config.toml"
    else
        echo "voicevox_path = \"$DOCKER_CMD\"" >> "$CONFIG_DIR/config.toml"
    fi
    # Don't auto-launch from app since launch.sh handles it
    if grep -q 'auto_launch_voicevox' "$CONFIG_DIR/config.toml"; then
        sed -i 's|^auto_launch_voicevox.*|auto_launch_voicevox = false|' "$CONFIG_DIR/config.toml"
    fi
else
    # Create minimal config
    cat > "$CONFIG_DIR/config.toml" << CONFIG_EOF
voicevox_path = "$DOCKER_CMD"
auto_launch_voicevox = false
CONFIG_EOF
fi

info "設定ファイルを更新: $CONFIG_DIR/config.toml"

# ---------- 完了 ----------
echo ""
echo -e "${GREEN}${BOLD}===== インストール完了！ =====${NC}"
echo ""
echo "アプリケーションメニューから「ずんだもん VRC」を起動できます。"
echo "またはコマンドラインから: zundamon_vrc_launch.sh"
echo ""
if [ "$NEED_RELOGIN" = true ]; then
    echo -e "${YELLOW}${BOLD}重要: dockerグループへの追加を反映するため、再ログインしてください。${NC}"
    echo ""
fi
