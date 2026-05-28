# Test Strategy and Fixture Plan

## Rule

Every open P0/P1 code issue must have a named test or fixture lane.

## Issue-to-test map

### `SCHEMA-001` / `SCHEMA-002`

- generator snapshot tests in `contract-schema-gen`
- schema drift checks against committed `schemas/`
- meta-validation / diff review in CI

### `CONTRACT-001`

- `canonical_bundle_roundtrip_preserves_authoritative_contract`
- `episode_export_builds_canonical_forge_envelope`
- `legacy_split_only_evidence_row_rebuilds_canonical_bundle`
- `promotion_reads_assessment_from_canonical_bundle_not_legacy_split_column`

### `CEA-001`

- negative-evidence update tests
- beta persistence tests
- cross-version migration tests if schema/storage changes

### `CEA-002`

- prediction-before-restart vs prediction-after-restart parity tests
- risk-flag stability fixtures

### `CONTROL-001` / `CONTROL-004`

- receipt round-trip tests
- retry/deadline lineage fixtures
- queue-hop propagation fixtures

### `KERNEL-001`

- `cf_c2_thin_export_degrades_explicitly`
- `cf_c2_mixed_semantics_do_not_hallucinate_hyperedge_membership`
- `runtime_surfaces_thin_export_as_conservative_advisory`

### `KERNEL-002`

- `cf_r1_operator_requires_explicit_stop_rule`
- `cf_r2_instability_terminates_explicitly`
- `runtime_exposes_degraded_kernel_failure_artifacts_without_log_spelunking`

### `KERNEL-003`

- `cf_o1_oracle_parity_on_supported_slice`
- `cf_o2_delta_parity_matches_bounded_recompute`
- `runtime_makes_oracle_parity_downgrade_visible_between_rich_and_thin_batches`

### `TEST-002`

High-priority trust-boundary crates:

- `Primitives/forge-policy`
- `Primitives/sandbox-workspace`
- `Primitives/typed-patch`
- `.parser-lib`

Required depth:

- `relative_path_guard_rejects_parent_escape_shapes`
- `local_patch_fs_rejects_generated_escape_paths`
- `parent_dir_paths_are_always_rejected`
- `strip_think_tags_is_idempotent_for_generated_inputs`
- `Primitives/forge-policy/fuzz/fuzz_targets/policy_surfaces.rs`
- `Primitives/sandbox-workspace/fuzz/fuzz_targets/workspace_fs.rs`
- `Primitives/typed-patch/fuzz/fuzz_targets/validate_patch.rs`
