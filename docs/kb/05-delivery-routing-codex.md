# Delivery & Routing: Deterministic Focused-App Injection

Date: 2026-03-29

## Problem
Injection failures or hidden fallback behavior can send text to the wrong place.

## Delivery Contract
- Delivery target is the current focused app (subject to configured inject mode).
- If focused app is not allowed for the current mode, block delivery and show actionable error.
- No hidden fallback to session forwarding or bridge transport.

## Injection Interface (current)
- Focused app detected via macOS System Events.
- Keystroke injection performed through AppleScript.
- Chunked insertion with newline preservation.

## Routing Algorithm (Current)
1. Resolve focused app name.
2. Validate against inject mode (`terminal_only`, `any_focused`, `auto`).
3. Split transcript into bounded chunks.
4. Inject chunks to focused app through AppleScript.
5. Emit runtime/talk log result with target app and chunk count.

## Failure Handling
- Accessibility denied: show explicit macOS TCC guidance.
- Focused app not allowed: block and tell user how to switch mode.
- AppleScript failure: show truncated stderr/stdout detail.

## Idempotency / Safety
- Keep one utterance delivery attempt per completed recording.
- Include timing and target app details in runtime logs for debugging.
