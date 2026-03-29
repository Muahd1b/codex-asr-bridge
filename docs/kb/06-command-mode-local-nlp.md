# Local Command Mode and Rewrite Pipeline

Date: 2026-03-29

## Current Scope
Supported rewrite modes:
- `fix_grammar`
- `concise`
- `formal`
- `bulletize`

Trigger flow:
- user selects text in focused app,
- presses `c`,
- pipeline rewrites + replaces selection.

## Missing Integrations for Rewrite Quality
1. Live wrong-word correction on partial/final transcript.
2. User correction dictionary + phrase replacements.
3. Protected token rules (paths, flags, code identifiers).
4. Optional local model final-pass rewrite.

## Recommended Hybrid Strategy
- Real-time stage: deterministic corrections only.
- Final stage: optional model rewrite on finalized utterance.

## Architecture Targets
- `command_parser`: deterministic intent mapping.
- `correction_engine`: dictionary + safe rules.
- `transform_engine`: style rewrites.
- `selection_adapter`: selected text IO + undo snapshot.

## Safety Controls
- Keep one-step undo state per rewrite action.
- Log pre/post hashes in debug mode only.
- Keep model rewrite feature-gated and optional.
