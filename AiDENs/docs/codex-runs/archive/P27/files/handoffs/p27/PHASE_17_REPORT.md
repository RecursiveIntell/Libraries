# P27 Phase 17 Report — 11A Semantic Disclosure Layer

## Scope

Phase 17 addressed `P27-012`: evidence-bearing operator outputs lacked consistent exactness, support-tier, degradation, proof-check, and known-limit labels.

No-go zones observed:

- No full 11A runtime or reference interpreter was introduced.
- No support tier was widened.
- No canonical-owner boundary changed.
- No V11/V12 proof-governed runtime claim was promoted.

## Changes

- Added `semantic_disclosure_value(...)` in `crates/aidens-cli/src/lib.rs` with `semantic_status`, `exactness`, `support_tier`, `degradation`, `proof_checks`, `known_limits`, and fenced `reference_semantics` promotion rules.
- Added semantic disclosure blocks to:
  - `AgentSpecValidationReportV1`
  - `AgentSpecDoctorReportV1`
  - `PlanActVerifyLoopV1OutputDisplay`
  - `AiDENsRunInspectReportV2/V3`
  - generic operator reports using `operator_support_tiers`
- Added CLI test assertions for doctor reports, AgentSpec validation, plan-act-verify output, and run-bundle inspection.
- Added `scripts/assert_p27_semantic_disclosure.py` and wired it into `scripts/p27_verify.sh`.
- Updated `STATUS.md` to close `P27-012` and add the Phase 17 ledger entry.

## Changed Files

- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-cli/src/agent.rs`
- `crates/aidens-cli/src/tests.rs`
- `scripts/assert_p27_semantic_disclosure.py`
- `scripts/p27_verify.sh`
- `STATUS.md`
- `handoffs/p27/PHASE_17_REPORT.md`

## Validation

Command logs are under `target/p27/audit/`.

- `cargo fmt --all -- --check` passed: `target/p27/audit/cargo_fmt_phase17.log`
- `python3 scripts/assert_p27_semantic_disclosure.py .` passed: `target/p27/audit/assert_p27_semantic_disclosure_phase17.log`
- `cargo test -p aidens-cli phase13_agent_validate_schema_checks_and_rejects_duplicate_keys` passed: `target/p27/audit/cargo_test_aidens_cli_phase17_agent_validate.log`
- `cargo test -p aidens-cli agent_run_persists_v3_bundle_in_receipt_store_and_inspects_after_restart` passed: `target/p27/audit/cargo_test_aidens_cli_phase17_run_bundle_semantics.log`
- `cargo test -p aidens-cli doctor_reports_provider_capability_matrix_without_cloud_or_native_overclaims` passed: `target/p27/audit/cargo_test_aidens_cli_phase17_doctor_semantics.log`
- `cargo test -p aidens-cli` passed: `target/p27/audit/cargo_test_aidens_cli_phase17_full.log`
- `cargo test -p aidens-integration-tests phase_09_mock_plan_act_verify_e2e_stores_exact_supported_local_receipt` passed: `target/p27/audit/cargo_test_integration_phase17_provider_e2e.log`
- `cargo test -p aidens-integration-tests phase_08_run_bundle_store_survives_cli_reopen` passed: `target/p27/audit/cargo_test_integration_phase17_run_bundle_store.log`
- `cargo check -p aidens-cli -p aidens-receipts -p aidens-integration-tests` passed: `target/p27/audit/cargo_check_phase17_cli_receipts_integration.log`
- `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` passed: `target/p27/audit/verify_current_phase17_skip_cargo.log`

Initial local guard and fmt-check logs also exist and show pre-correction findings:

- `target/p27/audit/assert_p27_semantic_disclosure_phase17_initial.log`
- `target/p27/audit/cargo_fmt_phase17_initial.log`

Both were corrected before the final validation set above.

## Support-Tier Changes

No support-tier claim changed. Phase 17 added disclosure labels to existing evidence/report surfaces only.

## Canonical Ownership

No canonical-owner boundary changed. New disclosures explicitly preserve AiDENs as a local operator/reporting layer and keep canonical truth delegated to owner crates.

## Exact / Approx / Degradation Labels

Labels changed in these artifacts:

- `AgentSpecValidationReportV1`: now emits `exact_check` or `failed_exact_check`.
- `AgentSpecDoctorReportV1`: now emits `exact_check` or `degraded_exact_check`.
- `PlanActVerifyLoopV1OutputDisplay`: now emits `exact_check` or `degraded_exact_check`.
- `AiDENsRunInspectReportV2/V3`: now emits `exact_check`, `degraded_exact_check`, or `failed_exact_check`.
- Generic support-tier operator reports: now emit `display_only`.

## Quarantine

No issues quarantined.

## Decision

Continue to Phase 18.
