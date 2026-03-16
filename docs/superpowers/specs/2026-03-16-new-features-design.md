# New Features Design — zundamon_vrc

## Overview

Five new features for the zundamon_vrc Linux TTS-to-VRChat app:

1. OSC Chatbox Integration
2. User Dictionary (VOICEVOX)
3. Soundboard
4. Media Audio Routing
5. Audio Effects (Echo)

## Tab Structure

Current: Input | Settings
New: **Input | Soundboard | Media | Settings**

---

## 1. OSC Chatbox Integration

**Goal:** Display spoken text in VRChat's chatbox overlay.

**Mechanism:**
- Send UDP OSC message to `127.0.0.1:9000` (configurable)
- OSC address: `/chatbox/input` with args `(String text, bool immediate=true, bool sound=false)`
- Triggered *before* playback starts — when `WavReady` is received on the UI thread, send OSC immediately, then spawn the playback thread. This way chatbox text appears as speech begins.
- Works on Linux with VRChat via Proton (Proton uses host network stack)

**Settings UI (new section: "OSC設定"):**
- Enable/disable toggle (default: disabled)
- OSC destination address (default: `127.0.0.1`)
- OSC port (default: `9000`)

**Dependencies:**
- `rosc` crate for OSC message construction
- `std::net::UdpSocket` for sending (no async needed)

**Integration point:** `src/app.rs` `process_messages()` — in the `WavReady` handler, send OSC before spawning the playback thread. The original text must be carried through `WavReady` (add a `text: String` field to the variant).

---

## 2. User Dictionary

**Goal:** Register custom word pronunciations via VOICEVOX's built-in dictionary API.

**VOICEVOX API endpoints:**
- `GET /user_dict` — list all registered words
- `POST /user_dict?surface={word}&pronunciation={reading}&accent_type={int}` — register
- `DELETE /user_dict/{word_uuid}` — delete

**No local storage needed** — VOICEVOX persists the dictionary server-side.

**Settings UI (new section: "ユーザー辞書"):**
- Table of registered words (表記 | 読み | 削除ボタン)
- Add form: surface text input + pronunciation text input + "追加" button
- `accent_type` defaults to `1` (most common pattern); advanced users can adjust later
- Load word list on section expand or VOICEVOX connection

**Architecture:** Dictionary operations are VOICEVOX-specific and do not belong on the `TtsEngine` trait. Instead:
- Add methods directly to `VoicevoxEngine`: `list_user_dict()`, `add_user_dict_word()`, `delete_user_dict_word()`
- In `tts_loop`, downcast or store a separate handle to `VoicevoxEngine` alongside the `TtsManager` to dispatch dictionary commands
- New `TtsCommand` variants: `LoadUserDict`, `AddUserDictWord { surface, pronunciation }`, `DeleteUserDictWord { uuid }`
- Corresponding `UiMessage` variants: `UserDictLoaded`, `UserDictUpdated`, `Error`

---

## 3. Soundboard

**Goal:** One-click playback of local audio files through the virtual microphone.

**New tab: "サウンドボード"**

**Folder-based management:**
- Configurable folder path (default: derived from `directories::ProjectDirs`, typically `~/.config/zundamon_vrc/sounds/`)
- Path stored in config as absolute expanded path (no tilde — use `ProjectDirs` at default creation time)
- Auto-scan for `.wav`, `.mp3`, `.ogg` files
- Display file names (without extension) as buttons in a grid layout (similar to template grid)
- "再スキャン" (rescan) button to refresh file list
- Folder path configurable in Settings

**Playback:**
- Click button → play file through virtual microphone
- Uses existing `playback` module (rodio primary, paplay fallback)
- Monitor playback follows existing monitor setting
- **Exclusive with TTS** — if TTS is synthesizing/playing, soundboard waits; if soundboard is playing, TTS waits

**Config additions:**
- `soundboard_path: String` (default: from `ProjectDirs` + `/sounds/`)

**Required dependency change:**
```toml
rodio = { version = "0.20", features = ["mp3", "vorbis"] }
```

---

## 4. Media Audio Routing

**Goal:** Route external audio (URLs or desktop apps) through the virtual microphone.

**New tab: "メディア"**

### URL Playback Mode
- URL text input + "再生" (play) button
- Pipeline: `yt-dlp -o - -f bestaudio {url} | ffmpeg -i pipe:0 -f wav -acodec pcm_s16le -ar 24000 -ac 1 pipe:1 | paplay --device {sink_name}`
  - `yt-dlp` extracts audio stream (outputs container format to stdout)
  - `ffmpeg` converts to raw WAV PCM (16-bit signed LE, 24kHz, mono) on stdout
  - `paplay` plays the WAV stream to the virtual microphone sink
- Managed as a chain of child processes; "停止" button kills the process group
- Status display: playing / stopped
- `yt-dlp` and `ffmpeg` presence check on tab open; show install instructions if missing

### Desktop Audio Capture Mode
- List running audio applications via `pactl list sink-inputs`
- Display as selectable list (app name + current output)
- "キャプチャ開始" (start capture) button:
  1. `pactl load-module module-combine-sink sink_name=ZundamonCombined slaves={virtual_sink},{default_sink}` — creates combined sink that outputs to both virtual mic and speakers
  2. `pactl move-sink-input {sink_input_id} {combined_sink}` — redirect app audio to combined sink
- "キャプチャ停止" (stop capture) button:
  1. `pactl move-sink-input {sink_input_id} {original_sink}` — restore app's original output
  2. `pactl unload-module {combined_module_id}` — cleanup combined sink
- User hears the audio (through speakers) AND it goes to virtual mic simultaneously
- "更新" (refresh) button to re-list sink-inputs

**Crash recovery:** On app startup, check for stale `ZundamonCombined` sinks via `pactl list short sinks` and unload them. Similar to existing `VirtualDevice` cleanup, add a `Drop` impl for `DesktopCapture` that restores original sink-input routing and unloads the combine-sink module.

**Playback exclusivity:** Media URL playback is exclusive with TTS and soundboard. Desktop capture is independent (it redirects existing audio, not playing new audio).

**Runtime dependencies:**
- `yt-dlp` — for URL audio extraction
- `ffmpeg` — for audio format conversion to WAV PCM

---

## 5. Audio Effects (Echo)

**Goal:** Apply simple echo effect to TTS audio before playback.

**Mechanism:** Direct WAV sample manipulation after VOICEVOX synthesis, before playback.

**WAV format from VOICEVOX:** 16-bit signed LE, 24000 Hz, mono. The effects module parses the 44-byte WAV header to extract PCM data, applies the effect, and reconstructs valid WAV bytes. The VOICEVOX output format is consistent, but the module should read sample rate and bit depth from the header rather than hardcoding.

**Echo algorithm:**
```
for i in delay_samples..total_samples {
    output[i] = input[i] + input[i - delay_samples] * decay;
}
// Clamp to i16 range to prevent overflow
```

**Settings UI (new section: "音声エフェクト"):**
- Echo enable/disable toggle (default: disabled)
- Delay slider: 50ms–500ms (step 10ms, default 200ms)
- Decay slider: 0.1–0.8 (step 0.05, default 0.4)

**Integration point:** In `process_messages()` when handling `WavReady`, apply effects to WAV bytes before passing to playback. New module `src/audio/effects.rs`.

**Config additions:**
- `echo_enabled: bool` (default: false)
- `echo_delay_ms: u32` (default: 200)
- `echo_decay: f64` (default: 0.4)

---

## Playback Exclusivity Model

All audio sources (TTS, soundboard, media URL) are mutually exclusive. Desktop audio capture is independent.

**Architecture:**
- Add an `Arc<AtomicBool>` named `is_playing` shared between UI thread and playback threads
- Before starting any playback (TTS, soundboard, media), check `is_playing`. If true, either queue the request or show "再生中..." in the UI and ignore
- Playback thread sets `is_playing = true` on start, `is_playing = false` on completion (in a `Drop` guard to handle panics)
- UI polls `is_playing` each frame to update the "再生中..." indicator and disable/enable playback buttons
- For queuing: use `Arc<Mutex<VecDeque<PlaybackRequest>>>` where `PlaybackRequest` is an enum of TTS WAV bytes, soundboard file path, or media stream. A dedicated playback thread drains the queue sequentially

**Simpler initial approach (recommended):** Start with the `AtomicBool` check-and-reject model (no queue). If playback is active, show a brief "再生中..." status and discard the request. Queuing can be added later if needed.

---

## New Dependencies

| Crate | Purpose |
|-------|---------|
| `rosc` | OSC message construction for VRChat chatbox |

| Crate Change | Detail |
|--------------|--------|
| `rodio` | Add features: `mp3`, `vorbis` for soundboard format support |

| Runtime | Purpose |
|---------|---------|
| `yt-dlp` | Audio extraction from URLs |
| `ffmpeg` | Audio format conversion (container → WAV PCM) |

---

## Config Additions Summary

```toml
# OSC
osc_enabled = false
osc_address = "127.0.0.1"
osc_port = 9000

# Soundboard
soundboard_path = "/home/user/.config/zundamon_vrc/sounds/"  # expanded via ProjectDirs

# Echo
echo_enabled = false
echo_delay_ms = 200
echo_decay = 0.4
```

All new fields use `#[serde(default)]` for backward compatibility with existing config files.

---

## File Structure (New/Modified)

```
src/
├── app.rs                  # Add new TtsCommand/UiMessage variants, is_playing AtomicBool
├── config.rs               # Add new config fields
├── audio/
│   ├── effects.rs          # NEW: echo effect processing (WAV header parse + sample manipulation)
│   ├── playback.rs         # Add is_playing guard, soundboard file playback
│   └── virtual_device.rs   # Add combine-sink support for media capture
├── osc.rs                  # NEW: OSC chatbox sender
├── media/
│   ├── mod.rs              # NEW: MediaManager
│   ├── url_player.rs       # NEW: yt-dlp + ffmpeg subprocess pipeline
│   └── desktop_capture.rs  # NEW: PulseAudio sink-input capture with crash recovery
├── tts/
│   ├── mod.rs              # Store VoicevoxEngine handle for dict operations
│   └── voicevox.rs         # Add user dict API methods
└── ui/
    ├── mod.rs              # Add Screen::Soundboard, Screen::Media
    ├── soundboard.rs       # NEW: soundboard tab UI
    ├── media.rs            # NEW: media tab UI
    └── settings.rs         # Add OSC, dictionary, echo, soundboard path sections
```
