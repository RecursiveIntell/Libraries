# Phase 3 Prompt — Root Markdown and Codex Artifact Classification

Normalize active run artifacts without deleting history.

Required:

- archive or quarantine stale root P24–P30 docs/scripts;
- mark existing P31 boundary compiler files as `deferred-next-plan`, not active current instruction;
- create/update `docs/codex-runs/CODEX_ARTIFACT_CLASSIFICATION.json`;
- update root Markdown and Codex artifact classification scripts to read `CURRENT_RUN.json`.

Run:

```bash
python3 scripts/assert_root_markdown_archive_policy.py
python3 scripts/assert_codex_artifact_classification.py
```

Ambiguous active root Markdown must be zero or blocker-quarantined.
