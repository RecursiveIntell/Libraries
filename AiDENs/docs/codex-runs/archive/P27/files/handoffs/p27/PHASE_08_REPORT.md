# P27 Phase Report

## Phase

- Phase ID: 08
- Phase title: Durable run receipt store v0
- Date: 2026-05-05T00:32:11Z

## Scope

- Intended work: add a filesystem-backed store for `AiDENsRunBundleV3` and prove a CLI can inspect a run bundle after reopening it from disk.
- Issue IDs in scope: `P27-018`, `P27-019`.
- Explicit non-goals: no provider/Ollama E2E hardening, no cloud-provider support, no canonical memory or verification truth store, no new canonical receipt semantics.

## Files inspected

- `prompts/phases/P27_PHASE_08_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_08_BEFORE_PHASE_09.md`
- `P27_PHASE_PLAN.md`
- `P27_MASTER_ISSUE_MATRIX.md`
- `STATUS.md`
- `crates/aidens-receipts/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-integration-tests/Cargo.toml`

## Files changed

- `STATUS.md`
- `crates/aidens-receipts/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-integration-tests/tests/phase_08_run_bundle_store.rs`
- `handoffs/p27/PHASE_08_REPORT.md`

## Changes made

- Added `RunBundleStoreConfig`, `RunBundleStore`, `RunBundleStoreRecord`, and `RunBundleStoreInspection` to `aidens-receipts`.
- The store persists only `AiDENsRunBundleV3` local operator evidence under `receipts/run-bundles/<run-id>/run-bundle.json`.
- The store appends `receipts/run-bundles/index.ndjson` records with `artifact_kind`, `ownership`, `support_tier`, `semantic_status`, content digest, canonical event-log path, and known limits.
- `agent run` now writes the normal output `run-bundle.json` and also stores a durable copy under the receipt root.
- `inspect-run` and `agent inspect` can inspect either a run output directory or a receipt-store root containing exactly one persisted run bundle.
- Added an integration test proving a run writes `AiDENsRunBundleV3`, reopens the receipt-store root, verifies the event-log digest, and reads the store record.
- Updated `STATUS.md` to close `P27-018` and partially close `P27-019`.

## Commands run

| Command | Result | Log |
|---|---|---|
| `cargo metadata --no-deps --format-version 1` | pass | `target/p27/audit/phase08_cargo_metadata_before.json` |
| `cargo fmt --check` before formatting | failed with formatting diffs only | `target/p27/audit/phase08_cargo_fmt_check_before.log` |
| `cargo fmt` | pass | `target/p27/audit/phase08_cargo_fmt.log` |
| `cargo fmt --check` after edits | pass | `target/p27/audit/phase08_cargo_fmt_check_after_integration.log` |
| `cargo test -p aidens-receipts` | pass | `target/p27/audit/phase08_cargo_test_aidens_receipts.log` |
| `cargo test -p aidens-cli agent_run_persists_v3_bundle_in_receipt_store_and_inspects_after_restart` | pass after correcting expected degraded label | `target/p27/audit/phase08_cargo_test_cli_bundle_store.log` |
| `cargo test -p aidens-integration-tests phase_08_run_bundle_store_survives_cli_reopen` | pass | `target/p27/audit/phase08_cargo_test_integration_run_bundle_store.log` |
| `cargo check -p aidens-receipts -p aidens-cli` | pass | `target/p27/audit/phase08_cargo_check_receipts_cli.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass | `target/p27/audit/phase08_verify_current_skip_cargo_final.log` |
| `python3 scripts/assert_support_claims.py .` | pass | `target/p27/audit/phase08_assert_support_claims_final.log` |
| `python3 scripts/assert_p27_current_run_truth.py .` | pass | `target/p27/audit/phase08_assert_p27_current_run_truth_final.log` |
| `python3 scripts/assert_p27_agents_md_current.py .` | pass | `target/p27/audit/phase08_assert_p27_agents_current.log` |
| CLI smoke: `agent new`, `agent run`, `agent inspect --run <receipt-store-root>` | pass; inspect report loaded persisted store bundle | `target/p27/audit/phase08_cli_inspect_from_receipt_store.json` |

One obsolete command probe, `python3 scripts/assert_current_run_truth.py .`, failed because that historical assertion still expects P26. It was not used as a Phase 08 validation gate; the P27-specific assertion passed.

## Evidence emitted

- `target/p27/audit/phase08_cargo_test_integration_run_bundle_store.log`
- `target/p27/audit/phase08_cli_inspect_from_receipt_store.json`
- `target/p27/audit/phase08_cargo_test_aidens_receipts.log`
- `target/p27/audit/phase08_cargo_test_cli_bundle_store.log`
- `target/p27/audit/phase08_cargo_check_receipts_cli.log`
- `target/p27/audit/phase08_verify_current_skip_cargo_final.log`
- `target/p27/audit/phase08_assert_support_claims_final.log`
- `target/p27/audit/phase08_audit_file_list.txt`

## 11A semantic impact

- Exact/approx labels touched: added `semantic_status` to `RunBundleStoreRecord`.
- Degradation labels touched: store records emit `degraded_exact_check` when the source `AiDENsRunBundleV3` failure taxonomy is degraded, otherwise `exact_check`.
- Support labels touched: no support profile claim was widened. The store records the source bundle's existing support tier for inspection.
- Proof/check hooks added: persisted bundle records include content digests and are reopened in integration and CLI smoke tests.

## Support profile impact

- No support-tier claim changed in `SUPPORT_PROFILE.md`.
- `STATUS.md` now records evidence-backed Phase 08 closure for durable local run-bundle storage.
- The CLI smoke output shows an existing `supported-local` AgentSpec bundle stored with `semantic_status=degraded_exact_check` because the run abstained on a permit/tool exposure boundary.

## Canonical-owner impact

- No canonical-owner boundary changed.
- The new store is explicitly AiDENs-local operator evidence. It is not a canonical memory, receipt, trace, or verification truth store.
- Canonical receipt and trace ownership remains delegated to `llm-tool-runtime`, `stack-ids`, `semantic-memory-forge`, and verification sibling crates as already declared by the bundle backpointers.

## Issues closed

- `P27-018`: execution context/run-bundle evidence is now durable under receipt roots and digest-bearing.
- `P27-019`: partially closed for run-bundle inspection from a durable receipt-store root.

## New issues / risks

- `P27-019` remains partially open for broader operator UX beyond `inspect-run`/`agent inspect`.
- `RunBundleStore` currently accepts only `AiDENsRunBundleV3`; older `V2` bundles remain direct-file inspectable but are not promoted into the new store.
- Multi-bundle receipt roots fail closed as ambiguous unless the operator points at a specific bundle/output path.

## Decision

Rationale: A durable local run-bundle store exists, stores only AiDENs-local operator evidence with explicit semantics, and is proven by both integration test and CLI smoke inspection after reopening from disk.

Decision: continue
