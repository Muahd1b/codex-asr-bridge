# Capability Implementation Recipes (Rust)

Date: 2026-03-29
Purpose: Concrete implementation recipes for key missing integrations.

## Recipe 1: Always-On Dictation + VAD
Objective:
- continuous listening with utterance-level endpointing.

Implementation:
1. `cpal` capture callback.
2. frame queue -> VAD classifier.
3. utterance builder for voiced spans.
4. emit utterance on speech end.
5. feed ASR worker.

## Recipe 2: Deterministic Focused-App Injection
Objective:
- predictable target behavior.

Implementation:
1. resolve focused app.
2. enforce inject mode contract.
3. chunk transcript.
4. inject via AppleScript.
5. emit delivery result and timings.

## Recipe 3: Live Wrong-Word Correction
Objective:
- reduce recognition mistakes during and after dictation.

Implementation:
1. correction dictionary (exact replacements).
2. safe phrase rewrite rules.
3. protected-token guard (paths/flags/code ids).
4. apply in partial + final stages.

## Recipe 4: Optional Final-Pass Rewrite Model
Objective:
- improve final readability without harming real-time UX.

Implementation:
1. keep partial path deterministic-only.
2. run local model rewrite on final utterance only.
3. feature-flag and latency-budget guard.
4. provide undo snapshot.

## Recipe 5: App Compatibility Profiles
Objective:
- harden injection across target apps.

Implementation:
- per-app policy for chunk size, newline behavior, and fallback method.
- baseline profiles for Terminal, iTerm, Warp.

## Recipe 6: Observability
Objective:
- fast debugging and measurable quality.

Implementation:
- `utterance_id` per turn,
- stage timings,
- error taxonomy,
- rolling logs + structured output.

## Recipe 7: Recovery UX
Objective:
- recover without restart.

Common actions:
- mic unavailable -> show device guidance,
- permission denied -> show TCC remediation,
- disallowed target app -> suggest mode/focus change,
- model failure -> show expected model path.
