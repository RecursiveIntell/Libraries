# PHASE 03 — Audit evidence extensions and root hygiene

Tasks:

1. Include `.patch` and `.diff` in context/audit modes.
2. Keep release/source-clean stricter unless policy allows patches.
3. Preserve `commands_run.log` and `commands_run.receipts.jsonl` in context/audit modes.
4. Generalize root package artifact cleanup using configurable patterns.
5. Ensure moved root artifacts create a manifest with sha256/size/mtime/reason.

Acceptance:

- Fixture package includes `.codex-runs/.../touched_diff.patch`.
- Root package sidecars move before packaging.
- Root remains clean after package generation.
