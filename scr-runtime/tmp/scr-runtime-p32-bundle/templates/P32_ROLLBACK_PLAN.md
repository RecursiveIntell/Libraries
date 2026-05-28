# P32 rollback plan

## Safe rollback command

```bash
git diff > /tmp/scr-runtime-p32.patch
git checkout -- .
git clean -fd
```

Do not run destructive rollback if untracked operator files exist. First inspect:

```bash
git status --short
git diff --name-only
find . -maxdepth 2 -type f -name 'P32_*' -print
```

## Partial rollback

List files grouped by phase.

## Quarantine plan

If only part of the pass is bad, move uncertain docs/fixtures to:

```text
docs/codex-runs/archive/P32-quarantine/
```

and revert code to last passing commit.
