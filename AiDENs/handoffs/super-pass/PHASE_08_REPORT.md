# Phase 08 Report - Boundary Compiler, Schema, and Repair

Date: 2026-05-07

## Scope

- Backlog selector: `Suggested_Phase` contains `Phase 08` or category is `Boundary compiler, JSON, schema & repair`.
- Rows in scope: 80 (`AHD-0431` through `AHD-0510`).
- Initial status: 80 `open`.
- Final status: 80 `fixed`; no raw `open` rows remain for Phase 08.

## Files Changed

- `crates/aidens-boundary-kit/src/lib.rs`
- `crates/aidens-contracts/src/schema_catalog.rs`
- `crates/aidens-runner/src/provider_tool.rs`
- `crates/aidens-cli/src/tests.rs`
- `crates/aidens-testkit/src/lib.rs`
- `crates/aidens-integration-tests/tests/p28_adversarial_conformance.rs`
- `crates/aidens-integration-tests/tests/phase_09_reference_hostile_tests.rs`
- `matrices/SUPER_PASS_BACKLOG_1020.csv`
- `matrices/SUPER_PASS_BACKLOG_1020.json`
- `handoffs/super-pass/PHASE_08_REPORT.md`

## Implementation

- Made boundary compile/parse defaults fail closed: markdown fence repair and JSON substring extraction are disabled by default.
- Added explicit `permissive_degraded_repair()` policy for parser-fallback paths that intentionally accept degraded repair with receipts.
- Added JSON resource ceilings for byte length, nesting depth, node count, string bytes, array items, and object keys.
- Strengthened duplicate-key scanning with unicode-escape and nested object/array hostile coverage.
- Added schema validation support for local `$ref`, `definitions`/`$defs`, `format`, `anyOf`, `oneOf`, `allOf`, `const`, array item limits, string length, and `additionalProperties` schemas.
- Unsupported schema keywords now fail validation instead of being silently ignored.
- Treatment-critical checks now support JSON Pointer paths, including escaped `/` and `~` segments and array indices.

## Hostile/Semantic Tests Added

In `aidens-boundary-kit`:

- `default_boundary_policy_rejects_repairable_wrappers`
- `unsupported_schema_keyword_fails_closed`
- `supported_schema_array_and_additional_property_object_are_enforced`
- `local_schema_ref_and_format_are_semantic_not_silent`
- `treatment_critical_json_pointer_paths_are_honored`
- `duplicate_key_scanner_handles_unicode_and_nested_objects`
- `resource_ceiling_rejects_excessive_depth_before_acceptance`

In `aidens-cli`:

- `boundary_compile_cli_rejects_unsupported_schema_keywords`

Existing integration hostile tests were updated to opt into degraded repair explicitly where that behavior is intentional.

## Validation

Passed:

- `cargo test -p aidens-boundary-kit`
  - Log: `target/super-pass/audit/phase08-cargo-test-aidens-boundary-kit.log`
- `cargo test -p aidens-cli`
  - Log: `target/super-pass/audit/phase08-cargo-test-aidens-cli.log`
- `cargo test -p aidens-contracts`
  - Log: `target/super-pass/audit/phase08-cargo-test-aidens-contracts.log`
- `cargo test -p aidens-testkit`
  - Log: `target/super-pass/audit/phase08-cargo-test-aidens-testkit.log`
- `cargo test -p aidens-runner`
  - Log: `target/super-pass/audit/phase08-cargo-test-aidens-runner.log`
- `cargo test -p aidens-integration-tests --test p28_adversarial_conformance p28_adversarial_boundary_fixtures_fail_closed`
  - Log: `target/super-pass/audit/phase08-cargo-test-integration-p28-boundary.log`
- `cargo test -p aidens-integration-tests --test phase_09_reference_hostile_tests boundary_repair_hard_fails_unverifiable_treatment_change`
  - Log: `target/super-pass/audit/phase08-cargo-test-integration-phase09-boundary.log`
- `cargo fmt --all --check`
  - Log: `target/super-pass/audit/phase08-cargo-fmt-all-check.log`
- `cargo check --workspace --all-targets`
  - Log: `target/super-pass/audit/phase08-cargo-check-workspace-all-targets.log`
- `cargo clippy --workspace --all-targets -- -D warnings`
  - Log: `target/super-pass/audit/phase08-cargo-clippy-workspace-all-targets.log`
- `cargo test --workspace --all-targets`
  - Log: `target/super-pass/audit/phase08-cargo-test-workspace-all-targets.log`

## Matrix Updates

- `AHD-0431` through `AHD-0510`: `fixed`
- Notes updated with the strict repair defaults, schema validation hardening, resource ceilings, JSON Pointer treatment integrity, and audit-log evidence.

## Exit Gate

Phase 08 gate result: `pass`

- Unsupported schema semantics are no longer silently ignored.
- Repairable wrappers are rejected by default and require explicit degraded repair policy.
- Duplicate-key hostile fixtures pass.
- Treatment-critical JSON Pointer paths hard-fail repaired material boundaries.
- Full workspace fmt, check, test, and clippy command bar passed.
- No raw `open` rows remain in Phase 08.

## Unresolved Risk

- Full JSON Schema is not claimed. The supported subset is explicit and unsupported keywords fail closed.
- No final support label is claimed from this phase.

## Decision

`continue`

Phase 08 is fixed and gate-passing.
