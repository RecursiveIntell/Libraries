# Production exact file touch map — 2026-03-18

This is the exact file set Codex should expect to touch for a clean production-closure pass.
It is intentionally concrete so the work stays non-archaeological.

## Core code changes

### `effect-runtime`
Modify:
- `effect-runtime/Cargo.toml`
- `effect-runtime/src/lib.rs`
- `effect-runtime/src/effect.rs`
- `effect-runtime/src/observation.rs`
- `effect-runtime/src/compensation.rs`
- `effect-runtime/tests/serde_roundtrip.rs`
- `effect-runtime/tests/fixture_conformance.rs`

Create:
- `effect-runtime/src/v25.rs`
- `effect-runtime/tests/v25_citation_flow.rs`

### `verification-control`
Modify:
- `verification-control/src/lib.rs`

Create:
- `verification-control/tests/v25_review_case_roundtrip.rs`
- `verification-control/tests/v25_citation_requirements.rs`

### `verification-policy`
Modify:
- `verification-policy/src/lib.rs`
- `verification-policy/tests/policy_profile_example_roundtrip.rs`

Create:
- `verification-policy/tests/v25_policy_citation_flow.rs`

### `verification-adjudication`
Modify:
- `verification-adjudication/src/lib.rs`
- `verification-adjudication/tests/policy_flow_integration.rs`

Create:
- `verification-adjudication/tests/v25_adjudication_citation_flow.rs`

### `remote-oracle-admission`
Modify:
- `remote-oracle-admission/src/lib.rs`

Create:
- `remote-oracle-admission/tests/v25_local_constitution_refs.rs`

### `federated-settlement`
Modify:
- `federated-settlement/src/lib.rs`

Create:
- `federated-settlement/tests/v25_local_constitution_refs.rs`

### schema publication
Modify:
- `contract-schema-gen/src/lib.rs`

## Schema and example files

Create or refresh:
- `schemas/effect-intent-v1.schema.json`
- `schemas/effect-preflight-report-v1.schema.json`
- `schemas/effect-commit-decision-v1.schema.json`
- `schemas/effect-execution-receipt-v1.schema.json`
- `schemas/effect-observation-bundle-v1.schema.json`
- `schemas/compensation-plan-v1.schema.json`
- `schemas/compensation-execution-receipt-v1.schema.json`
- `schemas/control-receipt-v1.schema.json`
- `schemas/effect-review-case-v1.schema.json`
- `schemas/effect-block-receipt-v1.schema.json`
- `schemas/delegation-review-case-v1.schema.json`
- `schemas/release-gate-case-v1.schema.json`
- `schemas/continuity-review-case-v1.schema.json`
- `schemas/policy-decision-v1.schema.json`
- `schemas/promotion-decision-v1.schema.json`
- `schemas/refutation-decision-v1.schema.json`
- `schemas/rollback-plan-v1.schema.json`
- `schemas/effect-adjudication-receipt-v1.schema.json`
- `schemas/release-rollback-decision-v1.schema.json`
- `schemas/remote-slice-request-v1.schema.json`
- `schemas/remote-slice-result-v1.schema.json`
- `schemas/cross-runtime-replay-ticket-v1.schema.json`
- `schemas/settlement-case-v1.schema.json`
- `schemas/settlement-receipt-v1.schema.json`
- `schemas/shared-replay-slice-v1.schema.json`
- `schemas/shared-divergence-report-v1.schema.json`
- `schemas/shared-view-downgrade-v1.schema.json`
- `schemas/local-dissent-record-v1.schema.json`
- matching `examples/*.example.json` files for every stem above.

## Fixtures and conformance

Modify:
- `contracts/fixtures/v25/manifest.json`
- `conformance/v25/README.md`
- `conformance/v25/manifest.json`

Create:
- `contracts/fixtures/v25/effect-constitutional-citation.bundle.json`
- `contracts/fixtures/v25/review-case-constitutional-citation.bundle.json`
- `contracts/fixtures/v25/remote-oracle-local-constitution.bundle.json`
- `contracts/fixtures/v25/federated-settlement-local-constitution.bundle.json`

## Gates and prompts

Modify:
- `Makefile`
- `.github/workflows/ci.yml`
- `docs/v25/README.md`
- `apply/v25/README.md`

Create:
- `scripts/check_v25_production_pack_truth.sh`
- `scripts/check_no_local_recomposition.sh`
- `scripts/check_v25_production_closure.py`
- `scripts/run_v25_production_pack_checks.sh`
