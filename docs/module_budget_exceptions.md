# Module Budget Exceptions

Documented exceptions to the module size hotspot budget.

| Path | Budget (lines) | Reason |
|------|---------------|--------|
| `profile-runtime/src/adapters.rs` | 1800 | Constitutional composition engine — projects all governance profiles into ObligationContributionV1 streams |
| `semantic-memory/src/db.rs` | 1650 | SQLite WAL connection pool and FTS5/BM25 integration — single-file by design |
| `semantic-memory/src/lib.rs` | 1650 | Core store API surface — decomposition would break encapsulation |
| `forge-pilot/src/main_support/mod.rs` | 1900 | CLI argument parsing and command dispatch — single entry point |
| `forge-pilot/src/loop_runner.rs` | 1100 | OODA loop orchestration — linear flow with rich control structures |
| `knowledge-runtime/src/runtime/core.rs` | 1400 | Query pipeline — classification, planning, execution, merge |
