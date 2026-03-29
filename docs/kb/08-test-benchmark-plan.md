# Test and Benchmark Plan

Date: 2026-03-29

## Test Layers

### Unit Tests
- transform/rewrite functions,
- correction rule engine,
- inject-mode parsing,
- VAD endpointing logic.

### Integration Tests
- end-to-end utterance -> transcript -> injection,
- target app/mode correctness,
- daemon lifecycle + health transitions.

### Soak Tests
- always-on mode for 30-60 minutes,
- repeated start/stop cycles,
- long-session memory stability.

## Injection Correctness Test
- prepare focused-app scenarios (allowed/disallowed targets),
- send synthetic utterances,
- assert behavior matches inject mode policy,
- assert no hidden fallback behavior.

## Latency Benchmarks
Metrics:
- speech-end -> transcript-ready (p50/p95)
- transcript-ready -> injection-complete (p50/p95)

Target SLOs:
- p95 speech-end -> transcript <= 1200 ms
- p95 transcript -> injection <= 1800 ms

## Failure Injection Cases
- missing model file,
- unavailable microphone,
- accessibility/input-monitoring denied,
- focused app disallowed by mode.

Expected outcomes:
- no panic,
- explicit user-facing remediation hints,
- recover without full process restart.

## Golden Dataset
- clean speech,
- noisy speech,
- whispered speech,
- code-heavy dictation,
- known wrong-word correction set.
