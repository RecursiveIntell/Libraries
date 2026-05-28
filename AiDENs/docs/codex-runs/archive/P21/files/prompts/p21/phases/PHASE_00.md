# Phase 00 — Package / Source Closure

Run all package integrity scanners. Restore any missing referenced fixture/script/eval files. Do not delete tests to pass.

Required commands:

```bash
python3 scripts/p21_scan_package_integrity.py .
python3 scripts/p21_scan_source_cross_refs.py .
python3 scripts/p20_2_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl
bash scripts/p21_verify.sh
```

Write `handoffs/p21/PHASE_00_REPORT.md`.
