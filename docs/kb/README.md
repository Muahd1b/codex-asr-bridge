# Local WhisperFlow Knowledge Base

Date: 2026-03-29
Scope: Architecture and implementation references for the local-first ASR dictation stack in this workspace.

## Purpose

This KB captures capability targets, system design decisions, implementation recipes, and migration planning for the Rust-first runtime.

## Index

- `01-capability-parity-matrix.md` - target capability matrix
- `02-rust-system-architecture.md` - Rust monolith architecture
- `03-asr-engine-whisper-stack.md` - ASR engine options and runtime tradeoffs
- `04-audio-capture-vad.md` - capture and endpointing strategy
- `05-delivery-routing-codex.md` - deterministic focused-app delivery contract
- `06-command-mode-local-nlp.md` - local rewrite/command-mode behavior
- `07-data-privacy-observability.md` - privacy and observability constraints
- `08-test-benchmark-plan.md` - validation and benchmark planning
- `09-capability-implementation-recipes.md` - implementation recipes
- `10-model-migration-plan.md` - model/runtime migration plan
- `SOURCES.md` - external source list

## Current Baseline

- Primary runtime: Rust TUI + global daemon (`tools/session-switcher-tui`).
- Primary ASR path: local Voxtral via Rust runtime.
- Delivery path: focused-app injection only (no FastAPI/WebSocket bridge path).

## Build Principle

- Local-first processing by default.
- Deterministic delivery behavior with explicit runtime states.
- No hidden fallback behavior.
- Observable and testable state transitions.
