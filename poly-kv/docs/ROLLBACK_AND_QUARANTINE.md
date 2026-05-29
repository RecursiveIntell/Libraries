# Rollback and Quarantine Plan

## Rollback

Before editing `z.py`:

```bash
git status --short
git diff > .codex-runs/<run>/pre_zpy_universal_packager.diff || true
cp z.py .codex-runs/<run>/z.py.before
```

Rollback paths:

```bash
cp .codex-runs/<run>/z.py.before z.py
# or
git checkout -- z.py scripts docs tests
```

Do not delete generated archive manifests. Move failed package artifacts to:

```text
docs/source-packages/archive/<stamp>/failed/
```

with a manifest explaining failure.

## Quarantine

Quarantine rather than include:

- files matching secret-like names unless explicitly allowed;
- unexpected binary artifacts;
- nested archives not produced by current run;
- stale root package sidecars;
- stale Codex control files;
- build caches/output;
- unsafe symlinks or platform collision paths.

Every quarantine action must record:

- original path;
- archived path if moved;
- sha256;
- size;
- mtime;
- reason;
- whether it was included in the final package.
