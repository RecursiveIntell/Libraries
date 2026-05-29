# Zip Source Certifier Report

## Summary

- Script version: `2026.05.01`
- Created UTC: `2026-05-02T06:04:37Z`
- Root: `/home/sikmindz/Coding/Libraries/AiDENs`
- Archive root: `/home/sikmindz/Coding/Libraries`
- Output: `/home/sikmindz/Coding/Libraries/AiDENs/AiDENs-aidens-codex-context-20260502.zip`
- Include roots: `41`
- External Cargo path dependency roots: `40`
- Profile: `aidens` requested as `auto`
- Mode: `codex-context`
- Strict: `True`
- Dry run: `False`
- Included files: `1270`
- Included bytes: `9194129`
- Excluded files: `29`
- Pruned dirs: `10`
- Findings: `2` (`0` errors, `2` warnings)
- Archive SHA-256: `c44048b51056a2649a48829f0c8d94d3af70bc7773bbdeb6958c31c3c0089cf9`
- Codex archive enabled: `True`
- Codex archive planned: `0`
- Codex archive moved: `0`
- Codex active stale after normalization: `0`

## Validation findings

| Severity | Code | Path | Detail |
|---|---|---|---|
| warning | `secret-like-filename` | `AiDENs/prompts/phases/PHASE_05_SECRET_REDACTION_AND_API_KEY_WARNING_CLOSURE.md` | File excluded because of secret-like-filename. |
| warning | `secret-like-filename` | `AiDENs/scripts/p22_secret_scan_fixture_test.py` | File excluded because of secret-like-filename. |

## Included files by extension

| Extension | Count |
|---|---:|
| `.rs` | 482 |
| `.md` | 406 |
| `.json` | 182 |
| `.toml` | 99 |
| `.sh` | 30 |
| `.csv` | 26 |
| `.lock` | 16 |
| `.py` | 10 |
| `<no-extension>` | 8 |
| `.txt` | 4 |
| `.jsonl` | 3 |
| `.ndjson` | 3 |
| `.yml` | 1 |

## Included files by top-level path

| Top-level path | Count |
|---|---:|
| `AiDENs` | 633 |
| `forge-pilot` | 92 |
| `semantic-memory` | 86 |
| `living-memory` | 60 |
| `Primitives` | 55 |
| `knowledge-runtime` | 38 |
| `assurance-runtime` | 18 |
| `continuity-runtime` | 16 |
| `profile-runtime` | 16 |
| `effect-runtime` | 15 |
| `verification-policy` | 15 |
| `authority-delegation` | 14 |
| `stack-ids` | 14 |
| `forge-memory-bridge` | 13 |
| `semantic-memory-forge` | 13 |
| `kernel-conformance` | 12 |
| `llm-tool-runtime` | 12 |
| `verification-control` | 10 |
| `attestation-exchange` | 9 |
| `constitutional-memory` | 9 |
| `discovery-portfolio` | 9 |
| `federated-settlement` | 9 |
| `mechanism-runtime` | 9 |
| `verification-adjudication` | 9 |
| `spec-execution` | 8 |
| `contract-schema-gen` | 6 |
| `remote-oracle-admission` | 6 |
| `constraint-compiler` | 5 |
| `recursive-kernel-core` | 5 |
| `verification-calibration` | 5 |
| `kernel-execution` | 4 |
| `kernel-oracles` | 4 |
| `.gitignore` | 1 |
| `06_RISK_REGISTER.md` | 1 |
| `AGENTS.md` | 1 |
| `AUDIT_2026-04-01.md` | 1 |
| `CLAUDE.md` | 1 |
| `COMBINED_AUDIT_2026-04-01.md` | 1 |
| `CONFORMANCE_GATES.md` | 1 |
| `CONTRACT_AND_TEMPORAL_TRUTH_HARDENING.md` | 1 |
| `CRATE_HARDENING_MATRIX.md` | 1 |
| `Cargo.lock` | 1 |
| `Cargo.toml` | 1 |
| `EXECUTION_EVIDENCE_AND_REFERENCE_INTERPRETER_PLAN.md` | 1 |
| `HOSTILE_AUDIT_SYNTHESIS_V5.md` | 1 |
| `KERNEL_AND_REGION_RUNTIME_PLAN.md` | 1 |
| `LIBRARIES_MASTER_MATRIX_V8.md` | 1 |
| `LIBRARIES_MASTER_TENSOR_V8.json` | 1 |
| `LIBRARIES_PROMPT.md` | 1 |
| `LIB_MASTER_ISSUE_MATRIX.md` | 1 |
| `LIB_MASTER_ISSUE_TENSOR.json` | 1 |
| `LIB_PROMPT.md` | 1 |
| `MASTER_ISSUE_MATRIX.md` | 1 |
| `MASTER_ISSUE_TENSOR.json` | 1 |
| `MASTER_TENSOR.md` | 1 |
| `Makefile` | 1 |
| `PACK_MANIFEST.json` | 1 |
| `PROMPT.md` | 1 |
| `README.md` | 1 |
| `RISK_REGISTER.md` | 1 |
| `SCOPE_NOTES.md` | 1 |
| `SNAPSHOT_2026-04-11.md` | 1 |
| `SOURCE_BASIS.md` | 1 |
| `STATUS_DASHBOARD.md` | 1 |
| `STATUS_EVIDENCE_MANIFEST.json` | 1 |
| `SUPPORT_PROFILE.md` | 1 |
| `TEST_AND_CONFORMANCE_PLAN.md` | 1 |
| `V9_IMPLEMENTATION_PLAYBOOK.md` | 1 |
| `claude_hard_audit_2026-03-30.md` | 1 |
| `gpt54_hard_audit_2026-03-30.md` | 1 |
| `gpt54_tensor_2026-03-30.json` | 1 |
| `support_lane.toml` | 1 |
| `zip.py` | 1 |

## Exclusion reasons

| Reason | Count |
|---|---:|
| `archive-file` | 22 |
| `doc-binary-disabled` | 2 |
| `secret-like-filename` | 2 |
| `generated-output` | 1 |
| `stale-codex-artifact-disabled` | 1 |
| `unsupported-extension-or-basename` | 1 |

## Sidecar files

- Manifest: `/home/sikmindz/Coding/Libraries/AiDENs/AiDENs-aidens-codex-context-20260502.manifest.json`
- Markdown report: `/home/sikmindz/Coding/Libraries/AiDENs/AiDENs-aidens-codex-context-20260502.report.md`
- Excluded file list: `/home/sikmindz/Coding/Libraries/AiDENs/AiDENs-aidens-codex-context-20260502.excluded.json`
- Findings: `/home/sikmindz/Coding/Libraries/AiDENs/AiDENs-aidens-codex-context-20260502.findings.json`

## Interpretation

This package has warnings. It is probably usable, but the warnings should be reviewed before using it as a Codex or audit handoff.
