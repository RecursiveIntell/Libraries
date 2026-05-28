---
name: hostile-audit
description: Audit SCR completion for false completion, missing receipts, missing active Codex pack, and packaging/test drift.
---

Use this skill before final report.

Look for:
- `.codex` required by tests but absent from archive.
- package certifier passing while pytest fails.
- automated phase-gate remnants in active workflow.
- generated `egg-info` in source package.
- root loose packaging scripts must be owned or explicitly classified as legacy.
- release claims without fresh-unzip proof.
