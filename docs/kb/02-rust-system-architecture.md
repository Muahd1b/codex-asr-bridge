# Rust System Architecture (Target)

Date: 2026-03-29

## Architectural Style
- Single Rust workspace with modular crates.
- Event-driven pipeline with explicit state machine.
- Strict separation of capture, inference, processing, delivery, and UI orchestration.

## Proposed Workspace Layout
- `crates/tui-app`: ratatui/crossterm UI and keybindings.
- `crates/audio-capture`: microphone capture and buffering.
- `crates/vad`: VAD segmentation and endpointing.
- `crates/asr-engine`: Whisper runtime adapter(s).
- `crates/processor`: cleanup, dictionary, snippets, style transforms.
- `crates/delivery`: Codex delivery + optional app insertion adapters.
- `crates/session-store`: Codex session discovery and selection state.
- `crates/telemetry`: tracing, metrics, run logs.
- `crates/config`: versioned config/profile schema and migrations.

## Runtime State Machine
- `Idle`
- `Listening`
- `SpeechDetected`
- `Transcribing`
- `PostProcessing`
- `Delivering`
- `Delivered` / `Error`

Transitions must be logged with an `utterance_id` and timestamps.

## Core Data Contracts
- `Utterance`
  - `utterance_id`
  - `audio_start_ts`
  - `audio_end_ts`
  - `pcm_f32_16khz_mono`
- `TranscriptResult`
  - `raw_text`
  - `normalized_text`
  - `latency_ms`
- `DeliveryResult`
  - `target_kind` (`focused_app_inject`)
  - `target_id`
  - `ok`
  - `stderr_excerpt`

## Concurrency Model
- UI thread: synchronous render + key handling (crossterm poll timeout loop).
- Workers: tokio tasks for capture, VAD, ASR, delivery.
- Channels:
  - bounded channels between stages for backpressure.
  - explicit drop policy and error reporting when saturated.

## Failure Domains
- Capture failure (mic unavailable).
- ASR failure (model not loaded / runtime fault).
- Delivery failure (session missing / CLI error).
- UI non-fatal render issues.

Each failure maps to a recovery action in UI and logs.
