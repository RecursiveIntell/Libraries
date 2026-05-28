# P01 Handoff - Public API honesty, no-op removal, and plan/runtime parity

## Summary

- Status: complete
- Commit/branch: parent git root `/home/sikmindz/Coding/Libraries`, branch `master`
- Date: 2026-04-26
- Scope: P01 only; no P02-P19 implementation was started.

## Files changed

- `crates/aidens-contracts/src/lib.rs`: added P01 honesty, config-apply, and plan/runtime parity artifacts; blocked `MemoryModeV1::Required`.
- `crates/aidens-app-kit/src/lib.rs`: made `from_plan` fail closed without an explicit provider, rejected disabled/unexecutable providers for provider-required plans, preserved custom tool registries, and exposed provider/tool/config-apply state through list/inspect accessors.
- `crates/aidens-runner/src/lib.rs`: added provider route and tool registry observability.
- `crates/aidens-cli/src/lib.rs`: added plan/doctor parity generation, config-apply receipt generation, provider-route doctor section, memory-mode doctor truth, and stricter plan validation.
- `crates/aidens-cli/tests/next_cli_plan_doctor.rs`: added compiled-plan/doctor parity and blocked-mode regression tests.
- `tests/fixtures/p01/*.json`: added golden fixtures for P01 artifacts.
- `STATUS.md`: marked P01 issue disposition and next-pass readiness.
- `handoffs/P01_PUBLIC_API_HONESTY_AND_NOOP_REMOVAL.md`: this handoff.

## Artifacts introduced or changed

- `ApiHonestyReceiptV1`: owner crate `aidens-contracts`; fixture `tests/fixtures/p01/api_honesty_receipt_v1.json`.
- `ConfigApplyReceiptV1`: owner crate `aidens-contracts`; fixture `tests/fixtures/p01/config_apply_receipt_v1.json`.
- `PlanRuntimeParityReportV1`: owner crate `aidens-contracts`; fixture `tests/fixtures/p01/plan_runtime_parity_report_v1.json`.

## Tests added or updated

- `memory_required_is_blocked_until_durable_memory_exists`: proves required memory fails closed before P09.
- `p01_api_honesty_receipt_distinguishes_honored_and_blocked_inputs`: proves P01 receipt policy shape.
- `p01_parity_report_exposes_mismatches`: proves parity reports fail on drift.
- `p01_golden_fixtures_deserialize`: proves P01 fixtures remain loadable.
- `from_plan_requires_explicit_provider_for_provider_required_plan`: regression for provider-unbound no-op.
- `from_plan_rejects_disabled_provider_for_provider_required_plan`: regression for disabled provider promotion.
- `from_plan_honors_bound_provider_and_plan_tools`: proves explicit provider and plan tools are honored.
- `explicit_builder_tool_registry_is_preserved_over_config_bundles`: proves custom tool registry input is not discarded and is observable through list/inspect accessors.
- `plan_validate_compile_and_doctor_accept_explicit_mock_config`: now checks compiled plan, config apply, parity report, and doctor sections agree.
- `plan_validate_rejects_memory_required_until_durable_store_exists`: regression for required memory.
- `plan_validate_rejects_disabled_provider_when_provider_is_required`: regression for disabled provider as invalid for required-provider plans.

## Commands run

```bash
cargo check --workspace
cargo test -p aidens-contracts
cargo test -p aidens-app-kit
cargo test -p aidens-cli
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
```

## Results

- check: passed.
- targeted tests: passed.
- fmt: passed.
- workspace test: passed.
- clippy: passed.
- verify: passed.

## Acceptance gates

- [x] No public method accepts meaningful input and silently discards it.
- [x] `from_plan` cannot produce a disabled-provider runner when `provider_required=true`; it fails with `provider-unbound` unless an explicit executable provider is bound.
- [x] Plan compile output and doctor report agree on provider route, tool exposure, memory mode, and scaffold state via `PlanRuntimeParityReportV1`.

## Blockers / risks

- Blocker: none for P01.
- Residual risk: app/config memory remains non-durable by design; `MemoryModeV1::Required` is blocked until P09 adds durable memory.
- Repository note: this working directory is under parent git root `/home/sikmindz/Coding/Libraries`; `AiDENs` appears as an untracked directory from that parent view.

## Next pass readiness

- Ready for P02: yes.
- Reason: P01 honesty, provider binding, custom tool preservation, memory-required rejection, and plan/doctor parity gates are implemented and verified.
