# Module budget exceptions

This file closes `MOD-001` without pretending the remaining large modules were split in this pass.
They are explicit review-budget exceptions, not hidden green status.

| Path | Current lines | Cap | Why the exception exists now | Next split seam |
| --- | ---: | ---: | --- | --- |
| `profile-runtime/src/adapters.rs` | 1775 | 1800 | Adapter families are still concentrated in one file and were not reopened during the hardening lane. | Split by adapter family and keep serde/schema tests with each family. |
| `semantic-memory/src/db.rs` | 1608 | 1650 | SQLite/query plumbing is still centralized and risky to reopen during closeout. | Split connection, query, and migration helpers into submodules. |
| `semantic-memory/src/lib.rs` | 1599 | 1650 | The crate front door still re-exports too much storage/query surface from one file. | Split public API, query helpers, and import/export seams. |
| `forge-pilot/src/main_support/mod.rs` | 1591 | 1600 | The terminal wrapper remains intentionally concentrated while the release lane stabilizes. | Split CLI parsing, rendering, and TUI worker control. |
| `forge-pilot/src/loop_runner.rs` | 1023 | 1050 | Closed-loop orchestration logic is still in one file, but the current lane only fixed correctness and truth gaps. | Split planning, iteration accounting, and halt/reporting seams. |
| `knowledge-runtime/src/runtime/core.rs` | 1198 | 1250 | Runtime-core orchestration still spans multiple bounded concerns. | Split query, advisory inference, and observation assembly. |

Proof surface:
- `bash scripts/check_hotspot_budgets.sh`
- `make gate`
