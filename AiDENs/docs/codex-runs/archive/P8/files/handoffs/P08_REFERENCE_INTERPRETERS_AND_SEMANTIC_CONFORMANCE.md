# P08 Handoff - Reference Interpreters and Semantic Conformance Harness

## Scope

Implemented P08 only. Later passes remain deferred.

## Files changed

- `crates/aidens-contracts/src/lib.rs`
  - Added `ReferenceCaseV1`, `ReferenceInterpreterReportV1`, `DifferentialConformanceFindingV1`, `GoldenFixtureManifestV1`, and `ReferenceDomainV1`.
  - Registered the four P08 artifact families in the generated schema registry.
  - Added P08 contract constructor and golden fixture deserialization tests.
- `crates/aidens-testkit/src/lib.rs`
  - Replaced the scaffold with dependency-light reference interpreters for plan/config, provider route, tool exposure, permits, boundary repair, receipt lineage, and future temporal-query semantics.
  - Added reference case generation, coverage manifest helpers, and differential comparison reports with human-readable diffs.
- Production conformance tests added in:
  - `crates/aidens-provider-kit/src/lib.rs`
  - `crates/aidens-config/src/lib.rs`
  - `crates/aidens-tool-kit/src/lib.rs`
  - `crates/aidens-permit-kit/src/lib.rs`
  - `crates/aidens-boundary-kit/src/lib.rs`
  - `crates/aidens-receipts/src/lib.rs`
- Added `aidens-testkit` dev-dependencies to the production crates above.
- `tests/fixtures/reference/`
  - Added P08 fixtures for reference cases, interpreter reports, differential findings, and golden fixture coverage.
- `schemas/`
  - Regenerated schemas; the registry now generates 47 schema files.
- Documentation/status/scaffold accounting:
  - `README.md`
  - `STATUS.md`
  - `SOURCE_TOUCH_MAP.md`
  - `ARTIFACT_SCHEMA_REGISTRY.md`
  - `schemas/README.md`
  - `scripts/assert_no_scaffold_promoted.sh`
  - `crates/aidens-cli/src/lib.rs`

## Tests added

- `aidens-testkit`
  - `reference_cases_cover_required_catalogs`
  - `mismatch_report_has_human_readable_diff`
  - `safe_coding_exposure_case_interprets_expected`
- `aidens-contracts`
  - `p08_reference_artifact_constructors_are_typed`
  - `p08_golden_fixtures_deserialize`
- Production/reference conformance tests:
  - provider readiness vs provider-route reference cases
  - default permit decisions vs permit reference cases
  - safe coding tool exposure vs exposure reference case
  - boundary repair outcomes vs repair reference cases
  - durable receipt lineage graph vs lineage reference case
  - config safe default plan semantics vs plan/config reference case

## Commands run

```bash
cargo test -p aidens-contracts -p aidens-testkit
cargo test -p aidens-contracts -p aidens-testkit -p aidens-config -p aidens-permit-kit -p aidens-tool-kit -p aidens-boundary-kit -p aidens-receipts -p aidens-provider-kit
cargo run -p aidens-cli -- schemas generate
cargo run -p aidens-cli -- schemas check
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
bash scripts/assert_no_scaffold_promoted.sh
bash scripts/assert_no_fake_completion.sh
```

All commands passed.

## Acceptance gate notes

- Reference cases cover all P08 provider kinds, `RiskClassV1` variants, `MemoryModeV1` variants, `ReceiptLevelV1` variants, and `ToolLifecycleStateV1` variants.
- Production/reference mismatches fail tests through `DifferentialConformanceFindingV1::human_diff`.
- `aidens-testkit` remains dependency-light: it depends on `aidens-contracts`, `serde`, `serde_json`, and `thiserror`, and does not call production runtime internals.
- `aidens-testkit` is now a partial active crate, not scaffold-only; scaffold checks and doctor output now list 14 remaining scaffold crates.

## Blockers

None.

## Next-pass readiness

P08 is complete and gated. P09 can start on episode memory, bitemporal claim storage, and retrieval honesty.
