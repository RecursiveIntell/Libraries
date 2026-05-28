# P20 Codex Run Prompt v2 — Truthful Finish and Release Hardening

You are executing `P20_TRUTHFUL_FINISH_AND_RELEASE_HARDENING` for AiDENs.

## Mission

Finish AiDENs v0.1 as a build-certified, documentation-honest, agency-aware, receipt-bearing orchestration layer over the real canonical libraries.

## Required initial read

Read these files before editing:

```text
AGENTS.md
CODEX_START_HERE.md
docs/p20/P20_FINISHLINE_SCOPE.md
docs/p20/P20_ACCEPTANCE_GATES.md
docs/p20/P20_PHASE_ORDER_AND_OPERATOR_PROTOCOL.md
docs/p20/P20_INVARIANT_CHECKLISTS.md
docs/p20/P20_OWNERSHIP_SOURCE_OF_TRUTH_MAP.md
docs/p20/P20_DEPENDENCY_SOURCE_OF_TRUTH_MATRIX.md
docs/p20/P20_DELETION_QUARANTINE_RULES.md
docs/p20/P20_ROLLBACK_REPAIR_QUARANTINE_PLAN.md
docs/p20/P20_CONTROL_PLANE_EXECUTION_EVIDENCE_SPEC.md
docs/p20/P20_AGENCY_GOVERNANCE_SPEC.md
docs/p20/P20_REFERENCE_INTERPRETER_CONFORMANCE_PLAN.md
```

If these files are not installed, install the overlay first.

## Hard laws

1. AiDENs directs/wires/coordinates; canonical crates own truth.
2. No shadow truth.
3. No silent semantic widening.
4. No fake provider capability.
5. No scaffold promotion.
6. No prompt-only agency policy.
7. No phase transition without invariant revalidation.
8. No unsupported completion claims.
9. No compatibility layer that invents semantics.
10. No final pass without audit evidence.

## Phase execution rule

Execute one phase at a time. At the end of each phase:

1. create/update `docs/p20/reports/PHASE_XX_REPORT.md`;
2. state pass/fail for that phase gate;
3. stop and wait for the operator's next guardrail injection.

Do not continue to the next phase until the operator provides the phase-specific injection prompt.

## Phase list

0. Operator arbitration, source basis, and baseline plan.
1. Build certification and raw failure repair.
2. Documentation truth reconciliation.
3. Contract ownership and shadow-truth collapse.
4. Boundary scanner and verify gate integration.
5. Provider capability truth.
6. Runner vertical slice proof.
7. Canonical adapter proof.
8. Agency/influence governance.
9. Reference interpreters and hostile tests.
10. Final audit bundle and hostile auditor handoff.

## Required final commands

```bash
bash scripts/p20_verify.sh
bash scripts/p20_generate_audit_bundle.sh
```

If they fail, P20 is not complete.

## First task

Start Phase 0 only. Produce `docs/p20/reports/PHASE_00_REPORT.md`, then stop.
