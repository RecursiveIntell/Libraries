# Phase 18 Report - Search, Pool, HNSW, and Semantic Memory Risks

## Scope

- Phase: `Phase 18 search/pool/HNSW hardening`
- Backlog rows: `CLAUDE-F-006` through `CLAUDE-F-012`
- Rows touched: 7
- Final row status: 7 `fixed`, 0 raw `open`

## Changes

- Patched the canonical sibling `semantic-memory` crate instead of creating AiDENs-local search/HNSW truth.
- Added a hard brute-force vector-scan circuit breaker with typed `MemoryError::VectorScanLimitExceeded`, while preserving warning telemetry before the hard stop.
- Hardened recency timestamp parsing to accept SQLite `YYYY-MM-DD HH:MM:SS`, fractional seconds, and RFC3339 timestamps; unparseable timestamps now emit a warning and drop only recency contribution.
- Added pool health counters for reader timeouts, writer poison recovery, and reader poison recovery; reader timeout remains a typed error and is now test-covered under contention.
- Replaced HNSW split key-map locks with one `RwLock<KeyMapState>` covering key-to-id, id-to-key, and deleted-ID state so search snapshots and mutations cannot observe inconsistent map/deletion state.
- Switched HNSW dirty flag and flush epoch coordination to sequentially consistent ordering.
- Made HNSW ID exhaustion explicit: deleted IDs are not silently recycled against an old graph; callers receive an error directing compact/rebuild before more inserts.
- Synced `06_CLAUDE_AUDIT_INTEGRATION.md` statuses with the current matrix.

## Files Changed

- `../semantic-memory/src/error.rs`
- `../semantic-memory/src/hnsw.rs`
- `../semantic-memory/src/pool.rs`
- `../semantic-memory/src/search.rs`
- `06_CLAUDE_AUDIT_INTEGRATION.md`
- `matrices/SUPER_PASS_BACKLOG_1020.csv`
- `matrices/SUPER_PASS_BACKLOG_1020.json`

## Tests Run

- `cargo test --manifest-path ../semantic-memory/Cargo.toml hnsw -- --nocapture`
  - Log: `target/super-pass/audit/phase18-semantic-memory-hnsw-tests.log`
- `cargo test --manifest-path ../semantic-memory/Cargo.toml vector_scan_hard_limit -- --nocapture`
  - Log: `target/super-pass/audit/phase18-semantic-memory-vector-scan-test.log`
- `cargo test --manifest-path ../semantic-memory/Cargo.toml timestamp_parser -- --nocapture`
  - Log: `target/super-pass/audit/phase18-semantic-memory-timestamp-test.log`
- `cargo test --manifest-path ../semantic-memory/Cargo.toml reader_timeout -- --nocapture`
  - Log: `target/super-pass/audit/phase18-semantic-memory-reader-timeout-test.log`
- `cargo test --manifest-path ../semantic-memory/Cargo.toml writer_mutex_poison -- --nocapture`
  - Log: `target/super-pass/audit/phase18-semantic-memory-poison-health-test.log`
- `cargo test --manifest-path ../semantic-memory/Cargo.toml`
  - Log: `target/super-pass/audit/phase18-semantic-memory-cargo-test.log`
- `cargo clippy --manifest-path ../semantic-memory/Cargo.toml --lib -- -D warnings`
  - Log: `target/super-pass/audit/phase18-semantic-memory-clippy-lib.log`
- `cargo fmt --manifest-path ../semantic-memory/Cargo.toml --all --check`
  - Log: `target/super-pass/audit/phase18-semantic-memory-fmt-check.log`
- `python3 scripts/assert_p29_audit_matrix_closure.py --completed-through 18`
  - Log: `target/super-pass/audit/phase18-audit-matrix-closure-through-18.log`
- `cargo fmt --all --check`
  - Log: `target/super-pass/audit/phase18-cargo-fmt-check.log`
- `cargo check --workspace --all-targets`
  - Log: `target/super-pass/audit/phase18-cargo-check-workspace-all-targets.log`
- `cargo clippy --workspace --all-targets -- -D warnings`
  - Log: `target/super-pass/audit/phase18-cargo-clippy-workspace-all-targets.log`
- `cargo test --workspace --all-targets`
  - Log: `target/super-pass/audit/phase18-cargo-test-workspace-all-targets.log`

## Rows Closed

- Fixed: `CLAUDE-F-006`, `CLAUDE-F-007`, `CLAUDE-F-008`, `CLAUDE-F-009`, `CLAUDE-F-010`, `CLAUDE-F-011`, `CLAUDE-F-012`
- Quarantined: none
- Deferred: none
- Open-blocking: none

## Unresolved Risk

- `cargo clippy --manifest-path ../semantic-memory/Cargo.toml --all-targets -- -D warnings` still trips pre-existing `expect_used` lint debt in semantic-memory integration tests; the patched library itself passes `--lib -- -D warnings`, all semantic-memory tests pass, and the AiDENs workspace clippy gate passes.
- HNSW deleted IDs are not reused inside a live graph because that would alias old graph nodes. The safe supported behavior is explicit exhaustion plus compact/rebuild to reclaim ID space.

## Exit Decision

Continue. Phase 18 exit gate passed: HNSW concurrency/ordering tests, vector scan hard-block tests, timestamp parse fallback tests, pool timeout/poison health tests, full semantic-memory tests, matrix closure through Phase 18, and AiDENs broad workspace command bar are green.
