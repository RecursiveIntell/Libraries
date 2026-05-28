# Zip Source Certifier Report

## Summary

- Script version: `2026.05.22-p31`
- Created UTC: `2026-05-24T01:29:50Z`
- Root: `/home/sikmindz/Coding/Libraries2`
- Archive root: `/home/sikmindz/Coding`
- Output: `/home/sikmindz/Coding/Libraries2/Libraries2-generic-rust-next-codex-context-20260524T012947Z.zip`
- Include roots: `5`
- External Cargo path dependency roots: `4`
- Profile: `generic-rust` requested as `auto`
- Mode: `next-codex-context`
- Package role: `next-codex-context`
- Strict: `True`
- Dry run: `False`
- Included files: `1333`
- Included bytes: `6302816`
- Excluded files: `188`
- Pruned dirs: `9`
- Findings: `33` (`0` errors, `33` warnings)
- Archive zip-byte SHA-256: `1e7ada49f798c930cabd9bd20e7e66e3323e44e5c2eb95dd813abd68e22e68e1`
- Archive hash semantics: `zip-byte-sha256-not-canonical-content-hash`
- Content manifest SHA-256: `5f60e482f362bafbca7fea7ad5749719b117a9fe534a697ae407e517a70d902d`
- Ecosystems detected: `rust`
- Codex archive enabled: `True`
- Codex archive planned: `0`
- Codex archive moved: `0`
- Codex active stale after normalization: `0`
- Root Markdown archive enabled: `False`
- Root Markdown inspected: `5`
- Root Markdown protected: `2`
- Root Markdown candidates: `2`
- Root Markdown ambiguous: `1`
- Root Markdown moved: `0`
- Root Markdown collisions: `0`
- Root package archive enabled: `True`
- Root package inspected: `20`
- Root package protected: `4`
- Root package candidates: `6`
- Root package moved: `6`
- Root package skipped existing: `0`
- Root package collisions: `0`

## Ecosystem parity

| Ecosystem | Detected | Manifests | Missing expected | Dry-run status |
|---|---:|---:|---:|---|
| `rust` | `True` | 20 | 2 | `available-not-run` |
| `python` | `False` | 0 | 0 | `not-applicable` |
| `node` | `False` | 0 | 0 | `not-applicable` |
| `go` | `False` | 0 | 0 | `not-applicable` |
| `docker` | `False` | 0 | 0 | `not-applicable` |
| `git` | `False` | 0 | 0 | `not-applicable` |

## Decision provenance

- Decisions recorded: `1530`
- Includes: `1333`
- Excludes: `188`
- Pruned dirs: `9`

## Validation findings

| Severity | Code | Path | Detail |
|---|---|---|---|
| warning | `cargo-path-dep-missing` | `Libraries2/AI-Batch-Queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/ComfyUI-RS/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/LLM-Pipeline/Cargo.toml` | Cargo path dependency does not exist: ../living-memory/living-memory |
| warning | `cargo-path-dep-missing` | `Libraries2/LLM-Pipeline/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/Tauri-Queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/agent-graph/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/attestation-exchange/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../forge-memory-bridge |
| warning | `cargo-path-dep-missing` | `Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../recursive-kernel-core |
| warning | `cargo-path-dep-missing` | `Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../semantic-memory-forge |
| warning | `cargo-path-dep-missing` | `Libraries2/constraint-compiler/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../forge-memory-bridge |
| warning | `cargo-path-dep-missing` | `Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../knowledge-runtime |
| warning | `cargo-path-dep-missing` | `Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../semantic-memory |
| warning | `cargo-path-dep-missing` | `Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../semantic-memory-forge |
| warning | `cargo-path-dep-missing` | `Libraries2/demo-tauri-libraries/src-tauri/Cargo.toml` | Cargo path dependency does not exist: ../../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/discovery-portfolio/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/federated-settlement/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/job-queue/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../assurance-runtime |
| warning | `cargo-path-dep-missing` | `Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../authority-delegation |
| warning | `cargo-path-dep-missing` | `Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../continuity-runtime |
| warning | `cargo-path-dep-missing` | `Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/profile-runtime/Cargo.toml` | Cargo path dependency does not exist: ../verification-policy |
| warning | `cargo-path-dep-missing` | `Libraries2/remote-oracle-admission/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `cargo-path-dep-missing` | `Libraries2/spec-execution/Cargo.toml` | Cargo path dependency does not exist: ../stack-ids |
| warning | `context-package-command-evidence-missing` | `/` | Context/audit package manifest does not include command-run evidence (commands_run.log, commands_run.receipts.jsonl, COMMAND_RECEIPTS.jsonl, COMMAND_EXECUTION_RECEIPTS.jsonl, or *_COMMANDS_RUN.md). |
| warning | `missing-source-root` | `/` | Missing src or crates. generic-rust profile expects src/ or crates/. |
| warning | `secret-content-named-secret-assignment` | `AGENT-SYSTEM.md` | Potential secret-like content detected at line 1309; value intentionally not printed. |
| warning | `secret-content-named-secret-assignment` | `Libraries2/docs/benchmarks/run_forge_bench.py` | Potential secret-like content detected at line 152; value intentionally not printed. |
| warning | `secret-like-filename` | `Libraries2/docs/13_settings_persistence_and_secret_handling.md` | File excluded because of secret-like-filename. |
| warning | `rust-expected-file-not-packaged` | `Cargo.lock` | rust adapter expected this existing file to be included. |
| warning | `rust-expected-file-not-packaged` | `Cargo.toml` | rust adapter expected this existing file to be included. |

## Included files by extension

| Extension | Count |
|---|---:|
| `.json` | 681 |
| `.rs` | 299 |
| `.md` | 260 |
| `.toml` | 25 |
| `<no-extension>` | 15 |
| `.lock` | 10 |
| `.ts` | 9 |
| `.txt` | 8 |
| `.sql` | 7 |
| `.py` | 6 |
| `.sh` | 6 |
| `.csv` | 2 |
| `.yml` | 2 |
| `.css` | 1 |
| `.html` | 1 |
| `.js` | 1 |

## Included files by top-level path

| Top-level path | Count |
|---|---:|
| `Libraries2` | 1253 |
| `Libraries` | 41 |
| `Gloss` | 13 |
| `AGENT-SYSTEM.md` | 1 |
| `AGENTS-TEMPLATE.md` | 1 |
| `Agent.md` | 1 |
| `Cat Info App.md` | 1 |
| `Coding-research-next-codex-context-20260524T012015Z.codex-archive.json` | 1 |
| `Coding.md` | 1 |
| `Director.md` | 1 |
| `MASTER_CODEBASE_REFERENCE2.md` | 1 |
| `Medicine.md` | 1 |
| `PLANa.md` | 1 |
| `Phone.md` | 1 |
| `Pictures.md` | 1 |
| `Playground.md` | 1 |
| `Portal Doctor.md` | 1 |
| `RecursiveOps.md` | 1 |
| `Research.md` | 1 |
| `STATEa.md` | 1 |
| `TRANSFER.md` | 1 |
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
| `unsupported-extension-or-basename` | 130 |
| `binary-build-artifact` | 44 |
| `archive-file` | 8 |
| `max-file-size-exceeded` | 3 |
| `doc-binary-disabled` | 1 |
| `generated-output` | 1 |
| `secret-like-filename` | 1 |

## Sidecar files

- Manifest: `/home/sikmindz/Coding/Libraries2/Libraries2-generic-rust-next-codex-context-20260524T012947Z.manifest.json`
- Markdown report: `/home/sikmindz/Coding/Libraries2/Libraries2-generic-rust-next-codex-context-20260524T012947Z.report.md`
- Excluded file list: `/home/sikmindz/Coding/Libraries2/Libraries2-generic-rust-next-codex-context-20260524T012947Z.excluded.json`
- Findings: `/home/sikmindz/Coding/Libraries2/Libraries2-generic-rust-next-codex-context-20260524T012947Z.findings.json`

## Interpretation

This package has warnings. It is probably usable, but the warnings should be reviewed before using it as a Codex or audit handoff.
