# Phase 00 Report

Run id: `20260522T045320Z-poly-kv-next`

## Startup preflight

Command: `bash scripts/preflight.sh | tee .codex-runs/20260522T045320Z-poly-kv-next/startup_preflight.md`

Result: pass. `scripts/preflight.sh` exited 0, reported Rust/Cargo/Python versions, found Cargo manifests, validated `docs/POLY_KV_SCHEMA_PROPOSAL.json`, and reported `public claim boundary ok`.

## Existing git state

Command: `git status --short > .codex-runs/20260522T045320Z-poly-kv-next/git_status_before.txt`

Baseline is dirty before this pass. Existing tracked modifications/deletions and untracked files are treated as user-owned. This pass must not revert them unless explicitly requested.

## Source inventory

Commands:

- `find . -maxdepth 4 -type f | sort > .codex-runs/20260522T045320Z-poly-kv-next/source_inventory.txt`
- `find . -name Cargo.toml -print | sort > .codex-runs/20260522T045320Z-poly-kv-next/cargo_manifests.txt`

Workspace manifests found:

- `Cargo.toml`
- `crates/quant-codec-core/Cargo.toml`
- `crates/poly-kv/Cargo.toml`

Existing crate ownership observed:

- `crates/quant-codec-core`: codec/profile IDs, dtype, digest, eval report, and KV shape/slice request primitives.
- `crates/poly-kv`: shared pool build/read semantics, exact fallback, q8 key reference path, raw exact values, manifests, receipts, memory accounting, tests, and Criterion synthetic bench.

## Existing tests inspected

- `crates/quant-codec-core/tests/digest_stability.rs`
- `crates/quant-codec-core/tests/serde_roundtrip.rs`
- `crates/quant-codec-core/tests/shape_validation.rs`
- `crates/poly-kv/tests/deterministic_replay.rs`
- `crates/poly-kv/tests/memory_accounting.rs`
- `crates/poly-kv/tests/receipt_roundtrip.rs`
- `crates/poly-kv/tests/shape_rejection.rs`
- `crates/poly-kv/tests/synthetic_roundtrip.rs`

## Initial implementation gaps

- Shape V2 is not present; `KvTensorShape` has no batch, query-head count, or explicit attention kind.
- `CompressionEvalReceiptV1` values are built but discarded in `PoolBuilder::build_from_blocks`.
- `MemoryAccounting::with_reader_count` multiplies active readers by default scratch when called from `SharedKvPool::memory_accounting`, losing mixed reader scratch budgets.
- `DecodeReceiptV1` does not disclose full-block decode or copy behavior.
- Manifest byte accounting uses a fixed estimate rather than canonical serialized length.
- Python sidecar layout is not present.
- Existing benchmark harness is Criterion-only and does not emit the required run JSON receipts.

## Phase 00 gate

Required artifacts exist:

- `.codex-runs/20260522T045320Z-poly-kv-next/startup_preflight.md`
- `.codex-runs/20260522T045320Z-poly-kv-next/source_inventory.txt`
- `.codex-runs/20260522T045320Z-poly-kv-next/git_status_before.txt`
- `.codex-runs/20260522T045320Z-poly-kv-next/commit_before.txt`
- `.codex-runs/20260522T045320Z-poly-kv-next/cargo_manifests.txt`
- `.codex-runs/20260522T045320Z-poly-kv-next/commands_run.log`

No implementation source files were edited before this report.
