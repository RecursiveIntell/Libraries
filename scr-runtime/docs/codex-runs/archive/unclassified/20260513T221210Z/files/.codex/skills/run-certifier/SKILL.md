---
name: run-certifier
description: Certify final ClaimLedger completion with commands, receipts, fresh-unzip evidence, and unresolved risks.
---

Use this skill for final handoff.

Required evidence:
- `python -m pytest -q`
- `bash scripts/run_all_checks.sh`
- `python scripts/validate_codex_pack.py`
- `python scripts/assert_codex_active_pack.py`
- `bash scripts/run_completion_checks.sh`
- archive includes `.codex` if an archive is produced
- final report in `docs/PASS_CL_P0_COMPLETION_AUTOMATED_PHASES.md`
