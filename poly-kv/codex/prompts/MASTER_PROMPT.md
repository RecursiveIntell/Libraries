# MASTER PROMPT — z.py Universal Packager Hardening

You are executing a deterministic, auditable implementation pass on `z.py`.

Your goal is to turn `z.py` into a reusable cross-repo source/context/handoff package certifier while preserving current behavior and evidence discipline.

Do not maximize apparent progress. Preserve source-of-truth boundaries, validate every change, and produce receipts.

## Source basis

Use these bundle docs:

- `docs/RESEARCH_SYNTHESIS.md`
- `docs/HIGH_ROI_CHANGE_MATRIX.md`
- `docs/ZPY_NEXT_ARCHITECTURE_SPEC.md`
- `docs/ECOSYSTEM_PARITY_CHECKS.md`
- `docs/SECURITY_AND_PORTABILITY_GATES.md`
- `docs/IMPLEMENTATION_PLAN.md`
- `schemas/PackagePolicyV1.schema.json`

## Hard rules

- Inspect current `z.py` before editing.
- Preserve old CLI invocations unless explicitly impossible.
- Do not remove existing poly-kv-specific safeguards; generalize them.
- Do not make z.py publish packages.
- Do not require Cargo/npm/Docker/Go/Python build tools for static validation to pass; missing tools must be reported honestly.
- Do not silently include secrets, generated packages, cache directories, or stale root artifacts.
- Every new inclusion/exclusion behavior must produce manifest/report evidence.
- Every package must be verifiable after transfer without relying on build-machine absolute paths.
- End with a hostile-auditor handoff.

## Required final report

Produce:

1. changed files;
2. commands run;
3. tests passed/failed/skipped with reasons;
4. z.py behavioral changes;
5. package-policy/config changes;
6. ecosystem adapters implemented and not implemented;
7. security/portability gates implemented and not implemented;
8. validation evidence;
9. unresolved risks;
10. rollback plan.
