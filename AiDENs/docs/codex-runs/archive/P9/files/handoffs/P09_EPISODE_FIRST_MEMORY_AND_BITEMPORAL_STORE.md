# P09 Handoff - Superseded Local Memory DTOs

This historical handoff has been superseded by the canonical-library doctrine:
AiDENs no longer owns local memory/evidence/claim/projection/episode/write/query
DTOs. Current memory code is a thin adapter over Forge, forge-memory-bridge,
semantic-memory, and knowledge-runtime.

## Scope

Implemented P09 only. Later passes remain deferred.

## Files changed

- `crates/aidens-contracts/src/lib.rs`
  - Local memory truth DTOs have been removed; contracts keep only orchestration/config/report surfaces.
- `crates/aidens-memory-kit/Cargo.toml`
- `crates/aidens-memory-kit/src/lib.rs`
  - Provides `CanonicalMemoryAdapter` over Forge exports, forge-memory-bridge, semantic-memory, and knowledge-runtime.
- `crates/aidens-receipts/src/lib.rs`
  - Appends canonical library receipt payloads only.
- `crates/aidens-boundary-kit/src/lib.rs`
  - Added memory claim input schema generation and validation.
- `crates/aidens-config/src/lib.rs`
  - Added `[memory].store_root` configuration.
- `crates/aidens-testkit/src/lib.rs`
  - Updated the reference plan/config interpreter so required memory depends on memory store configuration, not receipt store configuration.
- `crates/aidens-app-kit/src/lib.rs`
  - Enforced required-memory store policy in app config build paths.
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-cli/tests/next_cli_plan_doctor.rs`
  - Removed `aidens-memory-kit` from scaffold-only surfaces.
  - Added doctor/run/plan gating for required memory without `[memory].store_root`.
  - Reported optional memory without a store as degraded, not healthy.
- `scripts/assert_no_scaffold_promoted.sh`
  - Removed `aidens-memory-kit` from the scaffold-only allowlist.
- Documentation and status:
  - `README.md`
  - `STATUS.md`
  - `ARTIFACT_SCHEMA_REGISTRY.md`
  - `SOURCE_TOUCH_MAP.md`
  - `MASTER_ISSUE_MATRIX.md`
  - `passes/P01_PUBLIC_API_HONESTY_AND_NOOP_REMOVAL.md`
- P09 local fixtures were removed because memory/claim/evidence wire shapes are library-owned.
- `schemas/`
  - Regenerated schema output and manifest; P09 now brings the schema registry to 54 generated schema files.

## Tests added

- `aidens-memory-kit`
  - `appends_claim_supersedes_retroactively_and_answers_bitemporal_as_of`
  - `memory_receipts_can_be_written_to_durable_receipt_store`
  - `supersession_requires_existing_claim`
- `aidens-contracts`
  - `p09_memory_artifact_constructors_keep_valid_and_recorded_time_separate`
  - `p09_golden_fixtures_deserialize`
- `aidens-receipts`
  - `durable_store_appends_memory_write_and_query_receipts`
- `aidens-boundary-kit`
  - `memory_claim_input_validation_blocks_missing_bitemporal_fields`
- `aidens-cli`
  - `doctor_fails_memory_required_without_memory_store`
  - `doctor_reports_optional_memory_without_store_as_degraded`
  - `run_fails_memory_required_without_memory_store`
- `crates/aidens-cli/tests/next_cli_plan_doctor.rs`
  - required-memory rejection without a memory store
  - required-memory acceptance with `[memory].store_root`

## Commands run

```bash
cargo check --workspace
cargo run -p aidens-cli -- schemas generate
cargo test -p aidens-contracts p09
cargo test -p aidens-memory-kit
cargo test -p aidens-cli memory_required
cargo test -p aidens-boundary-kit memory_claim
cargo test -p aidens-receipts memory
cargo run -p aidens-cli -- schemas check
cargo fmt --all
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
bash scripts/assert_no_fake_completion.sh .
bash scripts/assert_no_scaffold_promoted.sh .
```

All commands passed.

## Acceptance gate notes

- Claims can be inserted, superseded retroactively, and queried by both valid time and recorded time.
- Supersession appends a new claim and leaves historical claims intact; destructive update is not used.
- `MemoryModeV1::Required` now fails plan/doctor/run paths unless `[memory].store_root` is configured.
- Optional memory without a store is reported as degraded, not healthy.
- `aidens-memory-kit` is no longer scaffold-only; remaining scaffold crates are still listed as disabled/deferred.
- P09 wire-visible artifacts are type-owned, schema-generated, and covered by golden fixture deserialization.

## Blockers

None.

## Next-pass readiness

P09 is complete and gated. P10 can start next; later passes remain out of scope until their turn in `BUILD_ORDER_DAG.md`.
