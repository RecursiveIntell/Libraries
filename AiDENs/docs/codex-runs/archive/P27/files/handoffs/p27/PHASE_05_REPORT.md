# P27 Phase Report

## Phase

- Phase ID: 05
- Phase title: Root Markdown truth and archive hygiene
- Date: 2026-05-04T23:14:28Z

## Scope

- Intended work: archive or classify root Markdown drift without deleting evidence, and align current-run archive policy with P27.
- Issue IDs in scope: `P27-007`, `P27-020`.
- Explicit non-goals: no capability work, no canonical-owner boundary changes, no support-claim widening, no broad root-doc semantic rewrite, no full package regeneration.

## Files inspected

- `prompts/phases/P27_PHASE_05_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_05_BEFORE_PHASE_06.md`
- `scripts/assert_root_markdown_archive_policy.py`
- `scripts/assert_root_markdown_archive_manifest.py`
- `scripts/assert_codex_artifact_classification.py`
- `z.py`
- `docs/codex-runs/CURRENT_RUN.md`
- `docs/codex-runs/ARCHIVAL_POLICY.md`
- `docs/codex-runs/CODEX_ARTIFACT_CLASSIFICATION.json`
- root `*.md`

## Files changed

- `docs/codex-runs/CURRENT_RUN.md`
- `docs/root-markdown-archive/20260504T231303Z/ROOT_MARKDOWN_ARCHIVE_MANIFEST.json`
- `docs/root-markdown-archive/20260504T231303Z/files/*`
- `scripts/assert_root_markdown_archive_policy.py`
- `z.py`
- `STATUS.md`
- `handoffs/p27/PHASE_05_REPORT.md`

## Changes made

- Updated `docs/codex-runs/CURRENT_RUN.md` from P25 to P27.
- Updated the root Markdown policy helper default current-run token from P25 to P27.
- Updated `z.py` default `--codex-current-run` from P25 to P27.
- Added `SHADOW_SEMANTICS_AUDIT.md` to `z.py` root Markdown protected files to align with the P27 policy helper.
- Archived 10 stale P25/P26 candidate root Markdown files into `docs/root-markdown-archive/20260504T231303Z/files/`.
- Preserved archive evidence with `ROOT_MARKDOWN_ARCHIVE_MANIFEST.json` including original path, archived path, SHA-256, byte size, mtime, reason, and classification.
- Updated `STATUS.md` to record Phase 05 results.

## Archived files

- `P25_CLAUDE_AUDIT_ABSORPTION.md`
- `P25_CODEX_RUN_PROMPT.md`
- `P25_LARGE_FILE_CONTAINMENT_PLAN.md`
- `P25_MASTER_PACKET.md`
- `P25_PHASE_PLAN.md`
- `P25_ROLLBACK_AND_QUARANTINE_PLAN.md`
- `P25_ROOT_MARKDOWN_ARCHIVE_TEST_PLAN.md`
- `P26_MASTER_PACKET.md`
- `P26_PHASE_PLAN.md`
- `P26_ROLLBACK_AND_QUARANTINE_PLAN.md`

## Commands run

| Command | Result | Log |
|---|---|---|
| root Markdown listing before | pass | `target/p27/audit/phase05_root_markdown_before.txt` |
| `python3 scripts/assert_root_markdown_archive_policy.py` before | fail; candidates present | `target/p27/audit/phase05_root_markdown_policy_before.log` |
| existing archive listing | pass | `target/p27/audit/phase05_existing_root_markdown_archive_files.txt` |
| `python3 z.py ... --archive-root-markdown-noise --root-markdown-archive-dry-run` | dry-run; showed P25 default/current-run drift and candidates | `target/p27/audit/phase05_zpy_root_markdown_dry_run_before.log` |
| `python3 -m py_compile z.py scripts/assert_root_markdown_archive_policy.py scripts/assert_root_markdown_archive_manifest.py` | pass | `target/p27/audit/phase05_py_compile.log` |
| `python3 z.py --root . --profile aidens --mode next-codex-context --codex-current-run P27 --archive-root-markdown-noise --archive-only --no-strict` | moved 10 candidates; ambiguous docs remain | `target/p27/audit/phase05_zpy_root_markdown_archive.log` |
| `python3 scripts/assert_root_markdown_archive_policy.py` after archive | pass with warning for 92 ambiguous root docs | `target/p27/audit/phase05_root_markdown_policy_after_archive.log` |
| `python3 scripts/assert_root_markdown_archive_manifest.py` | pass | `target/p27/audit/phase05_root_markdown_manifest_after_archive.log` |
| root Markdown listing after | pass | `target/p27/audit/phase05_root_markdown_after.txt` |
| archived candidate absence check | pass | `target/p27/audit/phase05_archived_candidate_absence_check.log` |
| `python3 scripts/assert_codex_artifact_classification.py` | fail; broader classification registry still incomplete | `target/p27/audit/phase05_assert_codex_artifact_classification.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass | `target/p27/audit/phase05_verify_current_skip_cargo.log` |
| current-run and AGENTS assertions | pass | `target/p27/audit/phase05_assert_current_run_truth_after_status.log`, `target/p27/audit/phase05_assert_agents_current.log` |

## Evidence emitted

- `docs/root-markdown-archive/20260504T231303Z/ROOT_MARKDOWN_ARCHIVE_MANIFEST.json`
- `target/p27/audit/phase05_root_markdown_policy_after_archive.log`
- `target/p27/audit/phase05_root_markdown_manifest_after_archive.log`
- `target/p27/audit/phase05_archived_candidate_absence_check.log`
- `target/p27/audit/phase05_zpy_root_markdown_archive.log`
- `target/p27/audit/phase05_assert_codex_artifact_classification.log`
- `target/p27/audit/phase05_verify_current_skip_cargo.log`

## 11A semantic impact

- Exact/approx labels touched: none in code schemas.
- Proof/check hooks added: root Markdown archive manifest with hashes and movement evidence.
- Degradation/support labels changed: none.

The archive manifest is AiDENs-local operator evidence. It preserves historical files and does not alter canonical sibling truth.

## Support profile impact

- No support-tier claim changed.

## Issues closed

- `P27-007`: root Markdown archive candidates are removed from the root and preserved with a manifest.

## Partial / remaining issues

- `P27-020`: current-run marker is now P27, but `scripts/assert_codex_artifact_classification.py` still reports unclassified active P27 prompt/handoff files and archived files under `docs/root-markdown-archive/20260504T231303Z/files/`. This needs classification registry cleanup in a later pass.
- 92 ambiguous root Markdown files remain. The Phase 05 archive policy allows them with warning because they require operator classification rather than mechanical movement.

## Decision

Rationale: Root Markdown candidate drift was reduced without deleting evidence, the archive manifest validates, and the current-run marker now points to P27. Remaining ambiguity is explicitly recorded and does not block Phase 06 scaffold-claim cleanup.

Decision: continue
