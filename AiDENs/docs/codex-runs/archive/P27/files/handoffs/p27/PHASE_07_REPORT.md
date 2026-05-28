# P27 Phase Report

## Phase

- Phase ID: 07
- Phase title: Reproducibility prerequisite and source-basis hardening
- Date: 2026-05-04T23:41:06Z

## Scope

- Intended work: make the sibling workspace prerequisite executable and update the source basis so package/cargo replay cannot be described as clean when sibling path dependencies are absent.
- Issue IDs in scope: `P27-005`.
- Explicit non-goals: no capability work, no vendoring sibling crates into AiDENs, no canonical-owner boundary changes, no promotion of package replay to full cargo-backed proof when cargo checks are skipped.

## Files inspected

- `prompts/phases/P27_PHASE_07_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_07_BEFORE_PHASE_08.md`
- `P27_PHASE_PLAN.md`
- `P27_MASTER_ISSUE_MATRIX.md`
- `Cargo.toml`
- `SOURCE_BASIS.md`
- `STATUS.md`
- `scripts/p27_verify.sh`
- `scripts/assert_package_self_replay.py`
- `scripts/assert_package_validation.py`
- `z.py`
- `docs/root-markdown-archive/20260504T231303Z/MANIFEST.json`

## Files changed

- `SOURCE_BASIS.md`
- `STATUS.md`
- `scripts/assert_sibling_workspace_layout.py`
- `scripts/p27_verify.sh`
- `z.py`
- `handoffs/p27/PHASE_07_REPORT.md`

## Changes made

- Added `scripts/assert_sibling_workspace_layout.py`, an executable prerequisite checker for Cargo path dependencies whose paths resolve outside AiDENs.
- The new checker emits a local operator receipt with `support_tier=verification`, `semantic_status=exact_check`, and `classification=sibling_workspace_present`, `sibling_workspace_missing`, or `manifest_missing`.
- Wired the sibling layout checker into `scripts/p27_verify.sh` after the scaffold profile fence.
- Updated `SOURCE_BASIS.md` with the P27 Phase 07 counts, the 22 required Cargo sibling path dependencies, and replay-mode classifications.
- Updated `STATUS.md` to close `P27-005` and add Phase 07 to the phase ledger.
- Updated `z.py` so Codex archive candidate discovery ignores `docs/root-markdown-archive/`, preventing archived root Markdown evidence from being re-archived during package generation.
- Restored root Markdown archive files that an initial package run had moved before the `z.py` archive exclusion fix.

## Commands run

| Command | Result | Log |
|---|---|---|
| workspace path dependency inventory | found 22 required sibling Cargo path dependencies | `target/p27/audit/phase07_workspace_path_deps_before.txt` |
| sibling path presence probe | all 22 present in `/home/sikmindz/Coding/Libraries` | `target/p27/audit/phase07_workspace_path_dep_presence_before.txt` |
| `python3 -m py_compile scripts/assert_sibling_workspace_layout.py` | pass | `target/p27/audit/phase07_py_compile.log` |
| `python3 scripts/assert_sibling_workspace_layout.py --root . --receipt-out ...` | pass; `sibling_workspace_present` | `target/p27/audit/phase07_sibling_layout_present.log`, `target/p27/audit/phase07_sibling_layout_present_receipt.json` |
| missing sibling fixture | fail-closed with exit code 2 and `sibling_workspace_missing` | `target/p27/audit/phase07_sibling_layout_missing.log`, `target/p27/audit/phase07_sibling_layout_missing_receipt.json`, `target/p27/audit/phase07_sibling_layout_missing_exit_code.log` |
| `bash -n scripts/p27_verify.sh` | pass | `target/p27/audit/phase07_bash_syntax.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass; includes sibling workspace layout gate | `target/p27/audit/phase07_verify_current_skip_cargo.log` |
| `python3 scripts/assert_current_run_truth.py .` | pass | `target/p27/audit/phase07_assert_current_run_truth.log` |
| `python3 scripts/assert_support_claims.py .` | pass | `target/p27/audit/phase07_assert_support_claims.log` |
| root Markdown archive manifest check after restore | pass | `target/p27/audit/phase07_root_markdown_manifest_after_restore.log` |
| Codex archive candidate probe after `z.py` fix | pass; candidates 0 | `target/p27/audit/phase07_codex_archive_candidate_probe_after_fix.log` |
| final package generation | pass; findings 0, Codex archive moved 0, root Markdown candidates 0 | `target/p27/audit/phase07_zpy_package_final.log`, `target/p27/audit/phase07_package_summary_facts_final.log` |
| `python3 scripts/assert_package_validation.py` | pass | `target/p27/audit/phase07_package_validation_final.log` |
| package self-replay with `P27_SKIP_CARGO=1` | pass; `semantic_status=degraded_exact_check` | `target/p27/audit/phase07_package_self_replay_final.log`, `target/p27/audit/phase07_package_self_replay_receipt_final.json` |

## Evidence emitted

- `target/p27/audit/phase07_sibling_layout_present_receipt_final.json`
- `target/p27/audit/phase07_sibling_layout_missing_receipt.json`
- `target/p27/audit/phase07_verify_current_skip_cargo.log`
- `target/p27/audit/phase07_package_validation_final.log`
- `target/p27/audit/phase07_package_self_replay_receipt_final.json`
- `target/p27/audit/phase07_package_summary_facts_final.log`
- `target/p27/audit/phase07_audit_file_list.txt`

## 11A semantic impact

- Exact/approx labels touched: added `semantic_status=exact_check` to sibling workspace layout receipts.
- Degradation labels touched: `SOURCE_BASIS.md` now explicitly labels `P27_SKIP_CARGO=1` package replay as `degraded_exact_check`.
- Support labels touched: no supported-local capability claim was widened.
- Proof/check hooks added: `scripts/verify_current.sh` now checks sibling workspace layout through `scripts/p27_verify.sh`.

## Support profile impact

- No support-tier claim changed.
- The new source-basis language narrows replay honesty: sibling workspace absence is an environment prerequisite failure, not a clean self-replay pass.

## Canonical-owner impact

- No canonical-owner boundary changed.
- AiDENs only checks presence of sibling crates required by its Cargo path dependencies. It does not claim canonical correctness for those sibling owners.
- Sibling owners named by the check: `assurance-runtime`, `attestation-exchange`, `authority-delegation`, `constraint-compiler`, `contract-schema-gen`, `federated-settlement`, `forge-memory-bridge`, `kernel-conformance`, `kernel-execution`, `kernel-oracles`, `knowledge-runtime`, `llm-tool-runtime`, `mechanism-runtime`, `recursive-kernel-core`, `remote-oracle-admission`, `semantic-memory`, `semantic-memory-forge`, `stack-ids`, `verification-adjudication`, `verification-calibration`, `verification-control`, and `verification-policy`.

## Issues closed

- `P27-005`: reproducibility prerequisite is now executable, receipt-bearing, wired into the current verifier, documented in `SOURCE_BASIS.md`, and package self-replay remains honestly degraded when cargo checks are skipped.

## New issues / risks

- Full cargo-backed package replay remains deferred when `P27_SKIP_CARGO=1` is used.
- The sibling layout checker proves path presence only. It does not validate sibling crate semantic correctness or ownership content.
- The initial Phase 07 package attempt exposed an archive interaction where `z.py` could move files already under `docs/root-markdown-archive/`; this was fixed and the archived files were restored before final package generation.

## Decision

Rationale: The sibling workspace prerequisite is now explicit, fail-closed, receipt-bearing, and included in verifier/package replay evidence. Package replay is green only under an honest degraded cargo-skipped classification.

Decision: continue
