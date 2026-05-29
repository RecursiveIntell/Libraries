# Phase 1 Prompt — Canonical Release Ledger and Protected Docs

Create or update the canonical release ledger and protected mirrors.

Required changes:

- create `docs/codex-runs/CURRENT_RUN.json` using `aidens.current-run.v1`;
- create/update `docs/codex-runs/CURRENT_RUN.md` as a human-readable mirror;
- append to `docs/codex-runs/RUN_LEDGER.jsonl`;
- update `README.md`, `STATUS.md`, `SOURCE_BASIS.md`, `SUPPORT_PROFILE.md` so they cite the ledger and do not claim P31 boundary compiler as current active work;
- add the P31A release-truth law to `AGENTS.md`;
- update/create release truth scripts.

Run:

```bash
python3 scripts/assert_release_ledger_schema.py
python3 scripts/assert_current_run_truth.py
python3 scripts/assert_release_truth_consistency.py
python3 scripts/assert_support_claims_have_evidence.py
```

Stop and report results.
