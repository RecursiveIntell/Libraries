# Codex Prompt — P17 Attested exchange, trust roots, federation, and external artifact admission

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P17_ATTESTED_EXCHANGE_FEDERATION_AND_EXTERNAL_ADMISSION.md`.

Implement P17 only. Do not start later passes.

## Goal

Allow artifacts to cross runtime boundaries without creating a central truth plane or silently laundering remote claims.

## Primary crates

- `aidens-delegation-kit`
- `aidens-governance-kit`
- `aidens-memory-kit`
- `aidens-contracts`
- `aidens-receipts`

## Required artifacts

- `AttestationEnvelopeV1`
- `TrustRootV1`
- `AdmissionDecisionV1`
- `RemoteOracleReceiptV1`
- `TreatyV1`
- `SettlementCaseV1`
- `SharedDispositionV1`

## Acceptance gates

- External artifact can be imported only through AdmissionDecisionV1.
- Remote contradiction creates settlement/dispute artifact, not direct overwrite.
- Trust-root revocation downgrades affected artifacts and emits receipts.

## Forbidden shortcuts

- Do not let remote votes overwrite local truth.
- Do not accept unsigned external truth-bearing artifacts without quarantine/disclosure.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
