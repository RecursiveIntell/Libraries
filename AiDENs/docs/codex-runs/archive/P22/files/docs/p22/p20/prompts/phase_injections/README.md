# Manual Phase-Injection Guardrails

These are the prompts the human operator pastes between Codex phases.

## Use pattern

1. Codex completes Phase XX and stops.
2. Read `docs/p20/reports/PHASE_XX_REPORT.md`.
3. Paste `GLOBAL_PRE_PHASE_INVARIANT_REVALIDATION.md`.
4. Paste the next phase-specific injection.
5. Let Codex run exactly one phase.
6. Repeat.

## Files

- `GLOBAL_PRE_PHASE_INVARIANT_REVALIDATION.md` — paste before every phase.
- `PHASE_00_START_INJECTION.md`
- `PHASE_01_BUILD_BASELINE_INJECTION.md`
- `PHASE_02_DOCS_TRUTH_INJECTION.md`
- `PHASE_03_CONTRACT_OWNERSHIP_INJECTION.md`
- `PHASE_04_SCANNER_VERIFY_INJECTION.md`
- `PHASE_05_PROVIDER_TRUTH_INJECTION.md`
- `PHASE_06_RUNNER_SLICE_INJECTION.md`
- `PHASE_07_CANONICAL_ADAPTERS_INJECTION.md`
- `PHASE_08_AGENCY_GOVERNANCE_INJECTION.md`
- `PHASE_09_REFERENCE_HOSTILE_TESTS_INJECTION.md`
- `PHASE_10_FINAL_AUDIT_INJECTION.md`

These prompts intentionally repeat hard laws. Redundancy is the point.
