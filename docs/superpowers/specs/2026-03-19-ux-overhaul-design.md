# UX Overhaul: Install Simplification & UI Redesign

**Date**: 2026-03-19
**Status**: Draft

## Overview

Two-part improvement to zundamon_vrc: (1) simplify installation and launch by distributing pre-built binaries and automating VOICEVOX lifecycle, and (2) redesign the UI with a TOML-based theme system, transparent window, custom title bar, and refreshed input screen layout.

## Part 1: Installation & Launch Simplification

### 1.1 GitHub Actions Release Pipeline

**Trigger**: Push of a `v*` tag (e.g., `v0.2.0`).

**Workflow** (`.github/workflows/release.yml`):
- Runner: `ubuntu-latest`
- Install system dependencies: `libgtk-3-dev`, `libasound2-dev`, `libpulse-dev`, etc.
- Run `cargo build --release`
- Rename binary to `zundamon_vrc-linux-x86_64`
- Create GitHub Release via `gh release create` with the binary attached

**Artifact**: Single statically-usable binary for x86_64 Linux.

### 1.2 install.sh Rework

**Default behavior changes from source build to binary download**:

1. Detect latest release from GitHub API (`https://api.github.com/repos/<owner>/<repo>/releases/latest`)
2. Download binary via `curl` (no `gh` CLI required)
3. Place binary at `~/.local/bin/zundamon_vrc`
4. `--from-source` flag preserves the current source-build path (requires Rust toolchain)
5. `PACKAGES` list adjusted: `rust` and `base-devel` only required for `--from-source`

**Unchanged**: Docker setup, GPU detection, launcher script, desktop entry creation.

### 1.3 VOICEVOX Auto-Launch (In-App)

**Startup sequence**:
1. App starts → health check `GET http://<voicevox_url>/version`
2. If healthy → proceed normally, status: "Connected"
3. If unhealthy → check `config.toml` for `voicevox_path`
4. If `voicevox_path` is set → spawn process (Docker or local binary)
5. Retry health check: 1-second interval, max 30 seconds
6. UI shows "VOICEVOXを起動中..." with spinner during wait
7. If timeout → status: "Disconnected", user can retry manually

**Shutdown**:
- `Drop` implementation on `AppState` (or a dedicated cleanup struct)
- If app spawned the VOICEVOX process → kill it / `docker stop` the container
- Graceful: try stop first, force-kill after 5 seconds

**Config interaction**:
- `auto_launch_voicevox = true` (default) — enables the above behavior
- `auto_launch_voicevox = false` — user manages VOICEVOX themselves
- Launcher script (`zundamon_vrc_launch.sh`) sets `auto_launch_voicevox = false` since it handles Docker externally

## Part 2: UI Redesign

### 2.1 Theme System

**`Theme` struct** — all visual properties in one place, deserialized from TOML:

```toml
[theme]
# Window
window_background = [15, 15, 20, 200]    # RGBA, alpha < 255 = translucent
window_rounding = 12.0

# Title bar
titlebar_background = [20, 20, 28, 240]
titlebar_text = [180, 180, 180, 255]

# Content
panel_background = [255, 255, 255, 15]
text_primary = [224, 224, 224, 255]
text_secondary = [160, 160, 160, 255]
text_muted = [100, 100, 100, 255]

# Accents
accent = [120, 200, 120, 255]
accent_hover = [140, 220, 140, 255]

# Status indicators
status_ok = [112, 192, 112, 255]
status_warn = [200, 200, 100, 255]
status_error = [200, 100, 100, 255]

# Widgets
button_background = [255, 255, 255, 15]
button_rounding = 6.0
input_background = [255, 255, 255, 10]
input_rounding = 8.0
chip_background = [255, 255, 255, 15]
chip_rounding = 16.0

# Tab bar
tab_active_background = [255, 255, 255, 30]
tab_rounding = 6.0

# Spacing
spacing_small = 4.0
spacing_medium = 8.0
spacing_large = 16.0
```

**Behavior**:
- On startup, read `[theme]` section from `config.toml`
- Missing fields fall back to built-in defaults (minimal dark theme)
- Convert `Theme` → `egui::Visuals` + `egui::Style` each frame
- Theme changes require app restart (hot-reload is a future enhancement)

### 2.2 Transparent Window

**eframe configuration**:
- `ViewportBuilder::default().with_transparent(true).with_decorations(false)`
- Clear color set to fully transparent `[0, 0, 0, 0]`
- Content area painted with `window_background` (semi-transparent) using rounded rect

**Wayland compatibility**:
- User runs Wayland on Manjaro Linux
- eframe uses `winit` which supports Wayland CSD (Client-Side Decorations) natively
- `with_decorations(false)` works correctly on Wayland
- Transparency depends on compositor (KWin, Mutter both support it)
- **Fallback**: if transparency is not visually working, the app still renders correctly — `window_background` alpha at 200/255 just appears as a solid dark color without compositing

### 2.3 Custom Title Bar

**Implementation** via `TopBottomPanel::top`:
- Height: ~32px
- Background: `titlebar_background` from theme
- Left region: drag handle (entire left area is draggable via `ui.interact()` + `ViewportCommand::StartDrag`)
- Center: "ZUNDAMON VRC" label in `titlebar_text` color, small caps, letter-spaced
- Right: minimize / maximize / close buttons using `ViewportCommand::Minimize`, `Maximized`, `Close`
- Buttons styled as subtle icons, highlight on hover with `accent` color

### 2.4 Input Screen Redesign (Vertical Stack)

**Layout top-to-bottom**:

1. **Speaker selector** — label "SPEAKER" in `text_muted` (small uppercase), ComboBox with `input_background` and `input_rounding`
2. **Text input** — multi-line `TextEdit`, `input_background`, `input_rounding`, 3+ rows. Enter to send, Shift+Enter for newline (unchanged behavior)
3. **Send button** — right-aligned, `accent` background, `button_rounding`
4. **Templates section** — label "TEMPLATES" in `text_muted`, horizontal_wrapped chips:
   - Each chip: `chip_background`, `chip_rounding` (pill shape)
   - **Text truncation**: strings longer than 12 characters are truncated with `...`
   - **Tooltip**: hover shows full text
   - **Max 2 rows visible**: overflow shows a "+N more" chip that expands on click
   - Delete: hover reveals `✕` on each chip
   - Last chip: "+ Add" for new template creation
5. **Status bar** (`BottomPanel`) — fixed at bottom, shows VOICEVOX and Virtual Mic status with colored dots (`status_ok` / `status_warn` / `status_error`)
6. **Error display** — toast-style notification above status bar, auto-dismisses after a few seconds

**Other screens** (Soundboard, Media, Settings):
- Theme colors applied automatically via `egui::Visuals`
- Layout unchanged — keep simple and functional

## Technical Considerations

### Dependencies

No new crate dependencies expected. eframe 0.31 already supports:
- `with_transparent(true)` and `with_decorations(false)` via `ViewportBuilder`
- `ViewportCommand` for drag, minimize, maximize, close
- Custom painting for rounded rects and styled widgets

### File Changes

**New files**:
- `src/ui/theme.rs` — `Theme` struct, TOML deserialization, conversion to egui Visuals/Style
- `src/ui/titlebar.rs` — custom title bar rendering
- `.github/workflows/release.yml` — CI release pipeline

**Modified files**:
- `src/main.rs` — viewport builder config (transparent, no decorations), clear color
- `src/app.rs` — load theme, apply visuals, VOICEVOX auto-launch logic
- `src/ui/input.rs` — redesigned layout (vertical stack, chips, status bar)
- `src/config.rs` — add `Theme` fields to config, deserialization
- `install.sh` — binary download mode as default

**Unchanged**:
- `src/ui/settings.rs` — theme colors auto-applied, layout stays
- `src/ui/soundboard.rs` — same
- `src/ui/media.rs` — same
- `src/tts/` — no changes
- `src/audio/` — no changes

### Risks

1. **Wayland transparency**: compositor-dependent, mitigated by opaque fallback
2. **Custom title bar UX**: drag area must be clearly communicated; entire title bar region (minus buttons) is draggable
3. **Theme TOML parsing errors**: invalid values fall back to defaults with a warning log, app does not crash
