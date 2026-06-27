# Baseline Gates — Research Implementation Pass

Date: 2026-06-26
Repo: /home/sikmindz/Coding/Libraries

## Commands run before feature work

```bash
cargo check -p fib-quant --all-targets 2>&1 | tee /tmp/libraries-fib-quant-check-baseline.log
cargo test -p fib-quant --all-targets 2>&1 | tee /tmp/libraries-fib-quant-test-baseline.log
```

## Results

| Command | Exit | Result |
|---|---:|---|
| `cargo check -p fib-quant --all-targets` | 0 | PASS |
| `cargo test -p fib-quant --all-targets` | 0 | PASS: 64 unit tests + integration tests passed |

## Warnings observed

- Workspace warning: profiles for non-root package `quant-governor` ignored.
- Pre-existing `gpu-backend/src/simd_nearest.rs` unused import warnings for `_mm_add_ps` and `_mm_shuffle_ps`.

## Logs

- /tmp/libraries-fib-quant-check-baseline.log
- /tmp/libraries-fib-quant-test-baseline.log

## Note

Full workspace baseline was not run before starting implementation because the tree is already extremely dirty (1820 status lines) and the user asked to proceed. This receipt establishes the first target crate (`fib-quant`) was green before RoPE-aware work.
