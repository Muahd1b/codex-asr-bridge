# Test and Benchmark Plan

Date: 2026-03-29

## Test Layers

### Unit Tests
- command parsing and transforms.
- inject-mode parsing/validation.
- VAD endpointing logic.

### Integration Tests
- end-to-end utterance -> transcript -> delivery.
- focused-app target/mode correctness.
- daemon lifecycle start/stop and health transitions.

### Soak Tests
- always-on mode for 30-60 minutes.
- repeated start/stop cycles for capture and server.

## Routing Correctness Test
- prepare focused-app scenarios (Terminal/iTerm/Warp + disallowed apps).
- send 100 synthetic utterances.
- assert deliveries obey inject mode and never use hidden fallback behavior.

## Latency Benchmarks
Metrics:
- speech-end -> transcript-ready (p50/p95)
- transcript-ready -> delivery-complete (p50/p95)

Target SLOs:
- p95 speech-end -> transcript <= 1200 ms
- p95 transcript -> delivery <= 1800 ms

## Failure Injection
- missing model file.
- unavailable microphone.
- accessibility/input-monitoring denied.
- focused app disallowed by inject mode.

Expected outcome:
- no panic,
- clear user-facing error,
- recover without full process restart.

## Golden Audio Dataset
Build local fixtures:
- short clean speech
- noisy speech
- whispered speech
- code-heavy dictation samples

Use fixtures in regression runs to track WER proxy and correction rate.
