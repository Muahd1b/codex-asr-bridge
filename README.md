# Codex ASR Switch

Local-only ASR dictation for macOS.

`ASR_Switch` is an injection-only Rust runtime:
- global hotkey microphone capture (fixed to `RIGHT_SHIFT`)
- local Voxtral transcription
- focused-app text injection via macOS accessibility (`System Events`)

No FastAPI/WebSocket bridge and no session-forwarding transport exist in this repo.

## Project Layout

- `tools/session-switcher-tui/` - Rust app + global daemon
- `config/profile.json` - runtime profile (created on first run)
- `scripts/download_model.py` - optional Whisper model download helper
- `docs/` - PRD + KB + analysis docs

## Run

```bash
ASR_Switch
```

From source:

```bash
cd tools/session-switcher-tui
cargo run --release
```

Daemon-only mode:

```bash
ASR_Switch daemon
```

## Runtime Behavior

- Hotkey flow is toggle-based:
  - first key press: start recording
  - second key press: stop, transcribe, inject
- Key release does not stop recording.
- Injection target is the current focused app, constrained by inject mode.

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

- `ASR_PROFILE_PATH`
- `ASR_PROJECT_DIR`
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

## Keybindings (TUI)

- `c`: rewrite selected text in focused app
- `p`: cycle rewrite mode
- `i`: cycle inject mode
- `g`: toggle global PTT daemon
- `r`: reload profile
- `v`: validate Voxtral setup
- `Tab`: switch pane
- `q`: quit

## Missing Integrations (Current Backlog)

1. VAD turn detection integration.
2. True always-on continuous dictation pipeline.
3. Live partial transcript streaming in UI.
4. Live wrong-word correction layer (dictionary + safe autofix).
5. Optional final-pass local rewrite model (post-utterance polish).
6. Injection fallback stack (keystroke + clipboard/paste + retry strategy).
7. App compatibility profiles (Terminal/iTerm/Warp + edge-case behavior rules).
8. Permission health diagnostics/remediation flow.
9. Structured observability (`utterance_id`, stage timings, error classes).
10. Personalization store (dictionary/snippets/boost terms).
11. Developer-aware token protection (paths, flags, case, code tokens).
12. Reliability harness (soak/latency/injection regression tests).
13. Packaging/autostart integration.

## Live Rewrite Strategy

Recommended architecture:
- Real-time path: deterministic corrections (dictionary/rules) on partial text.
- Final path: optional model-based rewrite on finalized utterance only.

This avoids latency and unstable rewrites during active dictation.

## macOS Permissions

Grant to your terminal host app:
- Accessibility
- Input Monitoring
- Microphone

Without Accessibility, injection fails.
