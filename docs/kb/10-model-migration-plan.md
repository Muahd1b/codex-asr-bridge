# Model Plan: Voxtral Primary, Whisper Optional

Date: 2026-03-29

## Current Runtime
- Production path uses local Voxtral in Rust runtime.
- Bridge-side model workflows are removed.

## Optional Whisper Track
Whisper support remains an optional enhancement for benchmarking or alternative runtime experiments.

## Paths
Existing local MLX model:
- `/Users/jonasknppel/DEV/models/whisper-large-v3/mlx-community__whisper-large-v3-mlx/weights.npz`

Potential whisper.cpp-compatible model target:
- `/Users/jonasknppel/DEV/models/whisper-large-v3/whispercpp/`

## Validation Checklist
- Model readable by runtime process.
- Startup load in target budget.
- No crash in repeated transcriptions.
- Measured quality comparison vs Voxtral baseline.

## Rollback Strategy
- Keep Voxtral as primary runtime while any optional Whisper track is unstable.

## Decision Gate
Before implementing Whisper path:
1. confirm runtime adapter choice,
2. confirm artifact location,
3. define benchmark/audio comparison set.
