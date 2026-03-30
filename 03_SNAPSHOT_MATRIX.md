# 03_SNAPSHOT_MATRIX

## High-level read

This snapshot is not a loose experiment pile. It has a real center of gravity.

The problem is not “missing architecture.”
The problem is **front-door truth, naming honesty, and last-mile convergence**.

## Repo-level metrics from this pass

| Metric | Value |
|---|---:|
| Workspace members | 30 |
| Default members | 29 |
| Explicit excludes | 19 |
| Support-profile closeout lane | 17 crates |
| Top-level doc/json/txt files at root | 32 |
| Non-archive meta files under docs/plans/prompts/scaffolds/reference/snippets/schemas/scripts | ~465 |
| Duplicate `SurfaceStatus` definitions | 5 |
| `target-*` directories embedded in the “source-clean” archive | 6 |

## Script reality

| Check | Result |
|---|---|
| `check_pack_truth.sh` | **fail** |
| `check_repo_surface.sh` | pass |
| `check_doc_truth.sh` | pass |
| `check_manifest_truth.sh` | pass |
| `check_hotspot_budgets.sh` | pass |
| `check_schema_registry_uniqueness.sh` | pass |
| `check_mirror_discipline.sh` | pass |
| `check_no_prod_panics.sh` | pass (narrow scope only) |
| `check_public_type_drift.py` | pass |
| `check_root_archive_manifest.py` | **fail** |
| `check_public_api_docs.py` | pass |
| `check_closeout_receipt.py` | pass |

## Crate signal matrix (src-only scan)

| Crate | Source LOC | Doc comments | Prod unwraps | Read |
|---|---:|---:|---:|---|
| `semantic-memory` | 15,458 | 978 | 34 | real storage/query core |
| `forge-pilot` | 8,873 | 0 | 24 | real orchestrator, badly underdocumented |
| `living-memory/living-memory` | 8,300 | 703 | 0 | real evidence/export engine |
| `LLM-Pipeline` (excluded) | 7,948 | 1,255 | 200 | substantial excluded satellite |
| `knowledge-runtime` | 5,152 | 809 | 60 | real runtime, some sharp unwraps |
| `semantic-memory-forge` | 4,549 | 362 | 254 | real authority/export lane, still panic-heavy |
| `job-queue` (excluded) | 3,132 | 435 | 358 | useful but outside certified lane |
| `stack-ids` | 2,902 | 504 | 96 | foundational, still too panic-prone |
| `forge-memory-bridge` | 2,339 | 315 | 74 | critical transform lane |
| `llm-tool-runtime` | 2,063 | 0 | 18 | meaningful seam, almost no rustdoc |
| `verification-control` | 1,791 | 20 | 2 | meaningful but under-explained |
| `kernel-conformance` | 1,262 | 0 | 76 | meaningful and under-explained |
| `constraint-compiler` | 1,177 | 1 | 12 | real core logic, still nearly undocumented |

## Thin governance/runtime surfaces (src-only)

| Crate | Source LOC | Read |
|---|---:|---|
| `mechanism-runtime` | 192 | schema carrier wearing a runtime name |
| `constitutional-memory` | 230 | schema carrier wearing a runtime name |
| `discovery-portfolio` | 249 | schema carrier wearing a runtime name |
| `spec-execution` | 352 | schema carrier wearing a runtime name |
| `federated-settlement` | 391 | schema carrier wearing a runtime name |
| `verification-adjudication` | 541 | real but still very small |
| `verification-policy` | 752 | real but small |
| `kernel-oracles` | 936 | thin but meaningful |
| `contract-schema-gen` | 955 | central enough to deserve docs and stronger guardrails |

## Repeated hotspot files

| File | Source LOC | Notes |
|---|---:|---|
| `semantic-memory-forge/src/envelope.rs` | 2,186 | core export/evidence file, also high unwrap density |
| `living-memory/living-memory/src/lab/evidence.rs` | 2,119 | major evidence file |
| `semantic-memory/src/projection_storage.rs` | 1,965 | storage hotspot |
| `stack-ids/src/ids.rs` | 1,952 | primitive hotspot |
| `verification-control/src/lib.rs` | 1,705 | control hotspot |
| `forge-pilot/src/main_support/mod.rs` | 1,556 | orchestration hotspot |
| `forge-memory-bridge/src/transform.rs` | 1,386 | bridge hotspot |
| `LLM-Pipeline/src/llm_call.rs` | 1,187 | excluded but large |
| `constraint-compiler/src/lib.rs` | 1,177 | one-file compiler hotspot |
| `forge-pilot/src/loop_runner.rs` | 1,132 | still unsplit |

## Specific duplicate type drift found

`SurfaceStatus` is duplicated in:

- `spec-execution/src/lib.rs`
- `mechanism-runtime/src/lib.rs`
- `federated-settlement/src/lib.rs`
- `discovery-portfolio/src/lib.rs`
- `constitutional-memory/src/lib.rs`

## Package-surface problem in one line

The repo already knows how to talk about truth.
It does **not yet** reliably publish that truth from the front door.
