# v25 production closure exec plan

## Goal

Close the remaining seams between the landed v25 composition owner and the downstream consumer surfaces so the stack can truthfully claim one replayable composite constitutional answer across effect, control, policy, adjudication, remote admission, and settlement.

## Ordered execution

1. Read the generated production gap audit.
2. Convert `effect-runtime` to typed IDs and direct composite-constitution citations.
3. Add an effect-side v25 helper surface that consumes compiled obligations rather than raw profiles.
4. Thread the same citations through `verification-control`.
5. Extend `verification-policy` so `PolicyDecision` cites the composite lane using `stack-ids` only.
6. Extend `verification-adjudication` so decisions and receipts cite the same policy/composition lane.
7. Add local constitutional refs to `remote-oracle-admission` and `federated-settlement`.
8. Register the missing review/adjudication schemas and backfill every missing example JSON.
9. Expand the fixture corpus and conformance notes.
10. Add no-local-recomposition and production-closure gates.
11. Regenerate schemas, run cargo tests, sync `libraries-source/`, and update the release proof docs.

## Non-goals

- inventing a new post-v26 spec wave,
- moving profile composition ownership away from `profile-runtime`,
- introducing a second hidden policy engine inside a consumer crate,
- claiming completion without cargo-backed proof,
- or rewriting historical docs to hide provenance.

## Required final proof

The end state is only acceptable if the commands in `docs/v25/PRODUCTION_ACCEPTANCE_AND_COMMANDS_20260318.md` all pass in a Rust-capable environment and `.github/workflows/ci.yml` runs the same checks.
