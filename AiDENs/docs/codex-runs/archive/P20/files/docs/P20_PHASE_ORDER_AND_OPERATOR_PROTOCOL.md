# P20 Phase Order and Operator Protocol

## Codex execution rule

Codex may complete one phase at a time. At the end of every phase it must stop and emit a phase report.

The human operator then pastes the matching guardrail prompt from `prompts/phase_injections/` before the next phase begins.

## Phase order

0. Operator arbitration, source basis, and baseline plan
1. Build certification and raw failure repair
2. Documentation truth reconciliation
3. Contract ownership and shadow-truth collapse
4. Boundary scanner and verify gate integration
5. Provider capability truth
6. Runner vertical slice proof
7. Canonical memory/kernel/governance/repair adapter proof
8. Agency/influence governance layer
9. Reference interpreters and hostile tests
10. Release audit bundle and hostile auditor handoff

## Required phase report format

At the end of every phase Codex must create/update:

```text
docs/p20/reports/PHASE_XX_REPORT.md
```

Each report must include:

- phase objective;
- files changed;
- commands run;
- tests added/updated;
- invariant checklist result;
- violations found;
- repairs/quarantines performed;
- unresolved risks;
- pass/fail gate result;
- next phase preconditions.

## Human injection rule

The operator must paste:

```text
prompts/phase_injections/GLOBAL_PRE_PHASE_INVARIANT_REVALIDATION.md
```

plus the phase-specific injection before each phase begins.

## Stop rule

If any invariant fails, Codex must halt, repair, quarantine, or explicitly mark the phase failed. Continuing for momentum is an architectural violation.
