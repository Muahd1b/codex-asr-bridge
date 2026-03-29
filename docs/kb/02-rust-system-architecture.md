# Rust System Architecture (Target)

Date: 2026-03-29

## Architectural Style
- Single Rust workspace, event-driven runtime.
- Explicit stage/state transitions.
- Separation of capture, ASR, processing, injection, and UI.

## Current Code Layout
- Single crate at `tools/session-switcher-tui`.
- Module split in `src/`:
  - `audio.rs`, `asr.rs`, `transform.rs`, `inject.rs`, `daemon.rs`, `ui/*`, `config.rs`.
- This is the active implementation baseline.

## Proposed Workspace Layout
- `crates/tui-app` - ratatui/crossterm UI.
- `crates/audio-capture` - microphone stream handling.
- `crates/vad` - voice activity detection and turn endpointing.
- `crates/asr-engine` - Voxtral/Whisper adapter layer.
- `crates/processor` - cleanup, correction, rewrite pipeline.
- `crates/injector` - focused-app injection + fallback chain.
- `crates/telemetry` - structured logs/metrics.
- `crates/config` - profile schema + migrations.

## Runtime State Machine
- `Idle`
- `Listening`
- `SpeechDetected`
- `Transcribing`
- `PostProcessing`
- `Injecting`
- `Injected` / `Error`

Every transition should include `utterance_id` and timestamps.

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
- `InjectionResult`
  - `target_app`
  - `mode`
  - `ok`
  - `stderr_excerpt`

## Concurrency Model
- UI loop: sync draw + key handling.
- Workers: async tasks/channels for capture -> vad -> asr -> processor -> injector.
- Bounded channels for backpressure and explicit overflow behavior.

## Failure Domains
- Mic capture unavailable.
- ASR runtime/model failure.
- Injection permission denial.
- Focused app not allowed by mode.
- UI/render non-fatal issues.

Each failure must map to a recovery hint in UI/runtime logs.
