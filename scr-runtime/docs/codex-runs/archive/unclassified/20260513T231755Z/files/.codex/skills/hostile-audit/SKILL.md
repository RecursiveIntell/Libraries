---
name: hostile-audit
description: Audit ClaimLedger completion for false completion, missing receipts, missing active Codex pack, and packaging/test drift.
---

Use this skill before final report.

Look for:
- `.codex` required by tests but absent from archive.
- package certifier passing while pytest fails.
- manual-injection remnants in active workflow.
- generated `egg-info` in source package.
- root loose scripts such as `z.py` without ownership.
- release claims without fresh-unzip proof.
