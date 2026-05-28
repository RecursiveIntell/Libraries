# File Audit Inventory — Current Full Snapshot

Generated from static inspection of `new3.zip` on 2026-03-08.

This inventory lists **every file** in the snapshot and records the audit depth used for this pass.

## Audit depth legend
- **direct_read**: file opened and read directly during the architecture audit
- **targeted_scan**: file scanned via targeted search/grep for invariants, compat seams, lag markers, or test coverage
- **inventory_only**: file inventoried but not deeply inspected in this pass; no material architecture issue isolated from static inspection

## Totals
- Total files inventoried: **224**
- Direct-read files: **29**
- Targeted-scan files: **174**
- Inventory-only files: **21**


## (root)

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| CLAUDE.md | root_doc | direct_read | DOC-001 | Read directly during architecture audit. |
| README.md | root_doc | direct_read | DOC-001 | Read directly during architecture audit. |

## AI-Batch-Queue

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| AI-Batch-Queue/Cargo.toml | crate_manifest | targeted_scan |  | Manifest inventoried; not a primary source of architecture issues. |
| AI-Batch-Queue/README.md | crate_doc | targeted_scan |  | Scanned for doctrinal consistency with current code. |
| AI-Batch-Queue/src/eta.rs | source | targeted_scan | ABQ-001 | Scanned for compat debt, lag markers, and issue cross-references. |
| AI-Batch-Queue/src/executor.rs | source | direct_read | ABQ-001 | Read directly during architecture audit. |
| AI-Batch-Queue/src/lib.rs | source | direct_read | ABQ-001 | Read directly during architecture audit. |
| AI-Batch-Queue/src/queue.rs | source | direct_read | ABQ-001 | Read directly during architecture audit. |
| AI-Batch-Queue/src/types.rs | source | direct_read | ABQ-001 | Read directly during architecture audit. |
| AI-Batch-Queue/tests/integration_tests.rs | test | targeted_scan | ABQ-001 | Scanned for invariants, compat seams, and proof coverage. |

## ComfyUI-RS

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| ComfyUI-RS/Cargo.lock | support | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| ComfyUI-RS/Cargo.toml | crate_manifest | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| ComfyUI-RS/README.md | crate_doc | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| ComfyUI-RS/src/client.rs | source | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| ComfyUI-RS/src/error.rs | source | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| ComfyUI-RS/src/lib.rs | source | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| ComfyUI-RS/src/types.rs | source | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| ComfyUI-RS/src/workflow.rs | source | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |

## LLM-Pipeline

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| LLM-Pipeline/Cargo.toml | crate_manifest | targeted_scan |  | Manifest inventoried; not a primary source of architecture issues. |
| LLM-Pipeline/README.md | crate_doc | targeted_scan |  | Scanned for doctrinal consistency with current code. |
| LLM-Pipeline/examples/basic_pipeline.rs | support | targeted_scan |  | Targeted scan. |
| LLM-Pipeline/examples/context_injection.rs | support | targeted_scan |  | Targeted scan. |
| LLM-Pipeline/examples/mock_example.rs | support | targeted_scan |  | Targeted scan. |
| LLM-Pipeline/examples/payload_chain.rs | support | targeted_scan |  | Targeted scan. |
| LLM-Pipeline/examples/streaming_pipeline.rs | support | targeted_scan |  | Targeted scan. |
| LLM-Pipeline/examples/thinking_mode.rs | support | targeted_scan |  | Targeted scan. |
| LLM-Pipeline/src/backend/backoff.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/backend/mock.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/backend/mod.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/backend/ollama.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/backend/openai.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/backend/recording.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/backend/sse.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/chain.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/client.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/diagnostics.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/error.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/events.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/exec_ctx.rs | source | direct_read | LLM-001 | Read directly during architecture audit. |
| LLM-Pipeline/src/lib.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/limits.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/llm_call.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/output_parser.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/output_strategy.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/parsing.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/payload.rs | source | targeted_scan | LLM-001 | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/pipeline.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/prompt.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/retry.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/retry_policy.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/stage.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/streaming.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| LLM-Pipeline/src/trace.rs | source | direct_read | LLM-001 | Read directly during architecture audit. |
| LLM-Pipeline/src/types.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |

## Ollama-Vision-RS

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| Ollama-Vision-RS/Cargo.lock | support | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| Ollama-Vision-RS/Cargo.toml | crate_manifest | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| Ollama-Vision-RS/README.md | crate_doc | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| Ollama-Vision-RS/src/captioner.rs | source | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| Ollama-Vision-RS/src/lib.rs | source | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| Ollama-Vision-RS/src/parser.rs | source | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| Ollama-Vision-RS/src/tagger.rs | source | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| Ollama-Vision-RS/src/types.rs | source | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| Ollama-Vision-RS/tests/truncation_tests.rs | test | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |

## Tauri-Queue

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| Tauri-Queue/Cargo.toml | crate_manifest | targeted_scan |  | Manifest inventoried; not a primary source of architecture issues. |
| Tauri-Queue/README.md | crate_doc | targeted_scan |  | Scanned for doctrinal consistency with current code. |
| Tauri-Queue/src/lib.rs | source | direct_read | TQ-001 | Read directly during architecture audit. |
| Tauri-Queue/tests/integration_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| Tauri-Queue/tests/test_helpers.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |

## Tauri-React-Hooks

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| Tauri-React-Hooks/README.md | crate_doc | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |

## agent-graph

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| agent-graph/Cargo.lock | support | targeted_scan |  | Targeted scan. |
| agent-graph/Cargo.toml | crate_manifest | targeted_scan |  | Manifest inventoried; not a primary source of architecture issues. |
| agent-graph/README.md | crate_doc | targeted_scan |  | Scanned for doctrinal consistency with current code. |
| agent-graph/src/checkpoint.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/checkpoint_store.rs | source | direct_read |  | Read directly during architecture audit. |
| agent-graph/src/checkpointer.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/command.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/config.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/edge.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/error.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/event_sink.rs | source | direct_read | AG-001, AG-002 | Read directly during architecture audit. |
| agent-graph/src/executor.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/graph.rs | source | direct_read | AG-001, AG-002 | Read directly during architecture audit. |
| agent-graph/src/interrupt.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/join.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/lib.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/node.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/outcome.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/payload.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/prelude.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/reducer.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/retry.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/router.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/state.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/src/stream.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| agent-graph/tests/checkpointer_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/execution_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/integration_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/interrupt_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/parallel_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/reducer_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/retry_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/routing_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/runtime_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/state_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/step5_verification.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/streaming_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| agent-graph/tests/subgraph_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |

## forge-memory-bridge

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| forge-memory-bridge/Cargo.lock | support | targeted_scan |  | Targeted scan. |
| forge-memory-bridge/Cargo.toml | crate_manifest | targeted_scan |  | Manifest inventoried; not a primary source of architecture issues. |
| forge-memory-bridge/src/batch.rs | source | direct_read | BRG-001 | Read directly during architecture audit. |
| forge-memory-bridge/src/envelope.rs | source | direct_read |  | Read directly during architecture audit. |
| forge-memory-bridge/src/error.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| forge-memory-bridge/src/legacy.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| forge-memory-bridge/src/lib.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| forge-memory-bridge/src/transform.rs | source | direct_read | BRG-002, BRG-003 | Read directly during architecture audit. |
| forge-memory-bridge/tests/forge_bridge_memory_proof.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |

## job-queue

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| job-queue/Cargo.lock | support | targeted_scan |  | Targeted scan. |
| job-queue/Cargo.toml | crate_manifest | targeted_scan |  | Manifest inventoried; not a primary source of architecture issues. |
| job-queue/README.md | crate_doc | targeted_scan |  | Scanned for doctrinal consistency with current code. |
| job-queue/src/config.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| job-queue/src/db.rs | source | direct_read | JQ-002 | Read directly during architecture audit. |
| job-queue/src/error.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| job-queue/src/events.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| job-queue/src/executor.rs | source | direct_read | JQ-002 | Read directly during architecture audit. |
| job-queue/src/lib.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| job-queue/src/queue.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| job-queue/src/types.rs | source | direct_read | JQ-001 | Read directly during architecture audit. |

## knowledge-runtime

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| knowledge-runtime/Cargo.lock | support | targeted_scan |  | Targeted scan. |
| knowledge-runtime/Cargo.toml | crate_manifest | targeted_scan |  | Manifest inventoried; not a primary source of architecture issues. |
| knowledge-runtime/src/adapters/mod.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/adapters/semantic_memory.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/config.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/entity/code_ids.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/entity/mod.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/entity/registry.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/error.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/evidence/mod.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/evidence/support.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/ids.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/lib.rs | source | direct_read | KR-001, KR-002, KR-003, KR-004 | Read directly during architecture audit. |
| knowledge-runtime/src/obs/mod.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/obs/trace.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/projection/lifecycle.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/projection/mod.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/query/classify.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/query/merge.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/query/mod.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/query/route.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/runtime.rs | source | direct_read | KR-001, KR-002, KR-003, KR-004 | Read directly during architecture audit. |
| knowledge-runtime/src/temporal/claims.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/src/temporal/mod.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| knowledge-runtime/tests/cross_crate_proof.rs | test | targeted_scan | E2E-001, KR-005 | Scanned for invariants, compat seams, and proof coverage. |
| knowledge-runtime/tests/invariant_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| knowledge-runtime/tests/ugly_case_tests.rs | test | targeted_scan | KR-005 | Scanned for invariants, compat seams, and proof coverage. |

## living-memory

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| living-memory/Cargo.toml | crate_manifest | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |

## llm-pipeline

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| llm-pipeline/.claude/settings.local.json | support | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |
| llm-pipeline/Cargo.lock | support | inventory_only |  | Inventoried as support/non-core crate for this architecture-closure pass. |

## semantic-memory

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| semantic-memory/.gitignore | support | targeted_scan |  | Targeted scan. |
| semantic-memory/AGENTS.md | crate_doc | targeted_scan |  | Scanned for doctrinal consistency with current code. |
| semantic-memory/Cargo.lock | support | targeted_scan |  | Targeted scan. |
| semantic-memory/Cargo.toml | crate_manifest | targeted_scan |  | Manifest inventoried; not a primary source of architecture issues. |
| semantic-memory/LICENSE | support | targeted_scan |  | Targeted scan. |
| semantic-memory/README.md | crate_doc | targeted_scan |  | Scanned for doctrinal consistency with current code. |
| semantic-memory/examples/basic_search.rs | support | targeted_scan |  | Targeted scan. |
| semantic-memory/examples/conversation_memory.rs | support | targeted_scan |  | Targeted scan. |
| semantic-memory/src/chunker.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/config.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/conversation.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/db.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/documents.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/embedder.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/episodes.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/error.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/graph.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/hnsw.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/knowledge.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/lib.rs | source | direct_read | SM-001, SM-002, SM-003 | Read directly during architecture audit. |
| semantic-memory/src/pool.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/projection_import.rs | source | direct_read | SM-005 | Read directly during architecture audit. |
| semantic-memory/src/projection_storage.rs | source | direct_read | SM-004 | Read directly during architecture audit. |
| semantic-memory/src/quantize.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/search.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/storage.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/tokenizer.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/src/types.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| semantic-memory/tests/brute_force_parity.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/chunker_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/compaction.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/concurrent_access.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/conversation_search_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/conversation_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/db_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/episode_identity.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/hardening_semantics.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/hardening_v5.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/hnsw_hotswap.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/hnsw_integration.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/hnsw_persistence.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/import_boundary_tests.rs | test | targeted_scan | SM-006 | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/import_ugly_cases.rs | test | targeted_scan | SM-006 | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/integration_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/knowledge_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/migration_v5.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/projection_v11_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/quantization.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/quantization_pipeline.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/search_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/step3_verification.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/step4_verification.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/storage_lifecycle.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/tokenizer_tests.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/trace_id_write_seam.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |
| semantic-memory/tests/vector_only_hnsw.rs | test | targeted_scan |  | Scanned for invariants, compat seams, and proof coverage. |

## semantic-memory-forge

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| semantic-memory-forge/Cargo.toml | crate_manifest | targeted_scan |  | Manifest inventoried; not a primary source of architecture issues. |
| semantic-memory-forge/src/bundle.rs | source | direct_read | FMF-001 | Read directly during architecture audit. |
| semantic-memory-forge/src/envelope.rs | source | direct_read | FMF-001 | Read directly during architecture audit. |
| semantic-memory-forge/src/estimator.rs | source | direct_read | FMF-001 | Read directly during architecture audit. |
| semantic-memory-forge/src/lib.rs | source | targeted_scan | FMF-001 | Scanned for compat debt, lag markers, and issue cross-references. |

## stack-ids

| File | Role | Audit depth | Issue refs | Notes |
| --- | --- | --- | --- | --- |
| stack-ids/Cargo.toml | crate_manifest | targeted_scan |  | Manifest inventoried; not a primary source of architecture issues. |
| stack-ids/src/digest.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| stack-ids/src/ids.rs | source | direct_read |  | Read directly during architecture audit. |
| stack-ids/src/lib.rs | source | direct_read |  | Read directly during architecture audit. |
| stack-ids/src/scope.rs | source | targeted_scan |  | Scanned for compat debt, lag markers, and issue cross-references. |
| stack-ids/src/trace.rs | source | direct_read | SID-001 | Read directly during architecture audit. |