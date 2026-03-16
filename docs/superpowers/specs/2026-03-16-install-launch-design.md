# Install & Launch Simplification Design

## Overview

Create `install.sh` for one-command setup and update `launch.sh` to start everything (VOICEVOX Docker + app) from a desktop shortcut.

**Target:** Arch/Manjaro Linux only (pacman).

---

## install.sh

### Step 1: Dependency Installation (pacman)

Install these packages if not already present:

- `base-devel` — build tools
- `rust` — Rust toolchain (includes cargo)
- `docker` — container runtime
- `pulseaudio` — audio system (pactl, paplay)
- `noto-fonts-cjk` — Japanese font for UI
- `yt-dlp` — media URL playback
- `ffmpeg` — audio format conversion

### Step 2: GPU Detection & VOICEVOX Image Selection

- Check for NVIDIA GPU via `lspci | grep -i nvidia`
- If found, ask user:
  ```
  NVIDIA GPUが検出されました。GPU版VOICEVOXを使いますか？
  GPU版は合成速度が大幅に速くなります。
  1) GPU版 (nvidia-container-toolkit が必要です)
  2) CPU版
  ```
- GPU版選択時: install `nvidia-container-toolkit` (from AUR or official repo), use image `voicevox/voicevox_engine:nvidia-latest`
- CPU版選択時: use image `voicevox/voicevox_engine:latest`

### Step 3: Docker Setup

- `sudo systemctl enable --now docker`
- `sudo usermod -aG docker $USER` (if user not already in docker group)
- `docker pull <selected_image>`
- If user was added to docker group, warn that re-login is needed

### Step 4: Build

- `cargo build --release`

### Step 5: Install

- Copy `target/release/zundamon_vrc` to `~/.local/bin/zundamon_vrc`
- Copy updated `launch.sh` to `~/.local/bin/zundamon_vrc_launch.sh`
- Generate `.desktop` file pointing to the installed launch script, place in `~/.local/share/applications/`
- Write VOICEVOX Docker command to config file (`~/.config/zundamon_vrc/config.toml`):
  - GPU: `voicevox_path = "docker run --rm --gpus all -p 50021:50021 voicevox/voicevox_engine:nvidia-latest"`
  - CPU: `voicevox_path = "docker run --rm -p 50021:50021 voicevox/voicevox_engine:latest"`
  - `auto_launch_voicevox = true`

### Step 6: Completion Message

```
インストール完了！
アプリケーションメニューから「ずんだもん VRC」を起動できます。
(dockerグループ追加が行われた場合: 再ログインが必要です)
```

---

## launch.sh (Updated)

The existing launch.sh simply runs the binary. The updated version:

1. Start VOICEVOX Docker container in background (if not already running)
2. Launch the app binary
3. On app exit, stop the VOICEVOX container

```bash
#!/usr/bin/env bash
VOICEVOX_CONTAINER="zundamon-voicevox"
# Read docker image from config or use default
VOICEVOX_IMAGE="${VOICEVOX_IMAGE:-voicevox/voicevox_engine:latest}"

# Start VOICEVOX if not running
if ! docker ps --format '{{.Names}}' | grep -q "^${VOICEVOX_CONTAINER}$"; then
    docker run -d --rm --name "$VOICEVOX_CONTAINER" -p 50021:50021 "$VOICEVOX_IMAGE"
fi

# Run the app
~/.local/bin/zundamon_vrc

# Stop VOICEVOX on exit
docker stop "$VOICEVOX_CONTAINER" 2>/dev/null
```

Note: The app already handles VOICEVOX launch via `auto_launch_voicevox` config + Docker command in `voicevox_path`. However, the launch.sh approach is more reliable for desktop shortcut use because:
- The app's Docker launch uses `docker run` which blocks a thread waiting for the container
- launch.sh uses `docker run -d` (detached) which returns immediately
- launch.sh handles cleanup on app exit regardless of how the app terminates

Since launch.sh handles VOICEVOX startup, set `auto_launch_voicevox = false` in config to avoid double-launch. The `voicevox_path` is still kept for manual launches from the settings UI.

---

## Files

- `install.sh` — NEW: one-command installer
- `launch.sh` — MODIFY: add VOICEVOX Docker management
- `zundamon_vrc.desktop` — MODIFY: update Exec path to installed location

No Rust code changes needed.
