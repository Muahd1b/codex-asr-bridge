# Local Dictation Knowledge Base

Date: 2026-03-29
Scope: Architecture, implementation, and validation references for the local injection-only ASR runtime.

## Current Baseline

- Runtime: Rust TUI + global daemon (`tools/session-switcher-tui`).
- ASR: local Voxtral path.
- Delivery: focused-app injection only.
- Hotkey: fixed `RIGHT_SHIFT`.

## Index

- `01-capability-parity-matrix.md`
- `02-rust-system-architecture.md`
- `03-asr-engine-whisper-stack.md`
- `04-audio-capture-vad.md`
- `05-delivery-routing-codex.md`
- `06-command-mode-local-nlp.md`
- `07-data-privacy-observability.md`
- `08-test-benchmark-plan.md`
- `09-capability-implementation-recipes.md`
- `10-model-migration-plan.md`
- `SOURCES.md`

## Backlog Summary (Missing Integrations)

1. VAD turn detection.
2. True always-on continuous dictation.
3. Live partial transcript streaming.
4. Live wrong-word correction pipeline.
5. Optional final-pass local rewrite model.
6. Injection fallback stack.
7. App compatibility profiles.
8. Permission health diagnostics.
9. Structured observability.
10. Personalization store.
11. Developer-aware token protection.
12. Reliability harness.
13. Packaging/autostart integration.
