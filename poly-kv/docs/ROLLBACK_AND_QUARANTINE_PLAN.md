# Rollback and Quarantine Plan

## Before edits

```bash
git status --short
git branch --show-current
git rev-parse HEAD
mkdir -p .codex-runs/$RUN_ID
```

If the repo is dirty, create a dirty-state inventory before edits.

## Rollback commands

For a normal failed pass:

```bash
git status --short
git diff > .codex-runs/$RUN_ID/failed_pass.diff
git restore --worktree --staged .
git clean -fd -- crates/poly-kv-python python docs/generated .codex-runs/$RUN_ID/tmp || true
```

For partial useful work:

```bash
mkdir -p docs/quarantine/$RUN_ID
cp -a <suspect-file-or-dir> docs/quarantine/$RUN_ID/
git restore --worktree --staged <suspect-file-or-dir>
```

## Quarantine triggers

- external adapter API not inspected but implementation attempted;
- schema generated from stale Rust types;
- benchmark script produces numbers without receipts;
- Python fast path silently copies while claiming zero-copy;
- HF adapter works only through undocumented private internals;
- any README performance/compatibility claim lacks local evidence.

## Recovery rule

Do not fix-forward blindly. Either repair locally with tests, quarantine the surface, or report blocker with exact files and failed commands.
