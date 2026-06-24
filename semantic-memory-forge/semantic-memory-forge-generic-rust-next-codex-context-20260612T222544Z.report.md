# Zip Source Certifier Report

## Summary

- Script version: `2026.05.22-p31`
- Created UTC: `2026-06-12T22:25:44Z`
- Root: `/home/sikmindz/Coding/Libraries/semantic-memory-forge`
- Archive root: `/home/sikmindz/Coding/Libraries`
- Output: `/home/sikmindz/Coding/Libraries/semantic-memory-forge/semantic-memory-forge-generic-rust-next-codex-context-20260612T222544Z.zip`
- Include roots: `2`
- External Cargo path dependency roots: `1`
- Profile: `generic-rust` requested as `auto`
- Mode: `next-codex-context`
- Package role: `next-codex-context`
- Strict: `True`
- Dry run: `False`
- Included files: `123`
- Included bytes: `1722148`
- Excluded files: `23`
- Pruned dirs: `2`
- Findings: `2` (`0` errors, `2` warnings)
- Archive zip-byte SHA-256: `cb17c0737bb7081324dc75217516e215f63560f8374437ed6ef333db2b4bb3be`
- Archive hash semantics: `zip-byte-sha256-not-canonical-content-hash`
- Content manifest SHA-256: `aadb4bbb5bea6075da969e336b7bfdbe61c66ddeb24be65687ac70b089e6f0cb`
- Ecosystems detected: `rust`
- Codex archive enabled: `True`
- Codex archive planned: `0`
- Codex archive moved: `0`
- Codex active stale after normalization: `0`
- Root Markdown archive enabled: `False`
- Root Markdown inspected: `2`
- Root Markdown protected: `2`
- Root Markdown candidates: `0`
- Root Markdown ambiguous: `0`
- Root Markdown moved: `0`
- Root Markdown collisions: `0`
- Root package archive enabled: `True`
- Root package inspected: `6`
- Root package protected: `6`
- Root package candidates: `0`
- Root package moved: `0`
- Root package skipped existing: `0`
- Root package collisions: `0`

## Ecosystem parity

| Ecosystem | Detected | Manifests | Missing expected | Dry-run status |
|---|---:|---:|---:|---|
| `rust` | `True` | 1 | 1 | `available-not-run` |
| `python` | `False` | 0 | 0 | `not-applicable` |
| `node` | `False` | 0 | 1 | `not-applicable` |
| `go` | `False` | 0 | 0 | `not-applicable` |
| `docker` | `False` | 0 | 0 | `not-applicable` |
| `git` | `False` | 0 | 0 | `not-applicable` |

## Decision provenance

- Decisions recorded: `147`
- Includes: `122`
- Excludes: `23`
- Pruned dirs: `2`

## Validation findings

| Severity | Code | Path | Detail |
|---|---|---|---|
| warning | `rust-expected-file-not-packaged` | `LICENSE` | rust adapter expected this existing file to be included. |
| warning | `node-expected-file-not-packaged` | `LICENSE` | node adapter expected this existing file to be included. |

## Included files by extension

| Extension | Count |
|---|---:|
| `.md` | 69 |
| `.rs` | 21 |
| `.json` | 16 |
| `.toml` | 5 |
| `<no-extension>` | 5 |
| `.py` | 3 |
| `.lock` | 2 |
| `.jsonl` | 1 |
| `.sh` | 1 |

## Included files by top-level path

| Top-level path | Count |
|---|---:|
| `semantic-memory-forge` | 20 |
| `stack-ids` | 16 |
| `.gitignore` | 1 |
| `.zpy` | 1 |
| `00_START_HERE.md` | 1 |
| `01_MASTER_ISSUE_TENSOR.json` | 1 |
| `02_PHASE_PLAN.md` | 1 |
| `03_IMPLEMENTATION_PLAYBOOK.md` | 1 |
| `03_TARGET_API_SPEC.md` | 1 |
| `04_EXACT_FILE_TOUCH_MAP.md` | 1 |
| `04_MATH_CONFORMANCE.md` | 1 |
| `05_ACCEPTANCE_GATES.md` | 1 |
| `05_TEST_AND_CONFORMANCE_PLAN.md` | 1 |
| `06_VALIDATION_COMMANDS.md` | 1 |
| `07_ROLLBACK_AND_QUARANTINE.md` | 1 |
| `08_FINAL_AUDITOR_HANDOFF.md` | 1 |
| `09_CODEX_FEATURES_AND_INSTALL.md` | 1 |
| `10_HOSTILE_AUDIT_CLAUDE.md` | 1 |
| `11_HOSTILE_AUDIT_GPT.md` | 1 |
| `11_HOSTILE_AUDIT_GPT_TENSOR.json` | 1 |
| `AGENTS.md` | 1 |
| `AUDIT_2026-04-01.md` | 1 |
| `BITEMPORAL_RUNTIME_HOSTILE_AUDIT_2026-06-02.md` | 1 |
| `CANONICAL_STACK_SPEC_V25_EFFECTIVE_CONSTITUTION_PROFILE_COMPOSITION_AND_OBLIGATION_FOLDING_RUNTIME.md` | 1 |
| `CANONICAL_STACK_SPEC_V26_ADVISORY_CONSTITUTIONAL_SEARCH_MINIMAL_EXCEPTION_SYNTHESIS_AND_POLICY_COUNTERFACTUAL_RUNTIME.md` | 1 |
| `CLAUDE.md` | 1 |
| `COMBINED_AUDIT_2026-04-01.md` | 1 |
| `CONFORMANCE_GATES.md` | 1 |
| `CONTRACT_AND_TEMPORAL_TRUTH_HARDENING.md` | 1 |
| `CRATE_HARDENING_MATRIX.md` | 1 |
| `Cargo.lock` | 1 |
| `Cargo.toml` | 1 |
| `EXECUTION_EVIDENCE_AND_REFERENCE_INTERPRETER_PLAN.md` | 1 |
| `HEARTBEAT.md` | 1 |
| `HNSW_BENCH_RESULTS_2026-06-02.md` | 1 |
| `HNSW_RESEARCH_2026-06-02.md` | 1 |
| `HOSTILE_AUDIT_SYNTHESIS_V5.md` | 1 |
| `IDENTITY.md` | 1 |
| `KERNEL_AND_REGION_RUNTIME_PLAN.md` | 1 |
| `LIBRARIES_AUDIT_2026-06-02.md` | 1 |
| `LIBRARIES_FINAL_REPORT_2026-06-02.md` | 1 |
| `LIBRARIES_FULL_STATE_DOSSIER_2026-05-27.md` | 1 |
| `LIBRARIES_HARDENING_AND_GAP_AUDIT_2026-05-27.md` | 1 |
| `LIBRARIES_HOSTILE_AUDIT_V30_CORRECTED_2026-05-29.md` | 1 |
| `LIBRARIES_IMPROVEMENT_DELTA_2026-05-27.md` | 1 |
| `LIBRARIES_MASTER_MATRIX_V8.md` | 1 |
| `LIBRARIES_MASTER_TENSOR_V8.json` | 1 |
| `LIBRARIES_PUBLISH_READY_2026-06-02.md` | 1 |
| `LIBRARIES_REMEDIATION_PLAN_2026-05-27.md` | 1 |
| `LIBRARIES_V30_HARDENING_ROADMAP.md` | 1 |
| `LIB_MASTER_ISSUE_TENSOR.json` | 1 |
| `Libraries-libraries-next-codex-context-20260612T222004Z.codex-archive.json` | 1 |
| `MASTER_ISSUE_TENSOR.json` | 1 |
| `MASTER_TENSOR.md` | 1 |
| `Makefile` | 1 |
| `OPERATOR_PASTE_FIRST.md` | 1 |
| `PACK_MANIFEST.json` | 1 |
| `PHASE_0_INVENTORY.md` | 1 |
| `PHASE_1_OWNERSHIP_LEDGER.md` | 1 |
| `PROMPT.md` | 1 |
| `README.md` | 1 |
| `RISK_REGISTER.md` | 1 |
| `SCOPE_NOTES.md` | 1 |
| `SNAPSHOT_2026-04-11.md` | 1 |
| `SOUL.md` | 1 |
| `SOURCE_BASIS.md` | 1 |
| `STATUS_DASHBOARD.md` | 1 |
| `STATUS_EVIDENCE_MANIFEST.json` | 1 |
| `SUB_WORKSPACES.md` | 1 |
| `SUPPORT_PROFILE.md` | 1 |
| `TEST_AND_CONFORMANCE_PLAN.md` | 1 |
| `TOOLS.md` | 1 |
| `USER.md` | 1 |
| `V9_IMPLEMENTATION_PLAYBOOK.md` | 1 |
| `claude_hard_audit_2026-03-30.md` | 1 |
| `deny.toml` | 1 |
| `gpt54_hard_audit_2026-03-30.md` | 1 |
| `gpt54_tensor_2026-03-30.json` | 1 |
| `hnsw-bench-receipt-hnsw_rs-20260602-174315.json` | 1 |
| `hnsw-bench-receipt-hnsw_rs-20260602-174742.json` | 1 |
| `hnsw-bench-receipt-hnsw_rs-20260602-175206.json` | 1 |
| `hnsw-bench-receipt-hnsw_rs-20260602-175704.json` | 1 |
| `hnsw-bench-receipt-hnsw_rs-20260602-180446.json` | 1 |
| `hnsw-bench-receipt-usearch-20260602-180715.json` | 1 |
| `hnsw-bench-receipt-usearch-20260602-180813.json` | 1 |
| `publish.sh` | 1 |
| `support_lane.toml` | 1 |
| `z.py` | 1 |
| `zip.py` | 1 |

## Exclusion reasons

| Reason | Count |
|---|---:|
| `archive-file` | 16 |
| `generated-sidecar` | 4 |
| `doc-binary-disabled` | 2 |
| `generated-output` | 1 |

## Sidecar files

- Manifest: `/home/sikmindz/Coding/Libraries/semantic-memory-forge/semantic-memory-forge-generic-rust-next-codex-context-20260612T222544Z.manifest.json`
- Markdown report: `/home/sikmindz/Coding/Libraries/semantic-memory-forge/semantic-memory-forge-generic-rust-next-codex-context-20260612T222544Z.report.md`
- Excluded file list: `/home/sikmindz/Coding/Libraries/semantic-memory-forge/semantic-memory-forge-generic-rust-next-codex-context-20260612T222544Z.excluded.json`
- Findings: `/home/sikmindz/Coding/Libraries/semantic-memory-forge/semantic-memory-forge-generic-rust-next-codex-context-20260612T222544Z.findings.json`

## Interpretation

This package has warnings. It is probably usable, but the warnings should be reviewed before using it as a Codex or audit handoff.
