# Phase 11 Report - Schema Governance

## Scope

- Phase: `Phase 11 schema governance`
- Backlog rows: `AHD-0686` through `AHD-0735`
- Rows touched: 50
- Final row status: 50 `fixed`, 0 raw `open`

## Changes

- Added explicit schema registry governance status and canonical owner delegation to `contract-schema-gen`.
- Added per-family schema identities, family admission state, and external-family quarantine defaults.
- Added content-addressed schema identities to generated manifest entries.
- Added schema compatibility change classification and major-bump flags.
- Added schema path case-fold collision findings to compatibility reports.
- Hardened the CLI schema checker so case-fold path collisions fail the gate.
- Added semantic tests for schema governance, external quarantine, content-addressed identity, deterministic generation, duplicate-key rejection, unregistered families, drift, and case-fold collisions.

## Files Changed

- `crates/aidens-contracts/src/schema_catalog.rs`
- `crates/aidens-contracts/src/tests.rs`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-cli/src/tests.rs`
- `matrices/SUPER_PASS_BACKLOG_1020.csv`
- `matrices/SUPER_PASS_BACKLOG_1020.json`

## Tests Run

- `cargo test -p aidens-contracts --lib`
  - Log: `target/super-pass/audit/phase11-cargo-test-aidens-contracts-lib.log`
- `cargo test -p aidens-cli schemas_`
  - Log: `target/super-pass/audit/phase11-cargo-test-aidens-cli-schemas.log`
- `cargo fmt --all --check`
  - Log: `target/super-pass/audit/phase11-cargo-fmt-check.log`
- `cargo check --workspace --all-targets`
  - Log: `target/super-pass/audit/phase11-cargo-check-workspace-all-targets.log`
- `cargo clippy --workspace --all-targets -- -D warnings`
  - Log: `target/super-pass/audit/phase11-cargo-clippy-workspace-all-targets.log`
- `cargo test --workspace --all-targets`
  - Log: `target/super-pass/audit/phase11-cargo-test-workspace-all-targets.log`

## Rows Closed

- Fixed: `AHD-0686` through `AHD-0735`
- Quarantined: none
- Deferred: none
- Open-blocking: none

## Unresolved Risk

- AiDENs schema registry remains a local generated display/report index. Canonical schema authority remains delegated to `contract-schema-gen` and relevant owner crates.
- The compatibility model now records major-bump semantics and hostile path collisions, but full cross-major migration execution remains limited to existing migration/backfill receipt surfaces.

## Exit Decision

Continue. Phase 11 exit gate passed with no raw open rows in scope and broad workspace command bar green.
