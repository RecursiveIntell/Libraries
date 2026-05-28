# File Audit Inventory — Codex V6

**Snapshot:** `now1.zip`
**Method:** full tree inventory; deep static inspection of the core authority lane and proof tests; light scan of surrounding repo structure; no successful build/test run in this environment.

## Scope note

This V6 handoff is intentionally centered on the **core authority lane**:

- `stack-ids`
- `semantic-memory-forge`
- `forge-memory-bridge`
- `semantic-memory`
- `knowledge-runtime`
- `forge-engine` at `living-memory/living-memory`

Adjacent control-flow crates still matter, but they were **not** re-audited deeply for this pass and should be treated as a secondary lane after the core projection substrate is closed.

## Crate summary

| Path | Approx. file count | Audit depth | Notes |
|---|---:|---|---|
| root control docs | dozens | deep on active control docs + canonical spec + `LATEST7.md`; inventory-only on older historical docs | the repo has many historical matrices/prompts; treat them as potentially stale |
| `stack-ids` | 7 | deep | trace/scope/ID law |
| `semantic-memory-forge` | 5 | deep | export envelope ownership and claim lineage gap |
| `forge-memory-bridge` | 8 | deep | import batch contract, `recorded_at`, legacy helpers |
| `semantic-memory` | 119 | deep on import/storage/types/tests; light elsewhere | projection storage is real, query surface still missing |
| `knowledge-runtime` | 29 | deep | retrieval still hybrid-centric, temporal/scope gaps explicit |
| `living-memory/living-memory` | subset of ~73 under `living-memory` | deep on export seam + evidence bundle types | export path exists but is semantically thin |
| `agent-graph`, `job-queue`, `AI-Batch-Queue`, `LLM-Pipeline`, `Tauri-Queue` | contextual only | inventory / light context only | secondary lane after core closure |

## Deep-static hotspots reviewed

### Root control plane
- `AGENTS.md` — stale and now misleading in places
- `MASTER_ISSUE_MATRIX_CODEX_V5.md` — useful history, but no longer the right prioritization
- `LATEST7.md` — important because it currently encodes the wrong `recorded_at` story
- `CANONICAL_STACK_SPEC_V5.md` — target-state law

### stack-ids
- `stack-ids/src/trace.rs` — canonical `TraceCtx` shape and durable-trace question

### semantic-memory-forge
- `semantic-memory-forge/src/envelope.rs` — export envelope and `ExportClaim` lineage fields

### forge-memory-bridge
- `forge-memory-bridge/src/batch.rs` — per-record `recorded_at` contract
- `forge-memory-bridge/src/transform.rs` — transformation logic, no-fake-lineage rule
- `forge-memory-bridge/src/legacy.rs` — lingering legacy bridge helpers
- `forge-memory-bridge/tests/forge_bridge_memory_proof.rs` — proof test currently asserting the wrong `recorded_at` semantics

### semantic-memory
- `semantic-memory/src/lib.rs` — canonical import path, compat path, import-log query, missing public projection query surface
- `semantic-memory/src/projection_storage.rs` — projection tables, dead-code query helpers, derivation-edge coverage
- `semantic-memory/src/types.rs` — `SearchResult` surface still lacks temporal projection fields
- `semantic-memory/tests/projection_v11_tests.rs` — current import proof surface
- `semantic-memory/tests/import_boundary_tests.rs` — legacy compat proof surface

### knowledge-runtime
- `knowledge-runtime/src/adapters/semantic_memory.rs` — still search-only adapter
- `knowledge-runtime/src/runtime.rs` — explicit temporal/scope degradation behavior
- `knowledge-runtime/src/config.rs` — overly absolute “will not be” language
- `knowledge-runtime/src/entity/registry.rs` — exact-only resolution behavior
- `knowledge-runtime/tests/cross_crate_proof.rs` — imported-data proof currently side-loads a fact

### forge-engine (`living-memory/living-memory`)
- `living-memory/living-memory/src/export.rs` — canonical export lane exists but is semantically thin
- `living-memory/living-memory/src/lab/evidence.rs` — rich bundle fields available for better export rendering

## Important inventory caveats

- No build/test run was completed here, so this handoff is grounded in static inspection only.
- The root workspace already exists and points at the core authority lane; that earlier repo-level blocker is now closed.
- Historical docs are plentiful and can easily mislead Codex if read as live authority.
- `living-memory` contains both crate code and a large documentation surface; only the export seam and evidence-bundle contract were reviewed deeply for this pass.
- `semantic-memory` still contains substantial compatibility material; not all of it is wrong, but much of it is louder than it should be.
