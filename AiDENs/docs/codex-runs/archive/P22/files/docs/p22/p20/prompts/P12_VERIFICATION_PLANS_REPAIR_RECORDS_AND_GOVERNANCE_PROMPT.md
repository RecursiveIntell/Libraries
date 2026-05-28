# Codex Prompt — P12 Canonical verification, repair, contradiction, and governance adapters

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P12_VERIFICATION_PLANS_REPAIR_RECORDS_AND_GOVERNANCE.md`.

Implement P12 only. Do not start later passes.

## Goal

Make risk-bearing outputs route through canonical verification, refutation, repair, and governance library artifacts before promotion.

## Primary crates

- `aidens-governance-kit`
- `aidens-repair-kit`
- `aidens-arbiter-kit`
- `aidens-contracts`
- `aidens-memory-kit`
- `aidens-receipts`

## Required artifacts

- `verification_control::CheckPlan`
- `verification_control::ControlReceipt`
- `verification_control::BoundaryRepairRecord`
- `verification_adjudication::PromotionDecision`
- `semantic_memory_forge::EvidenceBundle`
- `semantic_memory_forge::ContradictionWitnessV1`

## Acceptance gates

- Risk-bearing claims route through canonical verification-control/policy/adjudication artifacts.
- Contradiction uses canonical Forge witnesses and does not silently overwrite memory.
- Repair emits canonical verification-control or Forge repair lineage.

## Forbidden shortcuts

- Do not call a ranking score “verified”.
- Do not delete contradicted claims; supersede or quarantine them.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
