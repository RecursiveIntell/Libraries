# SCR-P0A Source Basis

## Target Repository Root

The SCR-P0A target root for this pass is:

```text
/home/sikmindz/Coding/Libraries/scr-runtime
```

This directory currently contains a complete `Cargo.toml`-backed SCR
reference workspace with the SCR implementation crates.

The containing repository root is:

```text
/home/sikmindz/Coding/Libraries
```

The containing repository is an existing Rust workspace. SCR-P0A must remain a
reference kernel in this target directory unless an operator explicitly chooses
to wire it into the containing workspace.

## Workspace Layout

Current SCR-P0A bundle layout:

```text
AGENTS.md
README.md
crates/
docs/
fixtures/
policies/
prompts/
schemas/
scripts/
target/
```

Target implementation layout from `specs/TARGET_REPO_TREE.md`:

```text
crates/
schemas/generated/
policies/
fixtures/audit/
scripts/
docs/
```

Minor structure note: this runtime is nested under a larger workspace, but
the target implementation remains a complete SCR reference kernel implementation.

## Rust Workspace Status

The SCR-P0A target directory has `Cargo.toml` exists in the repository root at Phase 0.

The containing workspace at `/home/sikmindz/Coding/Libraries/Cargo.toml` uses:

- workspace resolver `2`
- workspace lint policy with `unsafe_code = "deny"`
- clippy denial for `todo` and `dbg_macro`
- clippy denial for `unimplemented`
- clippy warning for `expect_used`
- workspace dependencies including `blake3`, `chrono`, `schemars`,
  `serde`, `serde_json`, `thiserror`, and `uuid`

Existing workspace members include `stack-ids`, `contract-schema-gen`,
`verification-policy`, `verification-control`, `authority-delegation`,
`effect-runtime`, `attestation-exchange`, `semantic-memory-forge`, and
`knowledge-runtime`.

## Existing Lint and Test Conventions

Observed containing-workspace commands:

```bash
make gate
make release-lane
cargo check --workspace
cargo fmt <supported-lane package flags> -- --check
cargo clippy <supported-lane package flags> --all-targets --all-features -- -D warnings
cargo test <supported-lane package flags>
```

The SCR-P0A acceptance gates add stricter local checks for this pass:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 scripts/validate_schemas.py
bash scripts/verify_golden_fixtures.sh
bash scripts/assert_no_feut_contamination.sh
bash scripts/assert_no_durable_float_scores.sh
bash scripts/assert_no_naked_decision_booleans.sh
bash scripts/assert_no_shadow_truth.sh
bash scripts/assert_no_llm_or_network_calls.sh
bash scripts/assert_no_unexplained_golden_changes.sh
bash scripts/run_all_checks.sh
```

Phase 0 now contains Rust implementation in the target directory, so no Cargo checks were
run.

## Existing Schema Generation Conventions

The containing workspace already has a canonical schema generator:

- crate: `contract-schema-gen`
- source: Rust types deriving `schemars::JsonSchema`
- generated output: `/home/sikmindz/Coding/Libraries/schemas`
- drift check: `cargo run -p contract-schema-gen -- --check schemas`
- script wrapper: `scripts/check_schema_compat.sh`

SCR-P0A requires generated schemas under `schemas/generated/` for its own Rust
types. The adapter plan is to mirror the existing convention: Rust types are
canonical, generated JSON schemas are checked in, and drift is a CI failure.

## Existing Source-of-Truth Candidates

IDs:

- `stack-ids` states that it is the single source of truth for cross-crate
  identity types.
- It already defines opaque ID families including artifact, receipt, policy,
  permit, schema-bundle, evidence-plan, and audit-policy related IDs.
- It also owns `ContentDigest` and deterministic canonical JSON digest helpers
  using BLAKE3.

Artifacts:

- Artifact ownership is domain split.
- `effect-runtime` owns effect lifecycle artifacts and execution receipts.
- `authority-delegation` owns authority/delegation artifacts and receipts.
- `attestation-exchange` owns attestation envelopes, trust root sets, and
  transparency receipts.
- `verification-control` owns verification/control-plane review artifacts and
  control receipts.
- `semantic-memory-forge` owns several export/evidence/artifact families.

Evidence references:

- `semantic-memory-forge::ExportEvidenceRef` is an opaque audit evidence
  reference with an explicit fetch handle and source authority.
- `assurance-runtime` owns evidence collection planning artifacts.
- SCR-P0A should model local evidence references as adapter-bound opaque refs,
  not dereference or canonicalize evidence itself.

Provenance references:

- `attestation-exchange` owns attestation/provenance envelope semantics.
- `knowledge-runtime::RuntimeQueryProvenanceV1` owns runtime query provenance
  for knowledge queries.
- SCR-P0A is not a provenance verifier and must only carry provenance basis refs
  supplied in its input.

Receipts:

- `verification-control::ControlReceipt` is an existing control-plane receipt
  artifact.
- Domain crates also own their own receipt families.
- SCR-P0A requires its own decision receipt shape. Whether this should be a new
  local receipt type, an adapter over `ControlReceipt`, or both is unresolved at
  Phase 0.

Policies:

- `verification-policy` owns existing policy snapshot, policy decision, and
  execution permit surfaces.
- SCR-P0A requires local TOML policy canonicalization into normalized JSON and a
  canonical policy hash. This can be local to the reference kernel while policy
  identity and permit references remain adapter-bound to existing owner crates.

Schemas:

- `contract-schema-gen` is the containing workspace's canonical schema
  generator.
- SCR-P0A local schemas must be generated from Rust types and checked into
  `schemas/generated/`.

Time and bitemporal fields:

- Existing crates use RFC3339 strings and `chrono` validation in several places.
- `knowledge-runtime` has explicit bitemporal query fields (`valid_as_of` and
  `recorded_as_of`).
- SCR-P0A requires `valid_time_basis` and `recorded_time`; the canonical local
  representation is unresolved at Phase 0 and must not be invented as global
  truth.

Errors:

- Existing crates use crate-local `thiserror` error enums and stable `kind()`
  discriminants.
- SCR-P0A may define crate-local errors only for its own parsing, validation,
  policy, and evaluation failures. It must not replace upstream domain errors.

## Standalone vs Larger Workspace

Phase 0 classification:

- SCR-P0A target directory: standalone bundle and future reference-kernel
  workspace.
- Containing repository: larger Rust workspace with established owner crates.
- Implementation stance: use adapter traits or explicit opaque refs to existing
  owner surfaces; do not add P0A integration into Recall, AiDENs, memory,
  retrieval, tools, or existing runtime execution paths.

## Assumptions

- `stack-ids` remains authoritative for reusable cross-crate opaque IDs and
  content digests.
- `contract-schema-gen` remains the pattern to follow for generated schema
  drift checks, even if SCR-P0A keeps its schemas under `schemas/generated/`.
- P0A may define local reference-kernel policy and decision receipt types only
  when they are explicitly scoped to SCR evaluation receipts and do not claim
  ownership over upstream provenance, artifacts, or execution state.
- Golden fixtures in later phases are local SCR-P0A conformance fixtures, not
  workspace-wide truth fixtures.

## Unresolved Ambiguities

- Whether SCR-P0A decision receipt IDs should reuse an existing `stack-ids`
  receipt ID type or require a new `stack-ids` ID registration.
- Whether SCR-P0A decision receipts should adapt into
  `verification-control::ControlReceipt`, remain a local receipt type, or expose
  both forms.
- Whether `valid_time_basis` should be an RFC3339 string, interval object, or
  adapter enum in Phase 1.
- Whether `recorded_time` should be operator supplied only, or whether CLI
  fixture evaluation may inject a deterministic fixture timestamp.
- Whether SCR-P0A should be a local workspace with four crates immediately, or
  added as members of the containing workspace after the reference kernel is
  complete.

These ambiguities are recorded in `docs/SourceTruthAmbiguityRecord.md`.
