# Data, Privacy, and Observability Design

Date: 2026-03-29

## Local Data Model
Store locally:
- runtime profile/config,
- correction dictionary,
- snippets,
- structured logs and metrics,
- optional debug transcript artifacts.

## Privacy Defaults
- Local-only ASR path by default.
- No cloud egress unless explicitly enabled.
- Optional no-retention mode for transcript artifacts.

## Retention Policy
- Default log retention: 7 days.
- Transcript retention: off by default.
- One command to purge local artifacts.

## Observability Stack (Target)
- `tracing` + `tracing-subscriber`.
- Stage metrics:
  - capture latency,
  - vad latency,
  - asr latency,
  - processing/correction latency,
  - injection latency,
  - error counts by class.

## Event Schema (Target)
- `event_type`
- `utterance_id`
- `target_app`
- `stage`
- `status`
- `latency_ms`
- `error_code`
- `timestamp`

## Current Gap
Structured telemetry is still limited and needs dedicated implementation.
