# UX Overhaul: Install Simplification & UI Redesign

**Date**: 2026-03-19
**Status**: Reviewed

## Overview

Two-part improvement to zundamon_vrc: (1) simplify installation and launch by distributing pre-built binaries and automating VOICEVOX lifecycle, and (2) redesign the UI with a TOML-based theme system, transparent window, custom title bar, and refreshed input screen layout.

Additionally, this spec includes a mandatory **Part 0** to fix existing security vulnerabilities that the new features would amplify.

## Part 0: Security Hardening (Prerequisites)

These fixes address existing vulnerabilities in the codebase. They MUST be completed before implementing Parts 1 and 2, since the auto-launch feature (§1.3) would make the command injection issues trigger automatically at startup.

### 0.1 Fix Command Injection via `voicevox_path`

**Problem**: `voicevox_path` from user-editable `config.toml` is passed directly to `sh -c` in `src/app.rs`. A value like `voicevox --foo; rm -rf ~` executes arbitrary shell commands.

**Fix**:
- Parse `voicevox_path` into executable + argument list using `shell-words` crate (or manual split)
- Spawn with `Command::new(exe).args(args)` — no shell involvement
- For Docker case: detect `docker` prefix, extract image/flags into structured fields, build `Command` directly
- Reject values containing shell metacharacters (``; | & $ ` ( ) { } < >``) at config load time with a warning log

### 0.2 Fix Command Injection via URL Player Pipeline

**Problem**: `src/media/url_player.rs` builds a shell pipeline interpolating user URL and config device name. Even with single-quote escaping, shell pipelines are inherently fragile.

**Fix**:
- Replace shell pipeline with Rust process pipeline: `yt-dlp` stdout piped to `ffmpeg` stdin piped to `paplay` stdin via `std::process::Stdio::piped()`
- Pass URL to `yt-dlp` and device name to `paplay` as discrete `Command` arguments
- No shell involvement at all

### 0.3 Validate `virtual_device_name`

**Problem**: `virtual_device_name` from config is used in `pactl` arguments without validation. Names containing `=` or spaces corrupt PulseAudio module arguments.

**Fix**:
- Validate at config load time: must match `[a-zA-Z0-9_-]+`, max 64 characters
- Reject and fall back to default (`ZundamonVRC`) if invalid
- Apply same validation in settings UI input field

### 0.4 Validate `voicevox_url`

**Problem**: `voicevox_url` is passed directly to HTTP client without scheme/host validation. Could be pointed at arbitrary internal network endpoints.

**Fix**:
- Parse with `url` crate at config load time
- Reject if host is not `127.0.0.1`, `localhost`, or `[::1]`
- Reject if scheme is not `http`
- Apply same validation in settings UI

## Part 1: Installation & Launch Simplification

### 1.1 GitHub Actions Release Pipeline

**Trigger**: Push of a `v*` tag (e.g., `v0.2.0`).

**Workflow** (`.github/workflows/release.yml`):
- Runner: `ubuntu-22.04` (not `latest`, for broader glibc compatibility)
- Install system dependencies: `libgtk-3-dev`, `libasound2-dev`, `libpulse-dev`, etc.
- Run `cargo build --release`
- Rename binary to `zundamon_vrc-linux-x86_64`
- Compute SHA256 checksum: `sha256sum zundamon_vrc-linux-x86_64 > SHA256SUMS`
- Create GitHub Release via `gh release create` with binary + `SHA256SUMS` attached

**Artifact**: Dynamically-linked binary for x86_64 Linux. **Minimum requirements**: glibc 2.35+ (Ubuntu 22.04 baseline), PulseAudio, GTK3, ALSA. Documented in release notes.

### 1.2 install.sh Rework

**Default behavior changes from source build to binary download**:

1. Detect latest release from GitHub API (`https://api.github.com/repos/<owner>/<repo>/releases/latest`)
2. Validate download URL matches expected pattern: `https://github.com/<owner>/<repo>/releases/download/v[0-9]+.[0-9]+.[0-9]+/zundamon_vrc-linux-x86_64`
3. Download binary + `SHA256SUMS` via `curl -fsSL` (HTTPS only)
4. Verify integrity: `sha256sum -c SHA256SUMS`
5. Place binary at `~/.local/bin/zundamon_vrc`
6. `--from-source` flag preserves the current source-build path (requires Rust toolchain)
7. `PACKAGES` list adjusted: `rust` and `base-devel` only required for `--from-source`

**Launcher script generation**: Use `printf '%q'` for shell-safe variable embedding instead of unquoted heredoc expansion.

**Unchanged**: Docker setup, GPU detection, desktop entry creation.

### 1.3 VOICEVOX Auto-Launch (In-App)

**Startup sequence**:
1. App starts → health check `GET http://<voicevox_url>/version`
2. If healthy → proceed normally, status: "Connected"
3. If unhealthy → check `config.toml` for `voicevox_path`
4. If `voicevox_path` is set → check for existing process/container first (duplicate guard)
5. If no existing process → spawn via validated `Command` (see §0.1)
6. Retry health check: 1-second interval, max 30 seconds
7. UI shows "VOICEVOXを起動中..." with spinner during wait
8. If timeout → keep spawned process alive, continue periodic health checks at 5-second interval. Status: "Disconnected (starting...)"

**Duplicate process guard**:
- For Docker: check `docker ps --filter name=zundamon-voicevox` before spawning
- For local binary: store PID, check `/proc/<pid>/status` before re-spawning
- Never spawn a second instance

**Shutdown**:
- `Drop` implementation on a dedicated cleanup struct (not `AppState` directly)
- If app spawned the VOICEVOX process → `docker stop` or `kill` with SIGTERM
- Graceful: try stop first, force-kill after 5 seconds
- Register `SIGTERM` handler via `ctrlc` crate for cleanup on force-quit

**Config interaction**:
- `auto_launch_voicevox = false` (default) — preserves backward compatibility with existing configs
- `auto_launch_voicevox = true` — enables auto-launch behavior
- Installer sets `auto_launch_voicevox = true` for new installs
- Existing configs without this field keep `false` (no surprise behavior change)

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

**Type definitions**:
- Color fields: `[u8; 4]` — TOML deserialization rejects values outside 0-255 automatically
- Float fields: `f32` — validated after deserialization: must be finite, rounding in 0.0-50.0, spacing in 0.0-100.0
- Validation via `Theme::validate()` called immediately after `toml::from_str`
- Invalid individual fields fall back to their default value with a `tracing::warn!` log

**Behavior**:
- On startup, read `[theme]` section from `config.toml`
- Existing configs without `[theme]` use built-in defaults (`#[serde(default)]` on `Theme`)
- Convert `Theme` → `egui::Visuals` + `egui::Style` **once at startup** and cache
- Theme changes require app restart (hot-reload is a future enhancement)

**Config file limits**:
- Cap file read size at 1 MB before parsing
- String fields: URLs ≤ 2048 chars, paths ≤ 4096 chars
- `templates` vector: max 100 entries, each ≤ 512 chars

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
- Uses wgpu backend (eframe 0.31 default) — alpha compositing requires compositor support for the Wayland surface

### 2.3 Custom Title Bar

**Implementation** via `TopBottomPanel::top("titlebar")` (separate from tab bar panel):
- Height: ~32px
- Background: `titlebar_background` from theme
- Left region: drag handle (entire left area is draggable via `ui.interact()` + `ViewportCommand::StartDrag`)
- Center: "ZUNDAMON VRC" label in `titlebar_text` color, small caps, letter-spaced
- Right: minimize / maximize / close buttons using `ViewportCommand::Minimize`, `Maximized`, `Close`
- Buttons styled as subtle icons, highlight on hover with `accent` color

**Window resize**:
- Keep `with_resizable(true)` — Wayland compositors (KWin, Mutter) provide resize handles even without server-side decorations via protocol-level resize zones
- If resize doesn't work on a specific compositor, the window remains at its current size (graceful degradation)

**Maximized state**:
- When maximized: suppress `window_rounding` (set to 0.0), title bar buttons update visual state (maximize icon → restore icon)
- Detect via `ctx.input(|i| i.viewport().maximized.unwrap_or(false))`

**Keyboard shortcuts**:
- `Alt+F4` → close (handle in egui input processing)
- Standard egui keyboard navigation preserved

**Full vertical layout order**:
1. Custom Title Bar (`TopBottomPanel::top("titlebar")`, 32px)
2. Tab Bar (`TopBottomPanel::top("tabs")`)
3. Status Bar (`TopBottomPanel::bottom("status")`)
4. Content Area (`CentralPanel`)

### 2.4 Input Screen Redesign (Vertical Stack)

**Layout top-to-bottom** (within CentralPanel):

1. **Speaker selector** — label "SPEAKER" in `text_muted` (small uppercase), ComboBox with `input_background` and `input_rounding`
2. **Text input** — multi-line `TextEdit`, `input_background`, `input_rounding`, 3+ rows. Enter to send, Shift+Enter for newline (unchanged behavior)
3. **Send button** — right-aligned via `ui.with_layout(Layout::right_to_left(Align::Center), ...)`, `accent` background, `button_rounding`
4. **Templates section** — label "TEMPLATES" in `text_muted`, horizontal_wrapped chips:
   - Each chip: `chip_background`, `chip_rounding` (pill shape)
   - **Text truncation**: strings longer than 12 characters are truncated with `...`
   - **Tooltip**: hover shows full text
   - **Max 2 rows visible**: overflow shows a "+N more" chip that expands inline (pushes content down). Expanded state shows a "Show less" chip to collapse. Max 5 rows when expanded, scroll beyond that.
   - Delete: hover reveals `✕` on each chip
   - Last chip: "+ Add" for new template creation
5. **Error display** — toast-style notification above status bar, auto-dismisses after **5 seconds**. Hover pauses the timer. Click dismisses immediately. Single error at a time (matches current `Option<String>` field).

**Other screens** (Soundboard, Media, Settings):
- Theme colors applied automatically via `egui::Visuals`
- Layout unchanged — keep simple and functional

## Technical Considerations

### Dependencies

**New crates**:
- `shell-words` — for safe parsing of `voicevox_path` into executable + args (§0.1)
- `ctrlc` — for SIGTERM handler to ensure cleanup (§1.3)
- `url` — for `voicevox_url` validation (§0.4)

**Existing**: eframe 0.31 already supports all required viewport/drawing features.

### File Changes

**New files**:
- `src/ui/theme.rs` — `Theme` struct, TOML deserialization, validation, conversion to egui Visuals/Style
- `src/ui/titlebar.rs` — custom title bar rendering
- `.github/workflows/release.yml` — CI release pipeline

**Modified files**:
- `src/main.rs` — viewport builder config (transparent, no decorations), clear color
- `src/app.rs` — load theme, apply visuals, VOICEVOX auto-launch logic, command injection fixes
- `src/ui/input.rs` — redesigned layout (vertical stack, chips, status bar)
- `src/config.rs` — add `Theme` fields, input validation (device name, URL, file size cap)
- `src/media/url_player.rs` — replace shell pipeline with Rust process pipeline
- `src/audio/virtual_device.rs` — validate sink name
- `install.sh` — binary download mode, checksum verification, safe variable embedding

**Unchanged**:
- `src/ui/settings.rs` — theme colors auto-applied, layout stays
- `src/ui/soundboard.rs` — same
- `src/ui/media.rs` — same
- `src/tts/` — no changes

### Risks

1. **Wayland transparency**: compositor-dependent, mitigated by opaque fallback
2. **Custom title bar UX**: drag area must be clearly communicated; entire title bar region (minus buttons) is draggable. Resize relies on compositor protocol support.
3. **Theme TOML parsing errors**: per-field fallback to defaults with warning log; app does not crash
4. **Binary portability**: dynamically linked against glibc 2.35+. Documented as minimum requirement. Older distros use `--from-source`.
5. **VOICEVOX process lifecycle**: duplicate guard prevents multiple instances. SIGTERM handler ensures cleanup. Stale containers self-heal on next startup via `docker rm -f`.
