# Production gap audit — 2026-03-18

This report was generated from the current repository snapshot by `scripts/audit_v25_production_gap.py`.
It is the grounded basis for the Codex closure pack and records what is still missing for a production-grade v25 lane.

## High-level findings

- `effect-runtime`: typed-ID markers 0/8; v25 citation markers 0/7.
- `federated-settlement`: v25 citation markers 0/7.
- `remote-oracle-admission`: v25 citation markers 0/7.
- `verification-adjudication`: v25 citation markers 0/7.
- `verification-control`: v25 citation markers 0/7.
- `verification-policy`: v25 citation markers 0/4.

## CI and command-surface findings

- `ci_runs_no_local_recomposition`: `False`
- `ci_runs_v25_json_surface`: `False`
- `ci_runs_v25_repo_truth`: `False`
- `make_has_production_closure_target`: `False`
- `make_has_v25_local_checks`: `False`

## Schema/example publication gaps

- Missing schemas: `continuity-review-case-v1`, `delegation-review-case-v1`, `effect-adjudication-receipt-v1`, `effect-block-receipt-v1`, `effect-review-case-v1`, `release-gate-case-v1`, `release-rollback-decision-v1`.
- Missing examples: `continuity-review-case-v1`, `control-receipt-v1`, `cross-runtime-replay-ticket-v1`, `delegation-review-case-v1`, `effect-adjudication-receipt-v1`, `effect-block-receipt-v1`, `effect-review-case-v1`, `local-dissent-record-v1`, `policy-decision-v1`, `promotion-decision-v1`, `refutation-decision-v1`, `release-gate-case-v1`, `release-rollback-decision-v1`, `remote-slice-request-v1`, `remote-slice-result-v1`, `rollback-plan-v1`, `settlement-case-v1`, `settlement-receipt-v1`, `shared-divergence-report-v1`, `shared-replay-slice-v1`, `shared-view-downgrade-v1`.
- Missing `contract-schema-gen` registrations: `continuity-review-case-v1`, `delegation-review-case-v1`, `effect-adjudication-receipt-v1`, `effect-block-receipt-v1`, `effect-review-case-v1`, `release-gate-case-v1`, `release-rollback-decision-v1`.

## No-local-recomposition scan

The current target consumers do **not** presently show obvious raw profile-field access. That is good, but CI still does not enforce it.

## Immediate closure priorities

1. Convert `effect-runtime` from raw `String` IDs to `stack-ids` newtypes and add v25 citation fields.
2. Thread composite constitutional refs through `verification-control`, `verification-policy`, and `verification-adjudication`.
3. Publish the missing schemas/examples for the review/adjudication surfaces and backfill example JSONs for every externally visible consumer artifact.
4. Add codified CI and local gates for no-local-recomposition and full production-closure validation.
