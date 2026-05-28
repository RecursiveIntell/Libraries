# Exact file touch map — v25

This map is the repo-facing list of files changed relative to the clean March 16 snapshot, excluding the `libraries-source/` mirror copy.

## Root truth and current entry points
- `00_START_HERE.md`
- `24_V25_SUPERSESSION_AND_CONSTITUTIONAL_CHANGE_NOTE_20260317.md`
- `CANONICAL_STACK_SPEC_V25_EFFECTIVE_CONSTITUTION_PROFILE_COMPOSITION_AND_OBLIGATION_FOLDING_RUNTIME.md`
- `CANONICAL_STACK_SPEC_V26_ADVISORY_CONSTITUTIONAL_SEARCH_MINIMAL_EXCEPTION_SYNTHESIS_AND_POLICY_COUNTERFACTUAL_RUNTIME.md`
- `README.md`

## Repo-facing v25 docs and plan
- `docs/v25/CURRENT_CODE_SNAPSHOT_NOTES_20260317.md`
- `docs/v25/FILE_CREATION_BACKLOG.md`
- `docs/v25/MASTER_ISSUE_MATRIX.md`
- `docs/v25/PER_CRATE_APPLY_PLAN.md`
- `docs/v25/README.md`
- `docs/v25/RELEASE_BAR_AND_ACCEPTANCE.md`
- `docs/v25/REPO_GAP_REPORT_20260317.md`
- `docs/v25/RISK_REGISTER.md`
- `docs/v25/SCHEMA_REGISTRY_AND_COMPATIBILITY_PLAN.md`
- `docs/v25/TEST_AND_CONFORMANCE_PLAN.md`
- `plans/v25-effective-constitution.execplan.md`

## Core code and workspace wiring
- `Cargo.toml`
- `contract-schema-gen/Cargo.toml`
- `contract-schema-gen/src/lib.rs`
- `knowledge-runtime/src/lib.rs`
- `knowledge-runtime/src/views.rs`
- `knowledge-runtime/tests/views_v25.rs`
- `profile-runtime/Cargo.toml`
- `profile-runtime/README.md`
- `profile-runtime/src/adapters.rs`
- `profile-runtime/src/applicability.rs`
- `profile-runtime/src/compose.rs`
- `profile-runtime/src/constitution.rs`
- `profile-runtime/src/exception.rs`
- `profile-runtime/src/lib.rs`
- `profile-runtime/src/profile_set.rs`
- `profile-runtime/src/rules.rs`
- `profile-runtime/tests/example_roundtrip.rs`
- `profile-runtime/tests/fixture_conformance.rs`
- `profile-runtime/tests/fixture_manifest.rs`
- `profile-runtime/tests/reference_composition.rs`
- `stack-ids/src/ids.rs`

## Schemas, examples, and governed fixtures
- `contracts/fixtures/v25/README.md`
- `contracts/fixtures/v25/blocked_locality_without_exception.bundle.json`
- `contracts/fixtures/v25/continuity_incident_mode_diff.bundle.json`
- `contracts/fixtures/v25/delegation_break_glass_depth.bundle.json`
- `contracts/fixtures/v25/disclosure_conflict.bundle.json`
- `contracts/fixtures/v25/locality_exception_admitted.bundle.json`
- `contracts/fixtures/v25/manifest.json`
- `contracts/fixtures/v25/policy_impact_diff.bundle.json`
- `contracts/fixtures/v25/release_readiness_blocked.bundle.json`
- `contracts/fixtures/v25/vendor_translation_caveat.bundle.json`
- `contracts/schemas/v25/manifest.json`
- `examples/applicability-context-v1.example.json`
- `examples/compiled-obligation-runtime-view-v1.example.json`
- `examples/compiled-obligation-set-v1.example.json`
- `examples/composition-conflict-runtime-view-v1.example.json`
- `examples/composition-conflict-set-v1.example.json`
- `examples/composition-receipt-v1.example.json`
- `examples/composition-rule-set-v1.example.json`
- `examples/continuity-policy-profile-v1.example.json`
- `examples/delegation-policy-profile-v1.example.json`
- `examples/effect-policy-profile-v1.example.json`
- `examples/effective-constitution-v1.example.json`
- `examples/effective-constitution-view-v1.example.json`
- `examples/policy-impact-diff-runtime-view-v1.example.json`
- `examples/policy-impact-diff-v1.example.json`
- `examples/profile-exception-bundle-v1.example.json`
- `examples/profile-set-v1.example.json`
- `examples/release-policy-profile-v1.example.json`
- `schemas/applicability-context-v1.schema.json`
- `schemas/compiled-obligation-set-v1.schema.json`
- `schemas/composition-conflict-set-v1.schema.json`
- `schemas/composition-receipt-v1.schema.json`
- `schemas/composition-rule-set-v1.schema.json`
- `schemas/continuity-policy-profile-v1.schema.json`
- `schemas/delegation-policy-profile-v1.schema.json`
- `schemas/effect-policy-profile-v1.schema.json`
- `schemas/effective-constitution-v1.schema.json`
- `schemas/policy-impact-diff-v1.schema.json`
- `schemas/profile-exception-bundle-v1.schema.json`
- `schemas/profile-set-v1.schema.json`
- `schemas/release-policy-profile-v1.schema.json`

## Apply/conformance/scripts
- `apply/v25/APPLY_SEQUENCE.md`
- `apply/v25/CHANGED_FILES.txt`
- `apply/v25/GENERATE_AND_CHECK_SCHEMAS.sh`
- `apply/v25/IMPLEMENTATION_STATUS.md`
- `apply/v25/PATCH_INDEX.md`
- `apply/v25/README.md`
- `apply/v25/SYNC_LIBRARIES_SOURCE_MIRROR.sh`
- `apply/v25/VERIFY_V25_SURFACE.sh`
- `conformance/v25/README.md`
- `conformance/v25/manifest.json`
- `scripts/check_v25_json_surface.py`
- `scripts/check_v25_repo_truth.sh`
- `scripts/run_v25_local_checks.sh`

## Tests
- `profile-runtime/tests/example_roundtrip.rs`
- `profile-runtime/tests/fixture_conformance.rs`
- `profile-runtime/tests/fixture_manifest.rs`
- `profile-runtime/tests/reference_composition.rs`
- `verification-policy/tests/policy_profile_example_roundtrip.rs`

## Mirror note

- `libraries-source/` is expected to be synchronized from the active repo root after these touches land.
