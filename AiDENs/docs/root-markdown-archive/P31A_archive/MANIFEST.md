# P31A Root Markdown Archive Manifest

**Date:** 2026-05-29
**Active run:** P31A
**Archive location:** `docs/root-markdown-archive/P31A_archive/`

## Archived categories

### Stale run docs (P24–P30)
All `P{N}_*.md`, `P{N}_*.json`, `P{N}_*.template.json` files from completed runs P24–P30.
Count: ~89 root-level + 10 status-evidence JSON files

### Codex sidecar artifacts
All `AiDENs-aidens-codex-context-*.json`, `AiDENs-aidens-codex-context-*.md`, `.zip`, `.excluded.json`, `.findings.json`, `.codex-archive.json` files.
Also `AiDENs-aidens-next-codex-context-*` variants.

### Ambiguous/legacy root markdown
- `00_OPERATOR_PASTE_FIRST.md` — codex operator prompt
- `05_ACCEPTANCE_GATES_AND_COMMAND_BAR.md` — codex injection
- `07_FORBIDDEN_FINAL_STATES_AND_LABEL_POLICY.md` — codex injection
- `ACCEPTANCE_GATES_AND_CI.md` — pre-P31A acceptance
- `AIDENS_STACK_INTEGRATION_GAP.md` — research artifact
- `MANIFEST.md` — codex manifest
- `MASTER_ISSUE_MATRIX.json` — historical matrix
- `README.z.py.md` — z.py doc sidecar
- `RUN_ORDER.md` — pre-P31A run order
- `STATUS_TEMPLATE.md` — template
- `INSTALL_P30_BUNDLE_TO_REPO.sh` — P30 bundle installer

### Archived subdirectories
- `prompts/` — 21 phase prompt files
- `evidence/` — P24-P26 static audit snapshots, CSVs
- `input_evidence/` — May 08 codex context sidecars
- `manual_injections/` — P30 revalidation injections (10 files)
- `matrices/` — P24-P30 issue/audit matrices
- `passes/` — P00-P19 pass descriptions
- `phase_prompts/` — 6 phase prompt templates
- `source_audits/` — May 07 audit sidecars + claude audit
- `handoff/` — P25 final auditor handoff template
- `phase_injections/` — P26-P27 gate injection files
- `handoffs/` — P30 gate supersession + super-pass reports

## Durable root docs retained

- `README.md` — project readme (P31A)
- `STATUS.md` — current run status (P31A)
- `SOURCE_BASIS.md` — source basis (P31A)
- `SUPPORT_PROFILE.md` — support label (P31A)
- `AGENTS.md` — agent protocol
- `CANONICAL_OWNER_MAP.md` — ownership declarations
- `SHADOW_SEMANTICS_AUDIT.md` — active shadow audit (P31A)
- `COMPATIBILITY_LEDGER.md` — compatibility tracking
- `ACCEPTANCE_GATES.md` — acceptance gate policy
- `TESTKIT_TARGETS.md` — test kit targets
- `z.py` — packaging script
- `zip.py` — zip utility

## Rollback

To restore any archived file, move it from `docs/root-markdown-archive/P31A_archive/` back to the repository root. All files retain their original names.