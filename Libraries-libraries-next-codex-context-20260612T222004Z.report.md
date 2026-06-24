# Zip Source Certifier Report

## Summary

- Script version: `2026.05.22-p31`
- Created UTC: `2026-06-12T22:20:15Z`
- Root: `/home/sikmindz/Coding/Libraries`
- Archive root: `/home/sikmindz/Coding`
- Output: `/home/sikmindz/Coding/Libraries/Libraries-libraries-next-codex-context-20260612T222004Z.zip`
- Include roots: `2`
- External Cargo path dependency roots: `1`
- Profile: `libraries` requested as `auto`
- Mode: `next-codex-context`
- Package role: `next-codex-context`
- Strict: `True`
- Dry run: `False`
- Included files: `4810`
- Included bytes: `55911374`
- Excluded files: `466`
- Pruned dirs: `32`
- Findings: `38` (`0` errors, `37` warnings)
- Archive zip-byte SHA-256: `f9cd2b4ddd2698d01bd2a1caed13807e520e5bb46f188ce751fdaa6972032221`
- Archive hash semantics: `zip-byte-sha256-not-canonical-content-hash`
- Content manifest SHA-256: `880bfde47d9abe6fcd67452dc013eb3579a47b05eff91acba831df71bdaf41c2`
- Ecosystems detected: `rust, python, git`
- Codex archive enabled: `True`
- Codex archive planned: `192`
- Codex archive moved: `192`
- Codex active stale after normalization: `0`
- Root Markdown archive enabled: `False`
- Root Markdown inspected: `62`
- Root Markdown protected: `6`
- Root Markdown candidates: `23`
- Root Markdown ambiguous: `33`
- Root Markdown moved: `0`
- Root Markdown collisions: `0`
- Root package archive enabled: `True`
- Root package inspected: `109`
- Root package protected: `6`
- Root package candidates: `6`
- Root package moved: `6`
- Root package skipped existing: `0`
- Root package collisions: `0`

## Ecosystem parity

| Ecosystem | Detected | Manifests | Missing expected | Dry-run status |
|---|---:|---:|---:|---|
| `rust` | `True` | 132 | 3 | `available-not-run` |
| `python` | `True` | 1 | 2 | `available-not-run` |
| `node` | `False` | 0 | 1 | `not-applicable` |
| `go` | `False` | 0 | 0 | `not-applicable` |
| `docker` | `False` | 0 | 0 | `not-applicable` |
| `git` | `True` | 1 | 1 | `available-not-run` |

## Decision provenance

- Decisions recorded: `5308`
- Includes: `4810`
- Excludes: `466`
- Pruned dirs: `32`

## Validation findings

| Severity | Code | Path | Detail |
|---|---|---|---|
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/AI-Batch-Queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/ComfyUI-RS/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/LLM-Pipeline/Cargo.toml` | Cargo path dependency does not exist: ../living-memory/living-memory |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/LLM-Pipeline/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/Tauri-Queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/agent-graph/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/attestation-exchange/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../forge-memory-bridge |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../recursive-kernel-core |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../semantic-memory-forge |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../forge-memory-bridge |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../knowledge-runtime |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../semantic-memory |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../semantic-memory-forge |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/discovery-portfolio/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/federated-settlement/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/job-queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../assurance-runtime |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../authority-delegation |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../continuity-runtime |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../verification-policy |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/remote-oracle-admission/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries/_salvage_from_libraries2/Libraries2/spec-execution/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `script-ref-missing` | `Libraries/scr-runtime/scripts/run_completion_checks.sh` | Possible script reference not found: .codex/tools/auto_phase_runner.py |
| warning | `secret-content-named-secret-assignment` | `AGENT-SYSTEM.md` | Potential secret-like content detected at line 1309; value intentionally not printed. |
| warning | `secret-content-named-secret-assignment` | `Libraries/_salvage_from_libraries2/Libraries2/docs/benchmarks/run_forge_bench.py` | Potential secret-like content detected at line 152; value intentionally not printed. |
| warning | `secret-like-filename` | `Libraries/_salvage_from_libraries2/Libraries2/docs/13_settings_persistence_and_secret_handling.md` | File excluded because of secret-like-filename. |
| warning | `rust-expected-file-not-packaged` | `Cargo.lock` | rust adapter expected this existing file to be included. |
| warning | `rust-expected-file-not-packaged` | `Cargo.toml` | rust adapter expected this existing file to be included. |
| warning | `rust-expected-file-not-packaged` | `README.md` | rust adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `poly-kv/python/poly_kv/_native.pyi` | python adapter expected this existing file to be included. |
| warning | `python-expected-file-not-packaged` | `poly-kv/python/poly_kv/py.typed` | python adapter expected this existing file to be included. |
| warning | `node-expected-file-not-packaged` | `README.md` | node adapter expected this existing file to be included. |
| warning | `git-expected-file-not-packaged` | `.gitignore` | git adapter expected this existing file to be included. |
| info | `git-metadata-excluded` | `.git/` | Git metadata detected and intentionally excluded from transferable package contents. |

## Included files by extension

| Extension | Count |
|---|---:|
| `.json` | 1567 |
| `.md` | 1370 |
| `.rs` | 1258 |
| `.toml` | 163 |
| `.py` | 127 |
| `.sh` | 93 |
| `<no-extension>` | 54 |
| `.lock` | 41 |
| `.txt` | 39 |
| `.csv` | 37 |
| `.ts` | 18 |
| `.patch` | 8 |
| `.sql` | 7 |
| `.yml` | 7 |
| `.log` | 5 |
| `.jsonl` | 4 |
| `.ndjson` | 3 |
| `.template` | 3 |
| `.css` | 1 |
| `.html` | 1 |
| `.js` | 1 |
| `.pyi` | 1 |
| `.tsv` | 1 |
| `.typed` | 1 |

## Included files by top-level path

| Top-level path | Count |
|---|---:|
| `Libraries` | 4744 |
| `Gloss` | 13 |
| `00_README.md` | 1 |
| `01_OPERATOR_DECISION_BRIEF.md` | 1 |
| `02_SCOPE_AND_ASSUMPTIONS.md` | 1 |
| `03_REQUIRED_INPUTS.md` | 1 |
| `04_FORBIDDEN_CHANGES.md` | 1 |
| `05_RUN_ORDER.md` | 1 |
| `ACCEPTANCE_GATES.md` | 1 |
| `AGENT-SYSTEM.md` | 1 |
| `AGENTS-TEMPLATE.md` | 1 |
| `AGENTS.md` | 1 |
| `AGENT_LOG.md` | 1 |
| `Agent.md` | 1 |
| `Cat Info App.md` | 1 |
| `Coding-research-next-codex-context-20260525T185045Z.codex-archive.json` | 1 |
| `Coding.md` | 1 |
| `Director.md` | 1 |
| `FINAL_REPORT_TEMPLATE.md` | 1 |
| `GENERATED_FILE_TREE.txt` | 1 |
| `MANUAL_PHASE_INJECTIONS.md` | 1 |
| `MASTER_CODEBASE_REFERENCE2.md` | 1 |
| `MASTER_ISSUE_TENSOR.md` | 1 |
| `Medicine.md` | 1 |
| `PACK_METADATA.json` | 1 |
| `PHASE_00_PREFLIGHT.md` | 1 |
| `PHASE_01_LIBRARIES_CANONICAL_CLOSURE.md` | 1 |
| `PHASE_02_SALVAGE_TERMINAL_DECISIONS.md` | 1 |
| `PHASE_03_RESIDUAL_LIBRARIES2_REFS.md` | 1 |
| `PHASE_04_DOWNSTREAM_DEPENDENCY_REPAIR.md` | 1 |
| `PHASE_05_SEMANTIC_MEMORY_AND_GLOSS_BOUNDARY.md` | 1 |
| `PHASE_06_CLAIMLEDGER_FORGE_BOUNDARY.md` | 1 |
| `PHASE_07_GENERATED_ARTIFACT_HYGIENE.md` | 1 |
| `PHASE_08_VALIDATION_AND_RECEIPTS.md` | 1 |
| `PHASE_09_FINAL_AUDITOR_HANDOFF.md` | 1 |
| `PLANa.md` | 1 |
| `Phone.md` | 1 |
| `Pictures.md` | 1 |
| `Playground.md` | 1 |
| `Portal Doctor.md` | 1 |
| `ROLLBACK_PLAN.md` | 1 |
| `RecursiveOps.md` | 1 |
| `Research.md` | 1 |
| `STATEa.md` | 1 |
| `TRANSFER.md` | 1 |
| `VALIDATION_COMMANDS.md` | 1 |
| `WORKSPACE_MAP.md` | 1 |
| `backup.py` | 1 |
| `codex.md` | 1 |
| `gitdb.md` | 1 |
| `recall-codex.md` | 1 |
| `research-architectural-next-steps-2026-04-15.md` | 1 |
| `website.md` | 1 |
| `z.py` | 1 |
| `zip.py` | 1 |

## Exclusion reasons

| Reason | Count |
|---|---:|
| `unsupported-extension-or-basename` | 145 |
| `generated-sidecar` | 144 |
| `archive-file` | 64 |
| `log-disabled` | 59 |
| `binary-build-artifact` | 44 |
| `max-file-size-exceeded` | 5 |
| `doc-binary-disabled` | 3 |
| `generated-output` | 1 |
| `secret-like-filename` | 1 |

## Sidecar files

- Manifest: `/home/sikmindz/Coding/Libraries/Libraries-libraries-next-codex-context-20260612T222004Z.manifest.json`
- Markdown report: `/home/sikmindz/Coding/Libraries/Libraries-libraries-next-codex-context-20260612T222004Z.report.md`
- Excluded file list: `/home/sikmindz/Coding/Libraries/Libraries-libraries-next-codex-context-20260612T222004Z.excluded.json`
- Findings: `/home/sikmindz/Coding/Libraries/Libraries-libraries-next-codex-context-20260612T222004Z.findings.json`

## Interpretation

This package has warnings. It is probably usable, but the warnings should be reviewed before using it as a Codex or audit handoff.
