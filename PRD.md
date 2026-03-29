# PRD: Local WhisperFlow-Style Dictation (Injection-Only)

Status: Draft v3
Date: 2026-03-29
Owner: Jonas
Workspace: /Users/jonasknppel/DEV/codex-asr-bridge

## 1. Product Summary
Build a fully local, privacy-first dictation product for macOS that delivers reliable focused-app injection with low-latency transcription and high correction quality.

## 2. Problem Statement
Current runtime works, but still lacks several integrations needed for WhisperFlow-level dictation polish.

Current pain:
- No true VAD-driven turn segmentation.
- No live partial transcript stream in UI.
- No live correction layer for wrong words during dictation.
- Injection reliability/fallback behavior needs deeper app-specific hardening.
- Observability and reliability testing are still lightweight.

## 3. Goals
Primary goals:
- Fully local speech-to-text by default.
- Deterministic focused-app injection behavior.
- Smooth always-on dictation with VAD turn boundaries.
- Strong correction quality for wrong-word cleanup.

Secondary goals:
- Local command-mode rewriting.
- Personal dictionary/snippets.
- Optional final-pass rewrite model (local) for polished final text.

## 4. Non-Goals (v1)
- FastAPI/WebSocket bridge transports.
- Session-forwarding or external connection routing.
- Mobile apps.
- Team billing/admin dashboards.

## 5. Functional Requirements
FR-1 Audio Capture:
- Push-to-talk and always-on modes.
- Reliable interruption and fast stop behavior.

FR-2 ASR:
- Local Voxtral runtime with readiness checks and lock controls.

FR-3 Delivery:
- Focused-app injection only.
- Inject mode guardrails (`terminal_only`, `any_focused`, `auto`).
- Explicit errors for permission/target failures.

FR-4 Text Processing:
- Existing rewrite modes.
- Live correction dictionary/rules for partial/final text.
- Optional final-pass local model rewrite on finalized utterance.

FR-5 Observability:
- Runtime/talk logs with stage-level status and failure reasons.
- `utterance_id` + latency timing for each delivery path.

## 6. Non-Functional Requirements
NFR-1 Privacy:
- No cloud egress by default.

NFR-2 Performance:
- p95 speech-end -> transcript <= 1200 ms target.
- p95 transcript -> injection complete <= 1800 ms target.

NFR-3 Reliability:
- Stable daemon behavior in long-running sessions.
- Recoverable failures without full app restart.

## 7. Missing Integrations (Backlog)
1. VAD turn detection integration.
2. True always-on continuous dictation pipeline.
3. Live partial transcript streaming in UI.
4. Live wrong-word correction layer.
5. Optional final-pass local rewrite model.
6. Injection fallback stack.
7. App compatibility profiles.
8. Permission health diagnostics/remediation flow.
9. Structured observability.
10. Personalization store.
11. Developer-aware token protection.
12. Reliability harness.
13. Packaging/autostart integration.

## 8. Milestones
M1 (P0 Stability):
- Add structured state transitions and failure taxonomy.
- Harden deterministic injection behavior and fallback policy.

M2 (P0 Dictation Quality):
- Add VAD-based turn segmentation.
- Add live partial transcript stream.

M3 (P1 Correction Quality):
- Add live correction dictionary/rule pipeline.
- Add optional final-pass local model rewrite.

M4 (P1 Productivity):
- Expand command mode + personalization store.

M5 (P2 Ops/Polish):
- Reliability test harness + packaging/autostart.

## 9. Acceptance Criteria
- AC-1: 100/100 utterances either inject into allowed target or fail with clear actionable reason.
- AC-2: Always-on mode runs 30 minutes without crash.
- AC-3: p95 speech-end -> transcript <= 1200 ms.
- AC-4: Wrong-word correction rate improves measurably on golden set.
- AC-5: Permission/focus failures are self-diagnosable in UI.

## 10. Immediate Next Step
Implement VAD-based turn segmentation first, then layer live partial streaming + correction pipeline.
