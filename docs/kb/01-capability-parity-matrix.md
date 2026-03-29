# Capability Parity Matrix: WhisperFlow Reference -> Local Injection Runtime

Date: 2026-03-29

## Capability Mapping
| Capability Group | WhisperFlow Reference | Local Target | Gap Level |
|---|---|---|---|
| Dictation UX | Fast dictation across apps | Global daemon + focused-app injection | Medium |
| Quiet speech support | Strong | Depends on model + VAD tuning | Medium |
| Always-on quality | Mature turn segmentation | Not fully integrated yet | High |
| Live correction quality | Backtracking + cleanup | Basic transforms only | High |
| Command mode | Rich rewrite operations | Present, needs expansion | Medium |
| Personalization | Dictionary + snippets | Not integrated yet | High |
| Reliability tooling | Mature operational behavior | Basic logs/tests only | High |
| Privacy defaults | Strong local/privacy controls | Local-first by design | Low |

## Required Scope (This Repo)
Included:
- local ASR,
- focused-app injection,
- command-mode local rewrites,
- deterministic runtime behavior,
- no bridge/session transport.

Out of scope:
- mobile,
- team/compliance features,
- hosted analytics,
- session-forwarding transports.

## Priority Order
P0:
- VAD turn detection.
- Always-on continuous pipeline.
- Injection fallback reliability.

P1:
- Live correction layer.
- Optional final-pass local rewrite model.
- Personalization store.

P2:
- App profiles, observability hardening, packaging/autostart.
