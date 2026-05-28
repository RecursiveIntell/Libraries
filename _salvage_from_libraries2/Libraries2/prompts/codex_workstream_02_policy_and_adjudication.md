# Codex workstream 02 — policy and adjudication

## Scope

- V25P-202
- V25P-203

## Files to prioritize

- `verification-policy/src/lib.rs`
- `verification-policy/tests/v25_policy_citation_flow.rs`
- `verification-adjudication/src/lib.rs`
- `verification-adjudication/tests/policy_flow_integration.rs`
- `verification-adjudication/tests/v25_adjudication_citation_flow.rs`

## Required outcome

- `PolicyDecision` cites the composition/effective/obligation lane using `stack-ids` only,
- adjudication outputs cite the same lane and a specific policy decision,
- and no dependency cycle on `profile-runtime` is created.
