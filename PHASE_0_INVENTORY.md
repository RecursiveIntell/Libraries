# PHASE 0 INVENTORY — Libraries Root Workspace Stabilization Pass
Date: 2026-05-28
Agent: kimi-k2.6
Scope: ~/Coding/Libraries (RecursiveIntell canonical shared-library workspace)

## Git State
- Repository: /home/sikmindz/Coding/Libraries
- HEAD: 0e621009013a7366bb40f6b21274f2aea0fbf1a1 Phase 5: Final audit — cargo fmt cleanup
- Dirty: yes
  - Deleted: 02_MASTER_ISSUE_MATRIX.md, 06_RISK_REGISTER.md, Libraries-libraries-next-codex-context-* (5 files)
  - Modified: forge-memory-bridge, poly-kv, stack-ids, turbo-quant (all 0-byte diff; likely stale submodule index entries)
  - Untracked: Libraries-libraries-next-codex-context-20260528T032031Z.* (5 files), docs/source-packages/archive/20260528T032033Z/
- Submodules: broken — .gitmodules missing but git index contains submodule entries for fib-quant (and possibly forge-memory-bridge, poly-kv, stack-ids, turbo-quant). `git submodule status` fails.

## Workspaces Found
1. **ROOT** `Cargo.toml` — 60 members, resolver = 2
2. `AiDENs/Cargo.toml` — 36 member crates, separate workspace, path-deps point to ../crates in root
3. `Primitives/Cargo.toml` — 10 member crates, separate workspace
4. `poly-kv/Cargo.toml` — 3 member crates, separate workspace
5. `scr-runtime/Cargo.toml` — 4 member crates, separate workspace

## Root Workspace Members (60)
All member directories exist on disk and have Cargo.toml:
- agent-graph, agent-guard, ai-batch-queue, assurance-runtime, attestation-exchange, authority-delegation, bitemporal-runtime, boundary-compiler, claim-ledger, comfyui-rs, constraint-compiler, constitutional-memory, continuity-runtime, contract-schema-gen, discovery-portfolio, effect-runtime, fib-quant, federated-settlement, forge-memory-bridge, forge-pilot, job-queue, kernel-conformance, kernel-execution, kernel-oracles, knowledge-runtime, living-memory/living-memory, llm-output-parser, llm-pipeline, poly-kv/crates/quant-codec-core, poly-kv/crates/poly-kv, llm-tool-runtime, mechanism-runtime, ollama-vision, Primitives/cea-core, Primitives/cea-sqlite, Primitives/cea-store, Primitives/check-runner, Primitives/effect-signature, Primitives/forge-policy, Primitives/mindstate-core, Primitives/sandbox-workspace, Primitives/stabilizer-core, Primitives/typed-patch, quant-governor, quant-eval, receipt-bench, profile-runtime, recursive-kernel-core, remote-oracle-admission, semantic-memory, semantic-memory-forge, scr-runtime-compression, spec-execution, stack-ids, tauri-queue, turbo-quant, verification-adjudication, verification-calibration, verification-control, verification-policy

## Crates on Disk but NOT in Root Workspace
- AiDENs/* (36 crates + fixture + scaffold)
- AiDENs/scaffold/boundary-compiler-core
- Primitives/ (as workspace root, not individual crates)
- poly-kv/ (workspace root), poly-kv/crates/poly-kv-python
- scr-runtime/ (workspace root)
- turbo-semantic/ (not in any workspace, duplicate name)

## Salvage Directory
- `_salvage_from_libraries2/` contains 20 Libraries2-era crates. All excluded from root workspace. Not built.

## Path Dependencies
- **Zero** external path dependencies (all path deps resolve within ~/Coding/Libraries).
- `living-memory/living-memory` has path deps using `../../` to reach root workspace crates and `../../Primitives/...`. All internal.

## Crate Name Duplicates
- `semantic-memory` declared by both `semantic-memory/` (canonical v0.5.0, Apache-2.0, in workspace) and `turbo-semantic/` (same name v0.5.0, MIT, NOT in workspace).
- `boundary-compiler-core` exists in `AiDENs/scaffold/` (not in root workspace) and also `boundary-compiler/` in root.

## Build / Check / Test / Clippy Status
| Command | Result | Notes |
|---|---|---|
| cargo fmt --all -- --check | PASS | clean |
| cargo check --workspace --all-targets | PASS | no compile errors |
| cargo test --workspace --all-targets | PASS | 2328 passed, 0 failed, 1 ignored across 254 suites with tests; 69 suites with 0 tests |
| cargo clippy --workspace --all-targets -- -D warnings | FAIL | 14 crates produce clippy errors under -D warnings |
| AiDENs workspace cargo check | FAIL | aidens-contracts test compile error (method not found) |
| Primitives workspace cargo check | PASS | clean |
| poly-kv workspace cargo check | PASS | clean |
| scr-runtime workspace cargo check | PASS | clean |

## Clippy-Failing Crates (14)
1. `ai-batch-queue` — manual checked division (1 error)
2. `bitemporal-runtime` — or_insert_with default, collapsible if (3 errors)
3. `boundary-compiler` — derivable impls, collapsible match, redundant closure (5 errors)
4. `claim-ledger` — identical blocks, map_or simplification, needless borrow, derivable impl (4 errors)
5. `comfyui-rs` — borrowed expression trait (1 error)
6. `job-queue` — redundant closures, collapsible match (3 errors)
7. `llm-tool-runtime` — collapsible match x5 (5 errors)
8. `quant-governor` — derivable impls (2 errors)
9. `receipt-bench` — dead code, derivable impls, needless borrow, unnecessary cast, redundant closure (17 errors)
10. `semantic-memory` — manual range contains, needless borrow (4 errors)
11. `typed-patch` — collapsible match (1 error)

## Stale / Generated / Shadow Artifacts
- Root-level Codex archive JSON/manifest/report files (untracked) — safe to delete
- `docs/source-packages/archive/20260528T032033Z/` — generated artifact
- `turbo-semantic/` — duplicate crate of semantic-memory, not referenced, not in workspace
- `_salvage_from_libraries2/` — already quarantined by directory naming

## Submodules / Git Hygiene
- `.gitmodules` missing but submodule entries remain in git index for `fib-quant` and possibly others.
- This causes `git submodule status` to fatal. Needs de-submodule cleanup.
