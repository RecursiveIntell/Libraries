# Zip Source Certifier Report

## Summary

- Script version: `2026.05.22-p31`
- Created UTC: `2026-05-24T01:20:33Z`
- Root: `/home/sikmindz/Coding/Libraries`
- Archive root: `/home/sikmindz/Coding/Libraries`
- Output: `/home/sikmindz/Coding/Libraries/Libraries-libraries-next-codex-context-20260524T012026Z.zip`
- Include roots: `1`
- External Cargo path dependency roots: `0`
- Profile: `libraries` requested as `auto`
- Mode: `next-codex-context`
- Package role: `next-codex-context`
- Strict: `True`
- Dry run: `False`
- Included files: `3151`
- Included bytes: `25856397`
- Excluded files: `175`
- Pruned dirs: `39`
- Findings: `2` (`0` errors, `1` warnings)
- Archive zip-byte SHA-256: `8284e2590d37ff491d36c9a7b3b17d0d5ea4ccec44bebeea86a7a8d7439d4b25`
- Archive hash semantics: `zip-byte-sha256-not-canonical-content-hash`
- Content manifest SHA-256: `bfc6880eeaf86c1a4fc34d8e2c6f314b0d182f4eb0e49ab6494d47f6f9b33ed2`
- Ecosystems detected: `rust, python, git`
- Codex archive enabled: `True`
- Codex archive planned: `86`
- Codex archive moved: `86`
- Codex active stale after normalization: `0`
- Root Markdown archive enabled: `False`
- Root Markdown inspected: `50`
- Root Markdown protected: `14`
- Root Markdown candidates: `18`
- Root Markdown ambiguous: `18`
- Root Markdown moved: `0`
- Root Markdown collisions: `0`
- Root package archive enabled: `True`
- Root package inspected: `93`
- Root package protected: `6`
- Root package candidates: `20`
- Root package moved: `20`
- Root package skipped existing: `0`
- Root package collisions: `0`

## Ecosystem parity

| Ecosystem | Detected | Manifests | Missing expected | Dry-run status |
|---|---:|---:|---:|---|
| `rust` | `True` | 93 | 0 | `available-not-run` |
| `python` | `True` | 1 | 0 | `available-not-run` |
| `node` | `False` | 0 | 0 | `not-applicable` |
| `go` | `False` | 0 | 0 | `not-applicable` |
| `docker` | `False` | 0 | 0 | `not-applicable` |
| `git` | `True` | 1 | 0 | `available-not-run` |

## Decision provenance

- Decisions recorded: `3365`
- Includes: `3151`
- Excludes: `175`
- Pruned dirs: `39`

## Validation findings

| Severity | Code | Path | Detail |
|---|---|---|---|
| warning | `script-ref-missing` | `scr-runtime/scripts/run_completion_checks.sh` | Possible script reference not found: .codex/tools/auto_phase_runner.py |
| info | `git-metadata-excluded` | `.git/` | Git metadata detected and intentionally excluded from transferable package contents. |

## Included files by extension

| Extension | Count |
|---|---:|
| `.md` | 1116 |
| `.json` | 868 |
| `.rs` | 719 |
| `.toml` | 120 |
| `.py` | 109 |
| `.sh` | 86 |
| `.csv` | 48 |
| `.lock` | 24 |
| `.txt` | 23 |
| `<no-extension>` | 16 |
| `.yml` | 5 |
| `.log` | 4 |
| `.jsonl` | 3 |
| `.ndjson` | 3 |
| `.template` | 3 |
| `.patch` | 1 |
| `.pyi` | 1 |
| `.tsv` | 1 |
| `.typed` | 1 |

## Included files by top-level path

| Top-level path | Count |
|---|---:|
| `AiDENs` | 966 |
| `poly-kv` | 279 |
| `schemas` | 211 |
| `fib-quant` | 178 |
| `turbo-quant` | 178 |
| `scr-runtime` | 146 |
| `semantic-memory` | 144 |
| `examples` | 134 |
| `turbo-semantic` | 127 |
| `contracts` | 110 |
| `forge-pilot` | 92 |
| `living-memory` | 77 |
| `Primitives` | 58 |
| `scripts` | 46 |
| `knowledge-runtime` | 38 |
| `assurance-runtime` | 18 |
| `continuity-runtime` | 16 |
| `profile-runtime` | 16 |
| `effect-runtime` | 15 |
| `stack-ids` | 15 |
| `verification-policy` | 15 |
| `authority-delegation` | 14 |
| `forge-memory-bridge` | 14 |
| `semantic-memory-forge` | 14 |
| `docs` | 12 |
| `kernel-conformance` | 12 |
| `llm-tool-runtime` | 12 |
| `phase_prompts` | 10 |
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
| `prompts` | 5 |
| `recursive-kernel-core` | 5 |
| `verification-calibration` | 5 |
| `kernel-execution` | 4 |
| `kernel-oracles` | 4 |
| `evidence` | 3 |
| `.agents` | 2 |
| `matrices` | 2 |
| `overlays` | 2 |
| `verification` | 2 |
| `.github` | 1 |
| `.gitignore` | 1 |
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
| `4` | 1 |
| `AGENTS.md` | 1 |
| `AUDIT_2026-04-01.md` | 1 |
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
| `HOSTILE_AUDIT_SYNTHESIS_V5.md` | 1 |
| `KERNEL_AND_REGION_RUNTIME_PLAN.md` | 1 |
| `LIBRARIES_MASTER_MATRIX_V8.md` | 1 |
| `LIBRARIES_MASTER_TENSOR_V8.json` | 1 |
| `LIB_MASTER_ISSUE_TENSOR.json` | 1 |
| `MASTER_ISSUE_TENSOR.json` | 1 |
| `MASTER_TENSOR.md` | 1 |
| `Makefile` | 1 |
| `OPERATOR_PASTE_FIRST.md` | 1 |
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
| `fixtures` | 1 |
| `gpt54_hard_audit_2026-03-30.md` | 1 |
| `gpt54_tensor_2026-03-30.json` | 1 |
| `manual_backstop_prompts` | 1 |
| `release` | 1 |
| `support_lane.toml` | 1 |
| `z.py` | 1 |
| `zip.py` | 1 |

## Exclusion reasons

| Reason | Count |
|---|---:|
| `generated-sidecar` | 96 |
| `archive-file` | 47 |
| `log-disabled` | 24 |
| `unsupported-extension-or-basename` | 5 |
| `doc-binary-disabled` | 2 |
| `generated-output` | 1 |

## Sidecar files

- Manifest: `/home/sikmindz/Coding/Libraries/Libraries-libraries-next-codex-context-20260524T012026Z.manifest.json`
- Markdown report: `/home/sikmindz/Coding/Libraries/Libraries-libraries-next-codex-context-20260524T012026Z.report.md`
- Excluded file list: `/home/sikmindz/Coding/Libraries/Libraries-libraries-next-codex-context-20260524T012026Z.excluded.json`
- Findings: `/home/sikmindz/Coding/Libraries/Libraries-libraries-next-codex-context-20260524T012026Z.findings.json`

## Interpretation

This package has warnings. It is probably usable, but the warnings should be reviewed before using it as a Codex or audit handoff.
