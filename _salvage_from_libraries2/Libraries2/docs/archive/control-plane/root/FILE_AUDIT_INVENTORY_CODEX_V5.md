# File Audit Inventory — Codex V5

**Full file list:** see `FILE_AUDIT_INVENTORY_CODEX_V5.csv`  
**Snapshot:** `new4.zip`  
**Method:** deep static inspection for core architecture hotspots; light static inspection for supporting source/tests; inventory-only for vendor/build noise.

## Crate summary

| Crate | File count | Deep-static | Light-static | Inventory-only |
|---|---:|---:|---:|---:|
| AI-Batch-Queue | 7 | 3 | 3 | 1 |
| ComfyUI-RS | 7 | 0 | 5 | 2 |
| LLM-Pipeline | 35 | 2 | 26 | 7 |
| Ollama-Vision-RS | 8 | 0 | 6 | 2 |
| Tauri-Queue | 4 | 1 | 2 | 1 |
| Tauri-React-Hooks | 978 | 0 | 8 | 970 |
| agent-graph | 37 | 6 | 29 | 2 |
| forge-memory-bridge | 9 | 4 | 3 | 2 |
| job-queue | 10 | 4 | 4 | 2 |
| knowledge-runtime | 28 | 6 | 20 | 2 |
| living-memory | 1 | 0 | 0 | 1 |
| semantic-memory | 52 | 7 | 41 | 4 |
| semantic-memory-forge | 6 | 3 | 1 | 2 |
| stack-ids | 7 | 0 | 5 | 2 |

## What “deep-static” means

The following files got the closest review because they govern architecture closure, provenance, runtime semantics, or retry/trace law:


### AI-Batch-Queue

- `AI-Batch-Queue/src/lib.rs` — issue refs: ABQ-001
- `AI-Batch-Queue/src/queue.rs` — issue refs: ABQ-001
- `AI-Batch-Queue/src/types.rs` — issue refs: ABQ-001

### LLM-Pipeline

- `LLM-Pipeline/src/exec_ctx.rs` — issue refs: LLP-002
- `LLM-Pipeline/src/trace.rs` — issue refs: LLP-001

### Tauri-Queue

- `Tauri-Queue/src/lib.rs` — issue refs: TQ-001,TQ-002

### agent-graph

- `agent-graph/src/checkpoint_store.rs`
- `agent-graph/src/config.rs` — issue refs: AG-004
- `agent-graph/src/event_sink.rs` — issue refs: AG-003
- `agent-graph/src/graph.rs` — issue refs: AG-001,AG-002
- `agent-graph/tests/retry_tests.rs`
- `agent-graph/tests/step5_verification.rs`

### forge-memory-bridge

- `forge-memory-bridge/src/batch.rs` — issue refs: BRG-001
- `forge-memory-bridge/src/legacy.rs` — issue refs: BRG-002
- `forge-memory-bridge/src/transform.rs`
- `forge-memory-bridge/tests/forge_bridge_memory_proof.rs` — issue refs: BRG-003

### job-queue

- `job-queue/src/db.rs` — issue refs: JQ-002
- `job-queue/src/events.rs` — issue refs: JQ-003
- `job-queue/src/executor.rs` — issue refs: JQ-001
- `job-queue/src/lib.rs` — issue refs: JQ-003

### knowledge-runtime

- `knowledge-runtime/src/config.rs` — issue refs: KR-004,KR-005
- `knowledge-runtime/src/lib.rs` — issue refs: KR-001,KR-002,KR-003
- `knowledge-runtime/src/projection/rebuild.rs` — issue refs: KR-006
- `knowledge-runtime/src/runtime.rs`
- `knowledge-runtime/tests/cross_crate_proof.rs` — issue refs: KR-007
- `knowledge-runtime/tests/ugly_case_tests.rs`

### semantic-memory

- `semantic-memory/src/lib.rs` — issue refs: SM-001,SM-002,SM-004,SM-005,SM-006
- `semantic-memory/src/projection_import.rs` — issue refs: SM-006
- `semantic-memory/src/projection_storage.rs` — issue refs: SM-003
- `semantic-memory/tests/hardening_semantics.rs`
- `semantic-memory/tests/hardening_v5.rs`
- `semantic-memory/tests/import_boundary_tests.rs` — issue refs: SM-007
- `semantic-memory/tests/import_ugly_cases.rs` — issue refs: SM-007

### semantic-memory-forge

- `semantic-memory-forge/src/bundle.rs` — issue refs: SMF-001
- `semantic-memory-forge/src/envelope.rs` — issue refs: SMF-001,SMF-002
- `semantic-memory-forge/src/estimator.rs` — issue refs: SMF-001

## Important inventory caveats

- `Tauri-React-Hooks/node_modules` and `dist/` were **not** semantically audited file-by-file; they were treated as vendor/build clutter and marked inventory-only.
- `living-memory` is effectively a stub in this snapshot.
- Absence of a top-level workspace manifest in the archive is itself tracked as a hygiene/integration issue.
