# PHASE 01 REPORT — Receipt/log durability

## Scope

- Backlog rows selected: 75 rows, `AHD-0286` through `AHD-0360`, all with `Suggested_Phase = Phase 01 receipt/log durability`.
- Files/crates touched: `crates/aidens-receipts/src/lib.rs`, `crates/aidens-cli/src/agent.rs`, `matrices/SUPER_PASS_BACKLOG_1020.csv`, `matrices/SUPER_PASS_BACKLOG_1020.json`.
- Non-goals: canonical receipt semantics remain delegated to owner crates; this phase only hardens AiDENs-local persistence, display ordering, and matrix evidence.

## Changes made

| Area | Files | Summary |
|---|---|---|
| Receipt log durability | `crates/aidens-receipts/src/lib.rs` | Added exclusive lock-file discipline around canonical event log append, moved sequence/digest-chain computation under the lock, rejected duplicate receipt IDs, flushed and `sync_all`ed append records. |
| Corruption handling | `crates/aidens-receipts/src/lib.rs` | Malformed NDJSON records are now written to a queryable quarantine file instead of making the whole readable history fail. |
| Bundle store durability | `crates/aidens-receipts/src/lib.rs` | Bundle files are written through an atomic temp-file/rename path; bundle-store index appends are fsynced; bundle files are removed if index persistence fails. |
| No final output before durable receipt | `crates/aidens-cli/src/agent.rs` | `agent run` now writes the durable `RunBundleStore` receipt before writing `run-bundle.json`, `run-bundle-store-record.json`, and `final.txt`. |
| Semantic hostile tests | `crates/aidens-receipts/src/lib.rs` | Added concurrent append, corrupt trailing line quarantine, and duplicate receipt ID tests. |

## Tests/commands run

| Command | Result | Evidence/log path |
|---|---|---|
| `cargo test -p aidens-receipts` | pass | `target/super-pass/audit/phase01-cargo-test-aidens-receipts.log` |
| `cargo test -p aidens-cli agent_run_persists_v3_bundle_in_receipt_store_and_inspects_after_restart` | pass | `target/super-pass/audit/phase01-cargo-test-aidens-cli-agent-run.log` |
| `cargo check -p aidens-receipts -p aidens-cli` | pass | `target/super-pass/audit/phase01-cargo-check-receipts-cli.log` |

## Issue matrix updates

| Status | Count | IDs |
|---|---:|---|
| fixed | 75 | `AHD-0286` through `AHD-0360` |
| quarantined | 0 |  |
| deferred | 0 |  |
| superseded | 0 |  |
| open-blocking | 0 |  |

## Gate result

- Phase gate: Receipt/done-state gate.
- Result: Pass for the scoped Phase 01 surfaces. Concurrent appends preserve one chain, corrupt lines quarantine, duplicate IDs fail closed, and the CLI final artifact is written after durable run-bundle receipt persistence.
- Remaining risk: Full workspace release labels still require later phase gates and final extracted-package replay.

## Notes for next phase

Phase 02 should focus on sandbox path policy and hostile fixtures. The receipt quarantine path added here can be reused for deny/quarantine records where the sandbox layer needs durable evidence.
