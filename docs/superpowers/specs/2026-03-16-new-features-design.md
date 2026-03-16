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
- Triggered alongside TTS playback — when `WavReady` is received and audio plays, also send OSC message
- Works on Linux with VRChat via Proton (Proton uses host network stack)

**Settings UI (new section: "OSC設定"):**
- Enable/disable toggle (default: disabled)
- OSC destination address (default: `127.0.0.1`)
- OSC port (default: `9000`)

**Dependencies:**
- `rosc` crate for OSC message construction
- `std::net::UdpSocket` for sending (no async needed)

**Integration point:** `src/audio/playback.rs` or `src/app.rs` — after successful WAV playback, send OSC if enabled.

---

## 2. User Dictionary

**Goal:** Register custom word pronunciations via VOICEVOX's built-in dictionary API.

**VOICEVOX API endpoints:**
- `GET /user_dict` — list all registered words
- `POST /user_dict?surface={word}&pronunciation={reading}&accent_type={int}` — register
- `PUT /user_dict/{word_uuid}?surface={word}&pronunciation={reading}&accent_type={int}` — edit
- `DELETE /user_dict/{word_uuid}` — delete

**No local storage needed** — VOICEVOX persists the dictionary server-side.

**Settings UI (new section: "ユーザー辞書"):**
- Table of registered words (表記 | 読み | 削除ボタン)
- Add form: surface text input + pronunciation text input + "追加" button
- Load word list on section expand or VOICEVOX connection

**Implementation:**
- Add methods to `VoicevoxEngine`: `list_user_dict()`, `add_user_dict_word()`, `edit_user_dict_word()`, `delete_user_dict_word()`
- New `TtsCommand` variants: `LoadUserDict`, `AddUserDictWord`, `DeleteUserDictWord`
- Corresponding `UiMessage` variants for responses

---

## 3. Soundboard

**Goal:** One-click playback of local audio files through the virtual microphone.

**New tab: "サウンドボード"**

**Folder-based management:**
- Configurable folder path (default: `~/.config/zundamon_vrc/sounds/`)
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
- `soundboard_path: String` (default: `~/.config/zundamon_vrc/sounds/`)

---

## 4. Media Audio Routing

**Goal:** Route external audio (URLs or desktop apps) through the virtual microphone.

**New tab: "メディア"**

### URL Playback Mode
- URL text input + "再生" (play) button
- Uses `yt-dlp` (runtime dependency) to extract audio stream
- Command: `yt-dlp -o - -f bestaudio {url} | ffplay -nodisp -autoexit -` or pipe raw audio to virtual sink via `paplay`
- Actually: `yt-dlp -o - -f bestaudio {url}` piped to `paplay --device {sink_name} --raw ...` or use ffmpeg to convert to WAV and play via rodio
- "停止" (stop) button to kill subprocess
- Status display: playing / stopped
- `yt-dlp` presence check on tab open; show install instructions if missing

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

**Playback exclusivity:** Media playback is exclusive with TTS and soundboard.

**Runtime dependencies:**
- `yt-dlp` — for URL audio extraction
- `ffmpeg` — may be needed for format conversion (yt-dlp often requires it anyway)

---

## 5. Audio Effects (Echo)

**Goal:** Apply simple echo effect to TTS audio before playback.

**Mechanism:** Direct WAV sample manipulation after VOICEVOX synthesis, before playback.

**Echo algorithm:**
```
for i in delay_samples..total_samples {
    output[i] = input[i] + input[i - delay_samples] * decay;
}
```

**Settings UI (new section: "音声エフェクト"):**
- Echo enable/disable toggle (default: disabled)
- Delay slider: 50ms–500ms (step 10ms, default 200ms)
- Decay slider: 0.1–0.8 (step 0.05, default 0.4)

**Integration point:** In the WAV processing pipeline, after receiving WAV bytes from VOICEVOX and before passing to playback. New module `src/audio/effects.rs`.

**Config additions:**
- `echo_enabled: bool` (default: false)
- `echo_delay_ms: u32` (default: 200)
- `echo_decay: f64` (default: 0.4)

---

## Playback Exclusivity Model

All audio sources (TTS, soundboard, media) are mutually exclusive:
- A playback queue/lock in `AudioManager` ensures only one plays at a time
- If something is playing, new requests wait until current playback finishes
- UI shows "再生中..." indicator when audio is active

---

## New Dependencies

| Crate | Purpose |
|-------|---------|
| `rosc` | OSC message construction for VRChat chatbox |

| Runtime | Purpose |
|---------|---------|
| `yt-dlp` | Audio extraction from URLs |
| `ffmpeg` | Audio format conversion (usually bundled with yt-dlp) |

---

## Config Additions Summary

```toml
# OSC
osc_enabled = false
osc_address = "127.0.0.1"
osc_port = 9000

# Soundboard
soundboard_path = "~/.config/zundamon_vrc/sounds/"

# Echo
echo_enabled = false
echo_delay_ms = 200
echo_decay = 0.4
```

---

## File Structure (New/Modified)

```
src/
├── app.rs                  # Add new TtsCommand/UiMessage variants, playback queue
├── config.rs               # Add new config fields
├── audio/
│   ├── effects.rs          # NEW: echo effect processing
│   ├── playback.rs         # Add playback lock/queue
│   └── virtual_device.rs   # Add combine-sink support for media capture
├── osc.rs                  # NEW: OSC chatbox sender
├── media/
│   ├── mod.rs              # NEW: MediaManager
│   ├── url_player.rs       # NEW: yt-dlp subprocess management
│   └── desktop_capture.rs  # NEW: PulseAudio sink-input capture
├── tts/
│   └── voicevox.rs         # Add user dict API methods
└── ui/
    ├── mod.rs              # Add Screen::Soundboard, Screen::Media
    ├── soundboard.rs       # NEW: soundboard tab UI
    ├── media.rs            # NEW: media tab UI
    └── settings.rs         # Add OSC, dictionary, echo, soundboard path sections
```
