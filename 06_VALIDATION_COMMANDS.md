# 06 — Validation Commands

Run from repo root.

## Required commands

```bash
python3 scripts/fibquant_startup_preflight.py --repo .
cargo fmt --all --check
cargo test -p fib-quant
python3 scripts/fibquant_final_assert.py --repo .
```

## Compatibility smoke commands

These should be run if the environment supports them:

```bash
cargo test -p turbo-quant
cargo test -p semantic-memory --features hnsw
cargo test -p semantic-memory --features hnsw,turbo-quant-codec
```

## Optional doc/eval output checks

```bash
find docs/compression -maxdepth 1 -type f -name 'FIBQUANT_*.md' -print
find target/compression-evals -maxdepth 1 -type f -print 2>/dev/null || true
```

## Failure classification

Codex must classify failures as one of:

- implementation blocker;
- missing dependency;
- external environment;
- pre-existing unrelated failure;
- forbidden-surface violation;
- mathematical conformance failure.

No failure may be hidden behind “probably fine.”
