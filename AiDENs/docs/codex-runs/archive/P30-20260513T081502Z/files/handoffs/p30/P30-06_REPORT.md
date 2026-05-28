# P30-06 Report

## Scope

Phase slice: doc drift, gate drift, package hygiene, and provider capability truth.

Matrix inventory from `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`:

- 11 total P30-06 rows.
- All are P1.
- Categories: `DOC-DRIFT` 1, `GATE-DRIFT` 7, `PACKAGE-HYGIENE` 2, `PROVIDER-CAPABILITY` 1.

Issue IDs addressed:

- `P30-ABSORB-0019`: completion audit source basis is no longer the stale `libraries-source-clean-20260426.zip`; test now asserts `aidens-p30-current-workspace`.
- `P30-ABSORB-0021`: `schemas/artifact_envelope.schema.json` now exists as a generated compatibility alias for the registered AiDENs-local `artifact-envelope/v1.schema.json` schema.
- `P30-ABSORB-0022` through `P30-ABSORB-0026`: historical `scripts/p24_verify.sh` through `scripts/p28_verify.sh` entrypoints now exist as explicit supersession wrappers to `scripts/verify_current.sh`.
- `P30-ABSORB-0027`: `scripts/verify.sh` continues to delegate to `verify_current.sh`, and active shell script references now resolve under `scripts/assert_script_refs_strict.py`.
- `P30-ABSORB-0031`: provider matrix currently marks non-implemented provider boundaries as `BoundaryUnavailable` with non-executable capability booleans; existing provider capability tests passed.

Issue IDs quarantined as remaining debt:

- `P30-ABSORB-0028`: root markdown archive policy still has broad ambiguity/history debt. This phase did not archive or reclassify root Markdown.
- `P30-ABSORB-0217`: package excluded-file absence semantics remain non-obvious in existing package sidecars. This phase did not regenerate or redesign package exclusion reporting.

## Changed Files

- `crates/aidens-contracts/src/schema_catalog.rs`
  - Registered `ArtifactEnvelopeV1` as the generated AiDENs-local `artifact-envelope` schema family.
- `crates/aidens-cli/src/lib.rs`
  - Schema generation now writes `artifact_envelope.schema.json` as a compatibility alias.
  - Schema checking now treats that alias as expected and checks it matches the registered schema content.
- `crates/aidens-cli/src/tests.rs`
  - Completion audit test now rejects the stale 2026-04-26 source basis label.
- `scripts/p24_verify.sh` through `scripts/p28_verify.sh`
  - Added explicit supersession wrappers that delegate to `verify_current.sh`.
- `schemas/artifact-envelope/v1.schema.json`
  - Generated registered artifact-envelope schema.
- `schemas/artifact_envelope.schema.json`
  - Generated compatibility alias required by the hostile audit row.
- `schemas/generated_schema_manifest_v1.json`
  - Regenerated; schema check reports `checked_schema_count=62`.
- `INSTALL_P30_BUNDLE_TO_REPO.sh`
  - Fixed installer guidance to reference `$DEST/scripts/p30_verify.sh` rather than a layout-dependent `AiDENs/scripts/...` path.

## Tests Added Or Updated

Updated:

- `package_completion_audit_reports_deferred_horizon_without_healthy_claims`

Existing provider tests used as evidence:

- `provider_backend_matrix_lists_p02_backends`
- `p20_provider_capability_matrix_matches_executable_truth`
- `p20_provider_fixture_does_not_overclaim_native_or_cloud_support`

## Commands Run

- `cargo run --manifest-path Cargo.toml -p aidens-cli -- schemas generate --out schemas`
  - Result: pass, `generated 62 schema files into schemas`.
- `cargo run --manifest-path Cargo.toml -p aidens-cli -- schemas check --root schemas`
  - Result: pass, `compatible=true`, `checked_schema_count=62`, `artifact_alias_exists=True`.
- `cargo test --manifest-path Cargo.toml -p aidens-cli package_completion_audit_reports_deferred_horizon_without_healthy_claims -- --nocapture`
  - Result: pass, 1 test passed.
- `cargo test --manifest-path Cargo.toml -p aidens-provider-kit provider_backend_matrix_lists_p02_backends -- --nocapture`
  - Result: pass, 1 test passed.
- `cargo test --manifest-path Cargo.toml -p aidens-provider-kit p20_provider -- --nocapture`
  - Result: pass, 2 tests passed.
- `bash -n scripts/p24_verify.sh scripts/p25_verify.sh scripts/p26_verify.sh scripts/p27_verify.sh scripts/p28_verify.sh scripts/verify_current.sh scripts/verify.sh INSTALL_P30_BUNDLE_TO_REPO.sh`
  - Result: pass.
- `for p in schemas/artifact_envelope.schema.json scripts/p24_verify.sh scripts/p25_verify.sh scripts/p26_verify.sh scripts/p27_verify.sh scripts/p28_verify.sh scripts/verify.sh; do test -e "$p" || exit 1; done`
  - Result: pass.
- `python3 scripts/assert_script_refs_strict.py .`
  - Initial result: failed on `INSTALL_P30_BUNDLE_TO_REPO.sh:8` referencing `AiDENs/scripts/p30_verify.sh`.
  - Final result after installer guidance fix: pass, `ok: script references resolve`.
- `cargo check --manifest-path Cargo.toml -p aidens-cli -p aidens-contracts -p aidens-provider-kit --all-targets --locked`
  - Result: pass.
- `cargo fmt --manifest-path Cargo.toml --all -- --check`
  - Result: pass.
- `python3 scripts/p30_guard.py --repo . | tail -n 8`
  - Result: exit 0, `findings=1841 hard=0`.

## Unresolved Risks And Quarantines

- Historical verifier wrappers do not recreate P24-P28 historical semantics. They are explicit supersession entrypoints to the current verifier, not proof that old phase gates still pass under old criteria.
- `schemas/artifact_envelope.schema.json` is an AiDENs-local schema alias, not a canonical stack artifact identity law.
- Root Markdown ambiguity and package excluded-file absence semantics remain open as P30-06 package-hygiene debt.
- Provider capability truth was verified against current tests and matrix entries; this phase did not implement cloud providers.

## Invariant Revalidation Checklist

- Missing historical gate script paths now exist and declare supersession by delegation.
- Active shell script references resolve.
- The artifact envelope schema is generated and schema-check compatible.
- Completion audit does not emit the stale 2026-04-26 source basis label.
- Provider matrix does not claim executable cloud provider support for unavailable boundaries.
- No v11A/v11B compliance claim is made from this phase.

## Proceed Statement

P30-06 can proceed for the gate-drift, schema-drift, source-basis, and provider-capability rows addressed above. `P30-ABSORB-0028` and `P30-ABSORB-0217` remain explicit package-hygiene debt and must limit final release claims.
