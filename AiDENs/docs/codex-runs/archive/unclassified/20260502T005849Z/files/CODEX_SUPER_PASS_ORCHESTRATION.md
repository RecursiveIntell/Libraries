# Codex Super Pass Orchestration

## Operating mode

Codex should execute passes in strict order. A pass may be split into multiple commits, but it is not complete until its acceptance gates pass and its handoff is written.

## Per-pass loop

1. Read `AGENTS.md`.
2. Read `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and current pass doc.
3. Inspect source before editing.
4. Implement smallest coherent slice of the pass.
5. Add/update tests before claiming behavior.
6. Run universal gate and pass-specific gates.
7. Update issue matrix/status/handoff.
8. If blocked, write blocker with exact file/function/command evidence.

## Handoff format

Each pass must produce a handoff section:

```markdown
## PXX handoff

- Files changed:
- Artifacts introduced:
- Tests added:
- Commands run:
- Command output summary:
- Acceptance gates satisfied:
- Known blockers:
- Next pass readiness:
```

## Failure handling

If a pass cannot complete, Codex must not skip ahead into advanced work. It must either repair the blocker or stop with a precise handoff.
