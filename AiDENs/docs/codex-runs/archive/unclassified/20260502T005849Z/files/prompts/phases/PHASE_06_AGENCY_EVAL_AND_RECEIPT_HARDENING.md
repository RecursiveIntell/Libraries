# PHASE 06 — AGENCY_EVAL_AND_RECEIPT_HARDENING

## Objective

Expand agency evals and receipt assertions.

## Required invariant revalidation

Before work, revalidate:

- no shadow truth;
- canonical ownership preserved;
- no silent semantic widening;
- execution evidence emitted for execution paths;
- agency/influence gates preserved;
- package/test integrity not weakened.

## Required output

- changed files;
- commands run;
- pass/fail against phase gates;
- invariant validation result;
- unresolved risks;
- next phase readiness.

## Stop rule

If this phase requires weakening a canonical boundary or deleting coverage to pass, stop and report.
