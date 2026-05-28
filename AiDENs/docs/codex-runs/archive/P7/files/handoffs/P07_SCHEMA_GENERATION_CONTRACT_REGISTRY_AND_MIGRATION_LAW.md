# P07 Handoff - Schema Generation, Artifact Registry, Compatibility, and Migration Law

## Scope

Implemented P07 only. Later passes remain deferred.

## Files changed

- `crates/aidens-contracts/src/lib.rs`
  - Added `ArtifactFamilyRegistryV1`, `ArtifactFamilyRegistrationV1`, `GeneratedSchemaManifestV1`, `GeneratedSchemaEntryV1`, `GeneratedSchemaDocumentV1`, `SchemaCompatibilityReportV1`, `SchemaCompatibilityCheckV1`, `MigrationPlanV1`, and `BackfillReceiptV1`.
  - Added `ReceiptKindV1::Backfill`.
  - Added the current artifact family registry and generated schema document/manifest helpers.
  - Added P07 tests for registry/manifest constructors, golden fixture readability, and P00-P06 migration/backfill readability.
- `crates/aidens-cli/src/lib.rs`
  - Added `aidens schemas generate`.
  - Added `aidens schemas check`.
  - Added schema drift, missing registered schema, manifest drift, and unregistered family checks.
  - Added CLI tests for deterministic generation and failing compatibility gates.
- `schemas/`
  - Added generated schema files for 43 registered artifact families under `schemas/<family>/vN.schema.json`.
  - Added `schemas/generated_schema_manifest_v1.json`.
  - Updated `schemas/README.md` to define generated schemas versus historical `*.sketch.json` files.
- `tests/fixtures/`
  - Added P07 golden fixtures for the new artifact types.
  - Added missing registry-backed historical fixtures for P00/P03/P05 artifacts used by migration tests.
- Documentation/status:
  - `README.md`
  - `STATUS.md`
  - `SOURCE_TOUCH_MAP.md`
  - `ARTIFACT_SCHEMA_REGISTRY.md`

## Tests added

- `aidens-contracts`
  - `p07_registry_manifest_and_migration_constructors_are_typed`
  - `p07_golden_fixtures_deserialize`
  - `p07_migration_path_keeps_old_fixtures_readable`
- `aidens-cli`
  - `schemas_generate_is_deterministic_and_check_passes`
  - `schemas_check_fails_on_unregistered_artifact_family`
  - `schemas_check_fails_on_same_major_schema_drift`

## Commands run

```bash
cargo check -p aidens-contracts -p aidens-cli
cargo fmt --all
cargo test -p aidens-contracts -p aidens-cli
cargo run -p aidens-cli -- schemas generate
cargo run -p aidens-cli -- schemas check
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
bash scripts/assert_no_fake_completion.sh
bash scripts/assert_no_scaffold_promoted.sh
```

All commands passed.

## Acceptance gate notes

- `cargo run -p aidens-cli -- schemas generate` deterministically writes 43 Rust-owned schema files plus the generated manifest.
- `cargo run -p aidens-cli -- schemas check` passes on the generated tree and fails in tests for unregistered families or same-major schema drift.
- P00-P06 fixture readability is covered by a P07 migration/backfill test with a successful `BackfillReceiptV1`.
- Generated schemas are owned by Rust `JsonSchema` types; historical `*.sketch.json` files are not treated as the compatibility gate.

## Blockers

None.

## Next-pass readiness

P07 is complete and gated. P08 may start from the generated schema registry, manifest, and migration fixture coverage.
