# Audio Capture and VAD Implementation Notes

Date: 2026-03-29

## Capture Layer
Recommended crate:
- `cpal` for microphone streams.

Capture requirements:
- mono normalization,
- resample/convert to 16kHz `f32`,
- non-blocking callback + queue handoff.

## Why VAD Is Missing-Critical
Without VAD turn segmentation:
- always-on mode is incomplete,
- utterance boundaries are weaker,
- latency/quality are harder to optimize.

## VAD Candidates
1. `webrtc-vad`
- lightweight,
- fast integration,
- deterministic behavior.

2. Silero VAD
- better noisy-environment quality,
- higher integration/runtime complexity.

## Recommended Path
- Start with WebRTC VAD.
- Keep interface abstract so Silero can be added later.

## Endpointing Rules (Target)
- Frame size: 20ms.
- Speech start: 3 voiced frames.
- Speech end: 20-30 unvoiced frames.
- Max utterance cap: 15s.
- Min utterance duration: 250ms.

## Data Flow
`cpal callback -> frame queue -> vad -> utterance builder -> asr queue`

## Debug Requirements
- frame energy,
- voiced/unvoiced decisions,
- utterance boundary markers,
- dropped frame count.
