# Phase 6 — Hostile Scripts and Final Audit

Create scripts:

```text
scripts/run_all_checks.sh
scripts/generate_schemas.sh
scripts/validate_schemas.py
scripts/verify_golden_fixtures.sh
scripts/assert_no_unexplained_golden_changes.sh
scripts/assert_no_feut_contamination.sh
scripts/assert_no_durable_float_scores.sh
scripts/assert_no_naked_decision_booleans.sh
scripts/assert_no_shadow_truth.sh
scripts/assert_no_llm_or_network_calls.sh
```

## Required behavior

- clean repo passes
- seeded FEUT/EEG production contamination fails
- seeded f64 durable score fails
- seeded bool decision API fails
- seeded receipt omission fails
- seeded shadow mutable store fails
- seeded network/LLM dependency fails

## Final report

Use `templates/FINAL_REPORT.md`.

Report:

- exact commands
- exact results
- changed files
- fixture decisions
- seeded violation results
- unresolved risks
- assumptions
- non-goals preserved
- confirmation that P0A did not integrate into Recall/AiDENs/memory/retrieval/tools
