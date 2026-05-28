Manual invariant injection after Phase 01:

Show:
1. `.codex/hooks.json` no longer contains null hooks,
2. every hook script exists and self-tests,
3. phase gates are executable or explicitly command-bound,
4. final gate checks required P32 artifacts,
5. seeded violation that the gate catches.

If the gates only describe checks but cannot fail, do not continue.
