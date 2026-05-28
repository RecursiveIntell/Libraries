# P20.1 Package Integrity Plan

## Required restorations

Restore or remove from manifest:

```text
evals/p20_agency_eval_cases.jsonl
fixtures/runner/expected_event_log.ndjson
supporting/matrices/forbidden_leftovers.csv
supporting/matrices/phase_acceptance_gates.csv
supporting/matrices/source_of_truth_matrix.csv
```

The supplied overlay includes seed versions of those files. Codex must verify they match actual tests/fixtures and update if necessary.

## Required checks

```bash
python3 scripts/p20_1_hard_code_audit.py --fail-on-blocking
python3 scripts/p20_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl
```

## Final rule

If a file is named in `MANIFEST.txt`, referenced by code, referenced by a script, or required by an install/handoff flow, it must be present in the release archive.
