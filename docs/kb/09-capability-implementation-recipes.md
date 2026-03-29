# Capability Implementation Recipes (Rust)

Date: 2026-03-29
Purpose: Concrete implementation recipes for each major user-facing capability.

## Recipe 1: Always-On Dictation
Objective:
- Continuous listening with low false triggers and fast utterance emission.

Implementation:
1. Capture microphone PCM via `cpal` callback.
2. Convert frames to mono + 16kHz f32.
3. Feed frames to VAD classifier.
4. Build utterance while voiced.
5. On end-of-speech, send utterance to ASR worker.
6. Emit partial UI status updates immediately.

Required controls:
- Toggle always-on (`a`).
- Hard stop within <= 200 ms.
- Visual state in TUI footer.

## Recipe 2: Deterministic Focused-App Delivery
Objective:
- Never inject transcript into an unintended app.

Implementation:
1. Resolve focused app from System Events.
2. Validate target against inject mode.
3. Split transcript into bounded chunks.
4. Inject chunks through AppleScript.
5. Surface success/failure in talk pane with target app + chunk count.

Guardrails:
- No hidden session-forwarding fallback.
- Block delivery if focused app is disallowed by mode.

## Recipe 3: Transcript Cleanup Pipeline
Objective:
- Improve readability with minimal latency cost.

Pipeline stages:
1. Normalize whitespace and punctuation spacing.
2. Optional filler stripping (`uh`, `um`, `you know`) with configurable aggressiveness.
3. Optional capitalization and sentence boundary cleanup.
4. Dictionary replacements.
5. Snippet expansion.

Constraints:
- Must preserve code-like tokens and file paths.
- Must be reversible (log pre/post text if debug enabled only).

## Recipe 4: Developer-aware Dictation
Objective:
- Better coding prompt quality.

Implementation:
- add lexical detector for:
  - snake_case,
  - camelCase,
  - dotted files,
  - flags/options (`--foo`, `-x`).
- protect backtick sections from cleanup transforms.
- preserve extension-bearing filenames exactly.

Future extension:
- editor context adapter for current file names and symbols.

## Recipe 5: Command Mode (Local)
Objective:
- Voice commands to rewrite selected text locally.

Implementation:
1. Parse command intent from transcript.
2. Obtain selected text from active target context.
3. Apply deterministic transform.
4. Show preview in TUI (optional).
5. Apply and keep one-step undo snapshot.

Supported intents (v1):
- rewrite concise,
- summarize,
- translate,
- bulletize,
- expand.

## Recipe 6: Observability and Debuggability
Objective:
- Diagnose failures quickly without external tooling.

Implementation:
- Structured JSON logs with `tracing`.
- Stage timings and delivery outcomes.
- rotating file logs + in-TUI tail.
- health panel showing:
  - model loaded,
  - mic available,
  - accessibility permissions,
  - injection mode/target readiness.

## Recipe 7: Error Recovery UX
Objective:
- Keep user in flow after errors.

Common errors and action hints:
- mic unavailable -> show input device list shortcut.
- model load fail -> show exact path expected.
- focused app disallowed -> show inject mode switch hint.
- AppleScript/TCC fail -> show permission remediation steps.
