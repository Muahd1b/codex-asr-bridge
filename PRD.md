# PRD: Local WhisperFlow-Style Dictation (Injection-Only)

Status: Draft v2
Date: 2026-03-29
Owner: Jonas
Workspace: /Users/jonasknppel/dev/codex-asr-bridge

## 1. Product Summary
Build a fully local, privacy-first dictation product for macOS with reliable push-to-talk / always-on voice capture and deterministic focused-app text injection.

## 2. Problem Statement
Current runtime is functional, but not yet at WhisperFlow-level polish and reliability for daily dictation.

Current pain:
- Always-on dictation still needs true VAD turn segmentation.
- Injection reliability varies by focused app and permissions state.
- UX and observability need stronger state clarity and recovery guidance.

## 3. Goals
Primary goals:
- Fully local speech-to-text path by default.
- Deterministic focused-app injection behavior.
- Smooth always-on dictation with VAD turn boundaries.
- One clear control surface (TUI + daemon states + logs).

Secondary goals:
- Local command-mode rewriting for selected text.
- Personal dictionary/snippets.
- Better low-latency post-processing polish.

## 4. Non-Goals (v1)
- FastAPI/WebSocket bridge transports.
- Session-forwarding or external connection routing.
- Mobile apps.
- Team billing/admin dashboards.

## 5. Functional Requirements
FR-1 Audio Capture:
- Push-to-talk and always-on modes.
- Reliable stop/start and interruption handling.

FR-2 ASR:
- Local Voxtral (current) with clear model readiness checks.

FR-3 Delivery:
- Focused-app injection only.
- Inject mode guardrails (`terminal_only`, `any_focused`, `auto`).
- Explicit, actionable errors for permission/target failures.

FR-4 Local Processing:
- Optional punctuation/filler cleanup.
- Local rewrite modes (concise/formal/bulletize/etc).

FR-5 Observability:
- Runtime/talk logs with deterministic state transitions.
- Timing and failure details surfaced in UI.

## 6. Non-Functional Requirements
NFR-1 Privacy:
- No cloud egress by default.

NFR-2 Performance:
- End-of-speech to transcript display p95 <= 1200 ms target.
- End-of-speech to delivery complete p95 <= 1800 ms target.

NFR-3 Reliability:
- Stable daemon behavior in long-running sessions.
- Recoverable error states without full restart.

## 7. Milestones
M1 (P0 Stability):
- Strengthen deterministic injection flow and failure handling.
- Add state-rich logs and health indicators.

M2 (P0/P1 Dictation Quality):
- Add VAD turn segmentation for always-on mode.
- Improve live partial/final transcript feedback.

M3 (P1 Productivity):
- Expand command mode transforms.
- Add dictionary/snippets.

M4 (P2 Polish):
- Packaging/onboarding, advanced shortcuts, tuning controls.

## 8. Acceptance Criteria
- AC-1: 100/100 utterances either inject successfully to allowed focused app or fail with explicit actionable reason.
- AC-2: Always-on mode can run 30 minutes without crash.
- AC-3: p95 end-of-speech -> transcript <= 1200 ms in target environment.
- AC-4: Permission and focus failures are clear and recoverable in UI.

## 9. Risks
- VAD tuning may miss speech or over-segment.
- Accessibility/TCC variance across host apps.
- Latency spikes on larger models.

## 10. Immediate Next Build Step
Implement VAD-based utterance segmentation and wire it into the existing daemon pipeline while preserving current injection guardrails.
