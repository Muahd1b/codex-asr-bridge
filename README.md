# Codex ASR Switch

Local-only ASR dictation for macOS.

`ASR_Switch` now runs as a Rust daemon/TUI workflow only:
- global hotkey microphone capture
- local Voxtral transcription
- focused-app text injection through macOS accessibility (`System Events`)

There is no FastAPI/WebSocket bridge and no session-forwarding connection path in this repository.

## Project Layout

- `tools/session-switcher-tui/` - Rust app + global hotkey daemon
- `config/profile.json` - runtime profile (created on first run)
- `scripts/download_model.py` - optional Whisper model download helper

## Run

```bash
ASR_Switch
```

Or from source:

```bash
cd tools/session-switcher-tui
cargo run --release
```

Daemon-only mode:

```bash
ASR_Switch daemon
```

## Dictation Flow

- Global hotkey is toggle-based:
  - first key press = start recording
  - second key press = stop + transcribe + inject
- Key release does not stop recording.
- Injection target is the currently focused app (respecting inject mode rules in profile).

## Path Resolution Defaults

- Profile path:
  - `ASR_PROFILE_PATH` (exact file)
  - or `ASR_PROJECT_DIR` + `/config/profile.json`
- Voxtral:
  - `ASR_VOXTRAL_BIN`
  - `ASR_VOXTRAL_MODEL_DIR`
  - default root: `~/DEV/voxtral.c`
- Lock files:
  - `ASR_VOXTRAL_LOCK_FILE` (default `/tmp/codex-asr-voxtral.lock`)
  - `ASR_GLOBAL_PTT_LOCK_FILE` (default `/tmp/codex-asr-global-ptt.lock`)

## Runtime Env Vars

- `ASR_VOXTRAL_BIN`
- `ASR_VOXTRAL_MODEL_DIR`
- `ASR_VOXTRAL_TIMEOUT_SEC`
- `ASR_VOXTRAL_EMPTY_RETRIES`
- `ASR_VOXTRAL_LOCK_TIMEOUT_MS`
- `ASR_VOXTRAL_LOCK_STALE_SEC`
- `ASR_VOXTRAL_LOCK_FILE`
- `ASR_GLOBAL_PTT_LOCK_FILE`
- `ASR_FFMPEG_BIN`
- `ASR_LANGUAGE`

## Hotkey

Profile field:
- `config/profile.json` -> `ptt_hotkey`

Current behavior:
- fixed to `RIGHT_SHIFT`
- no hotkey cycling in TUI

## Keybindings (TUI)

- `c`: rewrite selected text in focused app
- `p`: cycle rewrite mode
- `i`: cycle inject mode
- `g`: toggle global PTT daemon
- `r`: reload profile
- `v`: validate Voxtral setup
- `Tab`: switch pane
- `q`: quit

## macOS Permissions

Grant to your terminal app (or host app):
- Accessibility
- Input Monitoring
- Microphone

Without Accessibility, injection will fail.
