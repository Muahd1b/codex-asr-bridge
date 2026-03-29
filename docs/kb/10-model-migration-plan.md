# Model Migration Plan: MLX Whisper Large v3 -> Rust Runtime

Date: 2026-03-29

## Current Situation
Existing local model is MLX format:
- `/Users/jonasknppel/dev/models/whisper-large-v3/mlx-community__whisper-large-v3-mlx/weights.npz`

Rust-first target engine (`whisper-rs`) expects whisper.cpp-compatible model artifacts.

## Migration Paths

### Path A (recommended)
- Keep current MLX model for fallback verification only.
- Download whisper.cpp-compatible large-v3 model into a new model store.
- Point Rust ASR engine to that model path.

### Path B
- Keep MLX-only and call Python sidecar from Rust.
- Reject for final architecture (not fully Rust runtime).

## Proposed Directory Layout
- `/Users/jonasknppel/dev/models/whisper-large-v3/mlx-community__whisper-large-v3-mlx/` (existing)
- `/Users/jonasknppel/dev/models/whisper-large-v3/whispercpp/` (new)

## Validation Checklist
- Model file present and readable by Rust process.
- Startup model load succeeds in <= 5s warm start target.
- First transcription result matches baseline quality expectations.
- No crash on repeated transcriptions (100-run loop).

## Rollback Plan
- If Rust engine path fails quality/performance, temporarily keep Voxtral runtime as primary while Rust Whisper integration is fixed.

## Decision Gate
Before coding ASR crate, confirm:
1. accepted model runtime path (`whisper-rs`),
2. accepted model artifact location,
3. baseline benchmark audio set for quality checks.
