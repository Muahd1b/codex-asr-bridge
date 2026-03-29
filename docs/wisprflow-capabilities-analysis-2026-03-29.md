# Wispr Flow Capability Analysis

Date: 2026-03-29
Scope: Reference capability breakdown used to guide local injection-only ASR product decisions.

## Whisper Capability
- Quiet/low-volume dictation support.
- Integrated into normal dictation experience.

## Core Dictation
- Dictation across many text fields/apps.
- Real-time transcription + insertion UX.
- Multi-language support.

## Live Text Refinement
- Backtracking corrections while speaking.
- Filler cleanup and punctuation.
- List formatting from speech.
- Dictionary/snippet personalization.

## Command Mode
- Rewrite/transform selected text.
- Summarize/translate/expand/tone edits.
- Undo/cancel pathways.

## Developer-Oriented Capabilities
- Strong behavior in terminal/editor workflows.
- Better handling of identifiers/files/technical tokens.
- Context-sensitive insertion behavior.

## Security and Privacy
- Strong privacy/retention controls.
- Compliance posture for enterprise tiers.

## Relevance to This Repository (Current)
This repository now targets:
- local-only runtime,
- focused-app injection,
- no bridge/session transport.

Most relevant capability gaps to close:
1. VAD + always-on turn segmentation.
2. Live partial transcript stream.
3. Live wrong-word correction integration.
4. Optional final-pass local rewrite model.
5. App-specific injection fallback reliability.
6. Structured observability + reliability harness.

## Sources
- https://wisprflow.ai/features
- https://wisprflow.ai/pricing
- https://docs.wisprflow.ai/articles/2772472373-what-is-flow
- https://docs.wisprflow.ai/articles/4816967992-how-to-use-command-mode
- https://docs.wisprflow.ai/articles/6434410694-use-flow-with-cursor-vs-code-and-other-ides
- https://docs.wisprflow.ai/articles/9559327591-flow-plans-and-what-s-included
- https://docs.wisprflow.ai/articles/6274675613-privacy-mode-data-retention
- https://docs.wisprflow.ai/articles/6939510703-compliance-certifications-standards
- https://docs.wisprflow.ai/articles/1922179110-data-security-encryption
- https://docs.wisprflow.ai/articles/5375461355-subprocessors-third-party-security
- https://trust.wispr.ai/
