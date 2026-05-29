# P25 Operator Quickstart

## Minimal operator workflow

1. Open Codex in the AiDENs workspace.
2. Paste `prompts/P25_OPERATOR_PASTE_FIRST.md`.
3. Attach or paste this full P25 docset.
4. Paste `P25_CODEX_RUN_PROMPT.md`.
5. Let Codex execute Phase 00 and Phase 01 only.
6. When it stops, inspect report and paste the matching gate injection.
7. Repeat for the every-other-phase gates.

## Do not allow

- continuing without a phase report,
- continuing without gate injection after configured gates,
- additional z.py feature creep,
- V10 implementation,
- fake cloud/autonomy claims.

## Fast intervention if Codex drifts

Paste:

```text
STOP. You crossed or are approaching a P25 phase gate. Emit the phase report, changed files, commands, invariant validation, unresolved risks, and wait for my injection. Do not continue.
```
