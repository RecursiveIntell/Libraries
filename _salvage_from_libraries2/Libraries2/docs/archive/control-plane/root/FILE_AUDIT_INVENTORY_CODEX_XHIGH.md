# File Audit Inventory — Codex XHigh

This document preserves the **baseline static audit** that produced the XHigh control plane.

It is authoritative for the audited snapshot that XHigh started from, not for live progress after
subsequent repo-surface work. For current issue status, use `MASTER_ISSUE_MATRIX_CODEX_XHIGH.md`
and the root `README.md`.

The authoritative machine-readable inventory is:

- `FILE_AUDIT_INVENTORY_CODEX_XHIGH.csv`

That CSV contains **one row per file** for all `321` files in the snapshot.

## Snapshot summary

- `24` Cargo manifests
- `297` Rust files
- `87,499` Rust LOC
- `1,311` test annotations
- `108` `deprecated` hits
- `174` `legacy` hits
- `167` `compat` / `compatibility` hits
- `0` README / AGENTS files inside the snapshot

## Top-level layout

`.parser-lib, AI-Batch-Queue, ComfyUI-RS, LLM-Pipeline, Ollama-Vision-RS, Primitives, Tauri-Queue, agent-graph, forge-memory-bridge, job-queue, knowledge-runtime, living-memory, semantic-memory, semantic-memory-forge, stack-ids`

## Per-crate summary

| crate | group | rs_lines | test_annotations | docs_signal | compat_signal | packaging_gaps |
| --- | --- | --- | --- | --- | --- | --- |
| forge-engine | core_authority_lane | 11636 | 153 | weak | medium | missing_readme,missing_repository |
| forge-memory-bridge | core_authority_lane | 1929 | 34 | medium | high | missing_readme,missing_repository |
| knowledge-runtime | core_authority_lane | 8580 | 125 | strong | medium | missing_readme |
| semantic-memory | core_authority_lane | 25628 | 289 | strong | high | missing_readme |
| semantic-memory-forge | core_authority_lane | 1045 | 13 | strong | low | missing_readme,missing_repository |
| stack-ids | core_authority_lane | 1223 | 40 | medium | high | missing_readme,missing_repository |
| agent-graph | execution_and_workflow | 10547 | 135 | strong | high | missing_readme,missing_repository |
| ai-batch-queue | execution_and_workflow | 2766 | 56 | medium | low | missing_readme,placeholder_repository |
| job-queue | execution_and_workflow | 3511 | 43 | medium | high | missing_readme,missing_repository |
| llm-output-parser | execution_and_workflow | 3552 | 144 | strong | low | missing_readme,missing_repository |
| llm-pipeline | execution_and_workflow | 8544 | 212 | strong | high | missing_readme |
| tauri-queue | execution_and_workflow | 1305 | 29 | medium | medium | missing_readme,placeholder_repository |
| cea-core | primitives | 2210 | 7 | medium | low | missing_readme,missing_repository |
| cea-sqlite | primitives | 182 | 0 | weak | low | missing_readme,missing_repository |
| cea-store | primitives | 177 | 0 | weak | low | missing_readme,missing_repository |
| check-runner | primitives | 537 | 0 | weak | low | missing_readme,missing_repository |
| effect-signature | primitives | 37 | 0 | weak | low | missing_readme,missing_repository |
| forge-policy | primitives | 366 | 2 | weak | low | missing_readme,missing_repository |
| mindstate-core | primitives | 115 | 0 | weak | low | missing_readme,missing_repository |
| sandbox-workspace | primitives | 177 | 0 | weak | low | missing_readme,missing_repository |
| stabilizer-core | primitives | 277 | 0 | weak | low | missing_readme,missing_repository |
| typed-patch | primitives | 771 | 1 | weak | low | missing_readme,missing_repository |
| comfyui-rs | satellite_clients | 1570 | 22 | strong | low | missing_readme,missing_repository |
| ollama-vision | satellite_clients | 814 | 6 | strong | low | missing_readme,missing_repository |

## Manifest metadata gaps

| crate | manifest | problem | detail |
| --- | --- | --- | --- |
| llm-output-parser | .parser-lib/Cargo.toml | missing_readme_declared | README.md |
| llm-output-parser | .parser-lib/Cargo.toml | missing_repository | - |
| ai-batch-queue | AI-Batch-Queue/Cargo.toml | missing_readme_metadata | - |
| ai-batch-queue | AI-Batch-Queue/Cargo.toml | placeholder_repository | https://github.com/yourusername/ai-batch-queue |
| comfyui-rs | ComfyUI-RS/Cargo.toml | missing_readme_metadata | - |
| comfyui-rs | ComfyUI-RS/Cargo.toml | missing_repository | - |
| llm-pipeline | LLM-Pipeline/Cargo.toml | missing_readme_metadata | - |
| ollama-vision | Ollama-Vision-RS/Cargo.toml | missing_readme_metadata | - |
| ollama-vision | Ollama-Vision-RS/Cargo.toml | missing_repository | - |
| cea-core | Primitives/cea-core/Cargo.toml | missing_readme_metadata | - |
| cea-core | Primitives/cea-core/Cargo.toml | missing_repository | - |
| cea-sqlite | Primitives/cea-sqlite/Cargo.toml | missing_readme_metadata | - |
| cea-sqlite | Primitives/cea-sqlite/Cargo.toml | missing_repository | - |
| cea-store | Primitives/cea-store/Cargo.toml | missing_readme_metadata | - |
| cea-store | Primitives/cea-store/Cargo.toml | missing_repository | - |
| check-runner | Primitives/check-runner/Cargo.toml | missing_readme_metadata | - |
| check-runner | Primitives/check-runner/Cargo.toml | missing_repository | - |
| effect-signature | Primitives/effect-signature/Cargo.toml | missing_readme_metadata | - |
| effect-signature | Primitives/effect-signature/Cargo.toml | missing_repository | - |
| forge-policy | Primitives/forge-policy/Cargo.toml | missing_readme_metadata | - |
| forge-policy | Primitives/forge-policy/Cargo.toml | missing_repository | - |
| mindstate-core | Primitives/mindstate-core/Cargo.toml | missing_readme_metadata | - |
| mindstate-core | Primitives/mindstate-core/Cargo.toml | missing_repository | - |
| sandbox-workspace | Primitives/sandbox-workspace/Cargo.toml | missing_readme_metadata | - |
| sandbox-workspace | Primitives/sandbox-workspace/Cargo.toml | missing_repository | - |
| stabilizer-core | Primitives/stabilizer-core/Cargo.toml | missing_readme_metadata | - |
| stabilizer-core | Primitives/stabilizer-core/Cargo.toml | missing_repository | - |
| typed-patch | Primitives/typed-patch/Cargo.toml | missing_readme_metadata | - |
| typed-patch | Primitives/typed-patch/Cargo.toml | missing_repository | - |
| tauri-queue | Tauri-Queue/Cargo.toml | missing_readme_metadata | - |
| tauri-queue | Tauri-Queue/Cargo.toml | placeholder_repository | https://github.com/yourusername/tauri-queue |
| agent-graph | agent-graph/Cargo.toml | missing_readme_metadata | - |
| agent-graph | agent-graph/Cargo.toml | missing_repository | - |
| forge-memory-bridge | forge-memory-bridge/Cargo.toml | missing_readme_metadata | - |
| forge-memory-bridge | forge-memory-bridge/Cargo.toml | missing_repository | - |
| job-queue | job-queue/Cargo.toml | missing_readme_metadata | - |
| job-queue | job-queue/Cargo.toml | missing_repository | - |
| knowledge-runtime | knowledge-runtime/Cargo.toml | missing_readme_metadata | - |
| forge-engine | living-memory/living-memory/Cargo.toml | missing_readme_metadata | - |
| forge-engine | living-memory/living-memory/Cargo.toml | missing_repository | - |
| semantic-memory | semantic-memory/Cargo.toml | missing_readme_declared | README.md |
| semantic-memory-forge | semantic-memory-forge/Cargo.toml | missing_readme_metadata | - |
| semantic-memory-forge | semantic-memory-forge/Cargo.toml | missing_repository | - |
| stack-ids | stack-ids/Cargo.toml | missing_readme_metadata | - |
| stack-ids | stack-ids/Cargo.toml | missing_repository | - |

## Compatibility / migration hotspots (source files)

| path | compat_load | deprecated_hits | legacy_hits | compat_hits | unwrap_hits |
| --- | --- | --- | --- | --- | --- |
| forge-memory-bridge/src/legacy.rs | 51 | 5 | 35 | 11 | 7 |
| forge-memory-bridge/src/lib.rs | 34 | 11 | 9 | 14 | 0 |
| LLM-Pipeline/src/exec_ctx.rs | 33 | 10 | 17 | 6 | 0 |
| job-queue/src/events.rs | 32 | 20 | 0 | 12 | 0 |
| agent-graph/src/event_sink.rs | 32 | 12 | 7 | 13 | 0 |
| semantic-memory/src/lib.rs | 30 | 10 | 10 | 10 | 14 |
| stack-ids/src/trace.rs | 14 | 0 | 10 | 4 | 11 |
| semantic-memory/src/episodes.rs | 14 | 0 | 7 | 7 | 0 |
| agent-graph/src/graph.rs | 14 | 1 | 10 | 3 | 0 |
| LLM-Pipeline/src/trace.rs | 12 | 5 | 1 | 6 | 2 |
| job-queue/src/types.rs | 11 | 0 | 6 | 5 | 0 |
| stack-ids/src/scope.rs | 10 | 0 | 6 | 4 | 3 |
| job-queue/src/lib.rs | 10 | 5 | 3 | 2 | 0 |
| living-memory/living-memory/src/lab/evidence.rs | 8 | 0 | 4 | 4 | 0 |
| Tauri-Queue/src/lib.rs | 8 | 3 | 3 | 2 | 0 |

## Non-test unwrap hotspots (review, not automatic defect)

These are not all bugs.  
They are simply the highest-density non-test `unwrap()` concentrations worth human review.

| path | unwrap_hits | expect_hits | lines |
| --- | --- | --- | --- |
| job-queue/src/db.rs | 170 | 0 | 1414 |
| AI-Batch-Queue/src/queue.rs | 61 | 0 | 806 |
| LLM-Pipeline/src/llm_call.rs | 33 | 0 | 1341 |
| LLM-Pipeline/src/output_parser.rs | 30 | 0 | 372 |
| .parser-lib/src/list.rs | 27 | 0 | 567 |
| .parser-lib/src/repair.rs | 26 | 0 | 656 |
| forge-memory-bridge/src/transform.rs | 23 | 0 | 728 |
| Primitives/cea-core/src/tests.rs | 19 | 0 | 410 |
| knowledge-runtime/src/entity/registry.rs | 18 | 0 | 630 |
| .parser-lib/src/json.rs | 18 | 0 | 455 |
| semantic-memory/src/lib.rs | 14 | 0 | 4420 |
| .parser-lib/src/xml.rs | 12 | 0 | 254 |
| .parser-lib/src/number.rs | 12 | 0 | 287 |
| stack-ids/src/trace.rs | 11 | 0 | 414 |
| ComfyUI-RS/src/client.rs | 11 | 0 | 832 |

## Crate roots lacking crate-level rustdoc

| crate | path | issue |
| --- | --- | --- |
| cea-sqlite | Primitives/cea-sqlite/src/lib.rs | no_crate_level_rustdoc |
| cea-store | Primitives/cea-store/src/lib.rs | no_crate_level_rustdoc |
| check-runner | Primitives/check-runner/src/lib.rs | no_crate_level_rustdoc |
| effect-signature | Primitives/effect-signature/src/lib.rs | no_crate_level_rustdoc |
| forge-policy | Primitives/forge-policy/src/lib.rs | no_crate_level_rustdoc |
| mindstate-core | Primitives/mindstate-core/src/lib.rs | no_crate_level_rustdoc |
| sandbox-workspace | Primitives/sandbox-workspace/src/lib.rs | no_crate_level_rustdoc |
| stabilizer-core | Primitives/stabilizer-core/src/lib.rs | no_crate_level_rustdoc |
| typed-patch | Primitives/typed-patch/src/lib.rs | no_crate_level_rustdoc |
| forge-engine | living-memory/living-memory/src/lib.rs | no_crate_level_rustdoc |

## Audit interpretation

### What this means
- The snapshot is **code-rich**.
- The canonical lane is **architecturally credible**.
- The repo surface is **thin to nonexistent** inside the archive.
- Several public crates are still **mid-migration**.
- The primitive suite is **under-explained** relative to how important it may become.

### What this does not mean
- A high unwrap count automatically means low quality.
- A missing README means the code is weak.
- A crate with few tests is necessarily broken.

It means Codex should:
1. fix the repo front door first,
2. normalize packaging/docs,
3. then finish the remaining architecture and compatibility debt from a stable control plane.
