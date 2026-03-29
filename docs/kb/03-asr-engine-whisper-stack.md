# ASR Engine Notes: Voxtral Primary, Whisper Optional

Date: 2026-03-29

## Current Runtime Reality
- Primary runtime ASR is local Voxtral in Rust flow.
- No Python bridge-side ASR path is part of active runtime.

## Whisper Relevance
Whisper integration remains useful for:
- quality benchmarking,
- fallback experimentation,
- optional alternate runtime path.

## Engine Options

### Option A (Current Primary)
Use Voxtral in Rust daemon path.
Pros:
- already integrated,
- stable with current injection flow.
Cons:
- correction and always-on integrations still pending.

### Option B (Future Optional)
Add `whisper-rs` + whisper.cpp-compatible model artifacts.
Pros:
- strong Rust-native ecosystem,
- broad community validation.
Cons:
- model artifact migration and runtime tuning required.

## ASR Module API (Target)
- `load_model(config) -> AsrHandle`
- `transcribe_utterance(pcm_f32_16k_mono) -> TranscriptResult`
- optional `transcribe_partial(...)`

## Performance Guidance
- Keep model warm/resident.
- Bound utterance size via VAD endpointing.
- Use bounded queues between VAD and ASR.
- Track p50/p95 stage latency.

## Decision
- Keep Voxtral as primary runtime now.
- Treat Whisper integration as optional enhancement track.
