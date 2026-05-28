# Exact File Touch Map

This is the **most likely high-value touch set** for the next pass.

## Existing files to update

### Shared infrastructure
- `stack-ids/src/ids.rs`
- `contract-schema-gen/src/lib.rs`
- `kernel-conformance/src/reference_interpreters.rs`
- `kernel-conformance/tests/v16_v20_reference_interpreters.rs`
- `contracts/schemas/v16/manifest.json`
- `contracts/schemas/v17/manifest.json`
- `contracts/schemas/v18/manifest.json`
- `contracts/schemas/v19/manifest.json`
- `contracts/schemas/v20/manifest.json`
- `conformance/v16/README.md`
- `conformance/v17/README.md`
- `conformance/v18/README.md`
- `conformance/v19/README.md`
- `conformance/v20/README.md`

### v16
- `federated-settlement/src/lib.rs`
- `federated-settlement/tests/settlement_slice.rs`

### v17
- `mechanism-runtime/src/lib.rs`
- `mechanism-runtime/tests/mechanism_fit_slice.rs`

### v18
- `discovery-portfolio/src/lib.rs`
- `discovery-portfolio/tests/portfolio_slice.rs`

### v19
- `constitutional-memory/src/lib.rs`
- `constitutional-memory/tests/amendment_slice.rs`

### v20
- `spec-execution/src/lib.rs`
- `spec-execution/tests/spec_to_schema_slice.rs`

### Repo-level truth docs
- `README.md`
- `EXECUTIVE_SUMMARY.md`
- `V16_V20_SURFACE_STATUS.md`
- `AGENTS.md`
- `CODEX_OPERATING_PROMPT.md`
- `MANIFEST.txt`

## New files to create

### Crate-local ownership docs
- `federated-settlement/README.md`
- `federated-settlement/AGENTS.md`
- `mechanism-runtime/README.md`
- `mechanism-runtime/AGENTS.md`
- `discovery-portfolio/README.md`
- `discovery-portfolio/AGENTS.md`
- `constitutional-memory/README.md`
- `constitutional-memory/AGENTS.md`
- `spec-execution/README.md`
- `spec-execution/AGENTS.md`

### New tests
- `federated-settlement/tests/replay_publication_slice.rs`
- `federated-settlement/tests/divergence_and_suspension_slice.rs`
- `mechanism-runtime/tests/refuter_and_stability_slice.rs`
- `discovery-portfolio/tests/value_aware_selection_slice.rs`
- `discovery-portfolio/tests/budget_exhaustion_slice.rs`
- `constitutional-memory/tests/archive_compaction_slice.rs`
- `spec-execution/tests/generated_artifacts_and_veto_slice.rs`

### New fixtures
#### v16
- `contracts/fixtures/v16/shared-replay-happy.json`
- `contracts/fixtures/v16/shared-replay-downgrade.json`
- `contracts/fixtures/v16/divergence-report.json`
- `contracts/fixtures/v16/treaty-suspension.json`

#### v17
- `contracts/fixtures/v17/refuter-gated-fit.json`
- `contracts/fixtures/v17/stability-report-block.json`

#### v18
- `contracts/fixtures/v18/value-aware-plan.json`
- `contracts/fixtures/v18/program-hypothesis-set.json`
- `contracts/fixtures/v18/budget-overload.json`

#### v19
- `contracts/fixtures/v19/semantic-diff-linked-amendment.json`
- `contracts/fixtures/v19/archive-compaction.json`

#### v20
- `contracts/fixtures/v20/self-hosting-build.json`
- `contracts/fixtures/v20/generated-companions.json`
- `contracts/fixtures/v20/human-veto-rollback.json`

## Create-vs-update rule

Prefer updating the existing crate-local `lib.rs` files for this pass.
Do **not** split into modules unless a file becomes materially hard to audit.
