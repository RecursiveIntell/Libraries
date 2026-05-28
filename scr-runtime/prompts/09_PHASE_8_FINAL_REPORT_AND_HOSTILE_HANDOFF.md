# Phase 8 — Final Report and Hostile Auditor Handoff

## Objective

Produce a falsifiable completion report, not a victory lap.

## Required final artifacts

Create/update:

```text
docs/P31_COMPLETION_REPORT.md
docs/P31_HOSTILE_AUDITOR_HANDOFF.md
docs/P31_UNRESOLVED_RISKS.md
docs/P31_COMMAND_RECEIPTS.md
docs/P31_CHANGED_FILES.md
```

## Required content

Final report must include:

- source basis summary;
- owner-boundary map summary;
- changed files;
- deleted/archived files;
- commands run with exact output or summary + log path;
- tests passed/failed/skipped;
- package parity proof;
- fresh unzip proof if package generated;
- unresolved ambiguities;
- known deferred work;
- explicit non-goals preserved;
- confirmation that no existing crate capabilities were silently reinvented;
- exact remaining blockers if any.

## Final gate

Run:

```bash
bash scripts/run_p31_completion_checks.sh
```

No completion claim if it fails.
