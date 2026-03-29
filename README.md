# Voxtral Flow Dictation

Local-only ASR dictation for macOS.

`Voxdic` is an injection-only Rust runtime:
- global hotkey microphone capture (fixed to `RIGHT_SHIFT`)
- embedded Voxtral transcription (Rust FFI -> `vox_load`/`vox_stream_*`)
- focused-app text injection via macOS accessibility (`System Events`)
- single-process architecture: TUI + hotkey worker in one app process

No FastAPI/WebSocket bridge and no session-forwarding transport exist in this repo.

## Project Layout

- `tools/voxdic/` - Rust app + global daemon
- `config/profile.json` - runtime profile (created on first run)
- `scripts/download_model.py` - optional Whisper model download helper
- `docs/` - PRD + KB + analysis docs

## Run

```bash
Voxdic
```

From source:

```bash
cd tools/voxdic
cargo run --release
```

Daemon-only mode:

```bash
Voxdic daemon
```

Normal usage is just `Voxdic` (single app). `Voxdic daemon` remains available for standalone debugging.

## Runtime Behavior

- Hotkey flow is toggle-based:
  - first key press: start recording
  - second key press: stop, finalize transcript, inject
- Key release does not stop recording.
- During recording, Voxtral runs as a live stream session and emits partial transcript updates.
- Optional live injection mode can inject partial text into the focused app while recording.
- Injection target is the current focused app, constrained by inject mode.
- Single daemon + single transcribe worker gate (no overlapping utterance jobs).

## Path Resolution Defaults

- Profile path:
  - `ASR_PROFILE_PATH` (exact file)
  - or `ASR_PROJECT_DIR` + `/config/profile.json`
- Voxtral:
  - `ASR_VOXTRAL_MODEL_DIR`
  - default root: `~/DEV/voxtral.c`
- Build-time Voxtral source root:
  - `ASR_VOXTRAL_ROOT` (default `~/DEV/voxtral.c`)
- Lock files:
  - `ASR_GLOBAL_PTT_LOCK_FILE` (default `/tmp/voxdic-global-ptt.lock`)

## Runtime Env Vars

- `ASR_PROFILE_PATH`
- `ASR_PROJECT_DIR`
- `ASR_VOXTRAL_MODEL_DIR`
- `ASR_VOXTRAL_ROOT` (build-time)
- `ASR_VOXTRAL_EMPTY_RETRIES`
- `ASR_VOXTRAL_INTERVAL_SEC`
- `ASR_VOXTRAL_DELAY_MS`
- `ASR_VOXTRAL_FEED_CHUNK`
- `ASR_VOXTRAL_PREWARM_SECONDS`
- `ASR_GLOBAL_PTT_LOCK_FILE`
- `ASR_LANGUAGE`

## Performance Notes (Voxtral on macOS)

- `Voxdic` now initializes Metal explicitly in-process before model load.
- It enforces exactly one embedded engine per process (no second in-process instance).
- It runs a one-time startup prewarm (`ASR_VOXTRAL_PREWARM_SECONDS`) to front-load cache building.
- Model file size (`consolidated.safetensors`) is disk size, not direct RAM usage.
- Activity Monitor process memory will usually be lower than model size because:
  - model tensors are memory-mapped and paged on demand
  - GPU allocations are tracked separately from process resident RAM
- On Metal, memory can climb during first utterances as bf16->f16 weight caches fill. This is cache growth, not a new process/model spawn.
- TUI System panel shows `Backend: metal/cpu-fallback` and `metal_mem` for verification.

## Keybindings (TUI)

- `c`: rewrite selected text in focused app
- `p`: cycle rewrite mode
- `i`: cycle inject mode
- `l`: toggle live injection (`off`/`on`)
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
