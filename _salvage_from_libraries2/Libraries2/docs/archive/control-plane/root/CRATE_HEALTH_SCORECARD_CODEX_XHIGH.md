# Crate Health Scorecard — Codex XHigh

This scorecard preserves the **baseline static maturity read** that produced the XHigh control
plane.

It should be read as the starting audit, not as a live progress tracker after later repo-surface
and packaging passes. For current issue status, use `MASTER_ISSUE_MATRIX_CODEX_XHIGH.md`.

It is not a build badge.  
It is a **static maturity read** combining:
- source depth,
- test density,
- doc signal,
- compatibility debt,
- packaging gaps,
- and role in the wider stack.

## Legend

- **docs**: strength of crate-root rustdoc signal in the snapshot
- **tests**: rough signal from discovered test annotations
- **compat**: how much migration-era public surface is still visible
- **packaging_gaps**: missing README / repository / placeholder metadata

## Scorecard

| crate | group | rs_lines | tests | docs | compat | packaging_gaps | maturity_band | note |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| forge-engine | core_authority_lane | 11636 | 153 | weak | medium | missing_readme,missing_repository | strong_core_doc_gap | Large verification engine with meaningful tests; crate-root docs and packaging lag badly. |
| forge-memory-bridge | core_authority_lane | 1929 | 34 | medium | high | missing_readme,missing_repository | strong_core_compat_heavy | Boundary discipline is good; legacy surface is still too visible. |
| knowledge-runtime | core_authority_lane | 8580 | 125 | strong | medium | missing_readme | strong_core_thin_packaging | Real orchestration/read path with solid docs in lib.rs; missing repo front door and causal productization. |
| semantic-memory | core_authority_lane | 25628 | 289 | strong | high | missing_readme | strong_core_compat_heavy | Deep store/query substrate with strong test density; narrative and compat surface need cleanup. |
| semantic-memory-forge | core_authority_lane | 1045 | 13 | strong | low | missing_readme,missing_repository | strong_core_thin_packaging | Export schema/bundle layer is coherent; needs external docs and release framing. |
| stack-ids | core_authority_lane | 1223 | 40 | medium | high | missing_readme,missing_repository | strong_core_thin_packaging | Authority primitives look solid; packaging/docs absent in snapshot. |
| agent-graph | execution_and_workflow | 10547 | 135 | strong | high | missing_readme,missing_repository | substantial_compat_heavy | Large graph runtime with real tests; public event surface still straddles old and new eras. |
| ai-batch-queue | execution_and_workflow | 2766 | 56 | medium | low | missing_readme,placeholder_repository | real_satellite_underpackaged | Nontrivial implementation; packaging still looks prototype-grade. |
| job-queue | execution_and_workflow | 3511 | 43 | medium | high | missing_readme,missing_repository | substantial_compat_heavy | Real queue crate; legacy event fields and absent docs/repo metadata keep it looking mid-migration. |
| llm-output-parser | execution_and_workflow | 3552 | 144 | strong | low | missing_readme,missing_repository | strong_hidden_packaging | Well-tested parser hidden in a dot-prefixed path with missing README/repo. |
| llm-pipeline | execution_and_workflow | 8544 | 212 | strong | high | missing_readme | substantial_compat_heavy | Deep crate with two API eras and hidden parser dependency. |
| tauri-queue | execution_and_workflow | 1305 | 29 | medium | medium | missing_readme,placeholder_repository | wrapper_underpackaged | Useful adapter, but mostly a re-export shell with placeholder metadata. |
| cea-core | primitives | 2210 | 7 | medium | low | missing_readme,missing_repository | promising_primitive_needs_governance | Nontrivial primitive, but public contract and doc surface are thin. |
| cea-sqlite | primitives | 182 | 0 | weak | low | missing_readme,missing_repository | thin_primitive_needs_docs_tests | Tiny adapter crate with no docs/tests in snapshot. |
| cea-store | primitives | 177 | 0 | weak | low | missing_readme,missing_repository | thin_primitive_needs_docs_tests | Core abstraction but zero external docs/tests in snapshot. |
| check-runner | primitives | 537 | 0 | weak | low | missing_readme,missing_repository | promising_primitive_needs_governance | Useful execution primitive with no crate docs/tests. |
| effect-signature | primitives | 37 | 0 | weak | low | missing_readme,missing_repository | thin_primitive_needs_docs_tests | Very small primitive with no docs/tests. |
| forge-policy | primitives | 366 | 2 | weak | low | missing_readme,missing_repository | thin_primitive_needs_docs_tests | Policy primitive needs docs and broader proof. |
| mindstate-core | primitives | 115 | 0 | weak | low | missing_readme,missing_repository | thin_primitive_needs_docs_tests | Small core crate without docs/tests. |
| sandbox-workspace | primitives | 177 | 0 | weak | low | missing_readme,missing_repository | thin_primitive_needs_docs_tests | Likely important safety primitive, but not externally explained. |
| stabilizer-core | primitives | 277 | 0 | weak | low | missing_readme,missing_repository | thin_primitive_needs_docs_tests | Interesting primitive but no crate-level docs/tests. |
| typed-patch | primitives | 771 | 1 | weak | low | missing_readme,missing_repository | promising_primitive_needs_governance | Potentially central primitive that deserves docs/tests/front-door treatment. |
| comfyui-rs | satellite_clients | 1570 | 22 | strong | low | missing_readme,missing_repository | real_satellite_underpackaged | Looks usable; manifest/docs/repo surface lag. |
| ollama-vision | satellite_clients | 814 | 6 | strong | low | missing_readme,missing_repository | real_satellite_underpackaged | Crate docs are decent, but packaging/public release story is incomplete. |

## Strongest core areas

- `stack-ids`
- `semantic-memory-forge`
- `forge-memory-bridge`
- `semantic-memory`
- `knowledge-runtime`
- `forge-engine`

These look like a **real architecture**, not speculative modules.

## Most obvious governance / packaging gaps

- root repo surface missing from the snapshot
- no README / AGENTS inside the snapshot
- manifest metadata incomplete across most crates
- hidden or inconsistent directory naming
- placeholder repositories still present in satellite crates

## Most obvious compatibility hotspots

- `forge-memory-bridge`
- `semantic-memory`
- `job-queue`
- `agent-graph`
- `llm-pipeline`
- `tauri-queue`

These are the places where migration-era public surface is still loud enough to confuse the story.
