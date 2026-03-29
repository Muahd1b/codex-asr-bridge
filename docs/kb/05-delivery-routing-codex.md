# Delivery & Routing: Deterministic Focused-App Injection

Date: 2026-03-29

## Problem
Wrong-target insertion can happen when focus, permissions, or fallback behavior are not explicit.

## Delivery Contract
- Target is current focused app.
- Inject mode must be validated before delivery.
- If invalid/disallowed target, block delivery with clear action hint.
- No hidden external/session transport fallback.

## Injection Interface (Current)
- Focused app via System Events.
- AppleScript keystroke injection.
- Chunking with newline preservation.

## Target Routing Algorithm
1. Read focused app.
2. Validate against inject mode.
3. Build bounded chunks.
4. Inject chunk sequence.
5. Emit result log with target app + chunk count.

## Required Enhancements
- Add fallback chain policy (keystroke -> clipboard/paste -> retry path).
- Add app-specific profiles for terminal/editor edge cases.
- Add per-utterance delivery id and result telemetry.

## Failure Handling
- TCC/accessibility denied -> explicit remediation steps.
- Disallowed app -> suggest mode switch or focus change.
- Injection error -> include compact stderr detail.
