# File Audit Inventory — Current State
This inventory lists every non-lock file from the snapshot and marks whether it has a direct issue row, is a support file, is outside the current closure scope, or had no standalone issue from static inspection.

## AI-Batch-Queue
- `AI-Batch-Queue/Cargo.toml` — **Reviewed context file**. No new material issue from static inspection beyond crate-level issues.
- `AI-Batch-Queue/src/eta.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `AI-Batch-Queue/src/executor.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `AI-Batch-Queue/src/lib.rs` — **Action required**. Tracked in matrix: I037
- `AI-Batch-Queue/src/queue.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `AI-Batch-Queue/src/types.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `AI-Batch-Queue/tests/integration_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.

## ComfyUI-RS
- `ComfyUI-RS/Cargo.toml` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `ComfyUI-RS/src/client.rs` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `ComfyUI-RS/src/error.rs` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `ComfyUI-RS/src/lib.rs` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `ComfyUI-RS/src/types.rs` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `ComfyUI-RS/src/workflow.rs` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.

## LLM-Pipeline
- `LLM-Pipeline/Cargo.toml` — **Reviewed context file**. No new material issue from static inspection beyond crate-level issues.
- `LLM-Pipeline/examples/basic_pipeline.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `LLM-Pipeline/examples/context_injection.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `LLM-Pipeline/examples/mock_example.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `LLM-Pipeline/examples/payload_chain.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `LLM-Pipeline/examples/streaming_pipeline.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `LLM-Pipeline/examples/thinking_mode.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `LLM-Pipeline/src/backend/backoff.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/backend/mock.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/backend/mod.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/backend/ollama.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/backend/openai.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/backend/recording.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/backend/sse.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/chain.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/client.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/diagnostics.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/error.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/events.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/exec_ctx.rs` — **Action required**. Tracked in matrix: I041
- `LLM-Pipeline/src/lib.rs` — **Action required**. Tracked in matrix: I042
- `LLM-Pipeline/src/limits.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/llm_call.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/output_parser.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/output_strategy.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/parsing.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/payload.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/pipeline.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/prompt.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/retry.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/retry_policy.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/stage.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/streaming.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `LLM-Pipeline/src/trace.rs` — **Action required**. Tracked in matrix: I040
- `LLM-Pipeline/src/types.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.

## Ollama-Vision-RS
- `Ollama-Vision-RS/Cargo.toml` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Ollama-Vision-RS/src/captioner.rs` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Ollama-Vision-RS/src/lib.rs` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Ollama-Vision-RS/src/parser.rs` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Ollama-Vision-RS/src/tagger.rs` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Ollama-Vision-RS/src/types.rs` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Ollama-Vision-RS/tests/truncation_tests.rs` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.

## Tauri-Queue
- `Tauri-Queue/Cargo.toml` — **Reviewed context file**. No new material issue from static inspection beyond crate-level issues.
- `Tauri-Queue/src/lib.rs` — **Action required**. Tracked in matrix: I038
- `Tauri-Queue/tests/integration_tests.rs` — **Action required**. Tracked in matrix: I039
- `Tauri-Queue/tests/test_helpers.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.

## Tauri-React-Hooks
- `Tauri-React-Hooks/package.json` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Tauri-React-Hooks/src/index.ts` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Tauri-React-Hooks/src/types.ts` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Tauri-React-Hooks/src/useBufferedStream.ts` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Tauri-React-Hooks/src/useTauriConfig.ts` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Tauri-React-Hooks/src/useTauriEvent.ts` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Tauri-React-Hooks/src/useTauriEvents.ts` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Tauri-React-Hooks/src/useTauriMutation.ts` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.
- `Tauri-React-Hooks/src/useTauriQuery.ts` — **Out of current closure scope**. Reviewed only at a high level; not a current canonical-stack closure target in this pass.

## agent-graph
- `agent-graph/Cargo.toml` — **Reviewed context file**. No new material issue from static inspection beyond crate-level issues.
- `agent-graph/src/checkpoint.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/checkpoint_store.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/checkpointer.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/command.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/config.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/edge.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/error.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/event_sink.rs` — **Action required**. Tracked in matrix: I029
- `agent-graph/src/executor.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/graph.rs` — **Action required**. Tracked in matrix: I030, I031, I032
- `agent-graph/src/interrupt.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/join.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/lib.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/node.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/outcome.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/payload.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/prelude.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/reducer.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/retry.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/router.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/state.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/src/stream.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `agent-graph/tests/checkpointer_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/execution_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/integration_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/interrupt_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/parallel_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/reducer_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/retry_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/routing_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/runtime_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/state_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/step5_verification.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/streaming_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `agent-graph/tests/subgraph_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.

## forge-memory-bridge
- `forge-memory-bridge/Cargo.toml` — **Reviewed context file**. No new material issue from static inspection beyond crate-level issues.
- `forge-memory-bridge/src/batch.rs` — **Action required**. Tracked in matrix: I005, I006
- `forge-memory-bridge/src/envelope.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `forge-memory-bridge/src/error.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `forge-memory-bridge/src/legacy.rs` — **Action required**. Tracked in matrix: I007
- `forge-memory-bridge/src/lib.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `forge-memory-bridge/src/transform.rs` — **Action required**. Tracked in matrix: I004, I006

## job-queue
- `job-queue/Cargo.toml` — **Reviewed context file**. No new material issue from static inspection beyond crate-level issues.
- `job-queue/src/config.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `job-queue/src/db.rs` — **Action required**. Tracked in matrix: I034
- `job-queue/src/error.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `job-queue/src/events.rs` — **Action required**. Tracked in matrix: I033
- `job-queue/src/executor.rs` — **Action required**. Tracked in matrix: I035
- `job-queue/src/lib.rs` — **Action required**. Tracked in matrix: I033, I036
- `job-queue/src/queue.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `job-queue/src/types.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.

## knowledge-runtime
- `knowledge-runtime/Cargo.toml` — **Reviewed context file**. No new material issue from static inspection beyond crate-level issues.
- `knowledge-runtime/src/adapters/mod.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/adapters/semantic_memory.rs` — **Action required**. Tracked in matrix: I021
- `knowledge-runtime/src/config.rs` — **Action required**. Tracked in matrix: I023
- `knowledge-runtime/src/entity/code_ids.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/entity/mod.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/entity/registry.rs` — **Action required**. Tracked in matrix: I026
- `knowledge-runtime/src/error.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/evidence/mod.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/evidence/support.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/ids.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/lib.rs` — **Action required**. Tracked in matrix: I025, I026
- `knowledge-runtime/src/obs/mod.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/obs/trace.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/projection/lifecycle.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/projection/mod.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/query/classify.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/query/merge.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/query/mod.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/query/route.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/runtime.rs` — **Action required**. Tracked in matrix: I020, I022, I023, I024, I025, I027
- `knowledge-runtime/src/temporal/claims.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/src/temporal/mod.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `knowledge-runtime/tests/cross_crate_proof.rs` — **Action required**. Tracked in matrix: I017, I028
- `knowledge-runtime/tests/invariant_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `knowledge-runtime/tests/ugly_case_tests.rs` — **Action required**. Tracked in matrix: I028

## semantic-memory
- `semantic-memory/Cargo.toml` — **Reviewed context file**. No new material issue from static inspection beyond crate-level issues.
- `semantic-memory/examples/basic_search.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/examples/conversation_memory.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/src/chunker.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/config.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/conversation.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/db.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/documents.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/embedder.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/episodes.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/error.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/graph.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/hnsw.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/knowledge.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/lib.rs` — **Action required**. Tracked in matrix: I008, I009, I011, I012, I017
- `semantic-memory/src/pool.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/projection_import.rs` — **Action required**. Tracked in matrix: I016
- `semantic-memory/src/projection_storage.rs` — **Action required**. Tracked in matrix: I013, I014, I015, I019
- `semantic-memory/src/quantize.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/search.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/storage.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/tokenizer.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/src/types.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `semantic-memory/tests/brute_force_parity.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/chunker_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/compaction.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/concurrent_access.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/conversation_search_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/conversation_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/db_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/episode_identity.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/hardening_semantics.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/hardening_v5.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/hnsw_hotswap.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/hnsw_integration.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/hnsw_persistence.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/import_boundary_tests.rs` — **Action required**. Tracked in matrix: I018
- `semantic-memory/tests/import_ugly_cases.rs` — **Action required**. Tracked in matrix: I010
- `semantic-memory/tests/integration_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/knowledge_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/migration_v5.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/projection_v11_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/quantization.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/quantization_pipeline.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/search_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/step3_verification.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/step4_verification.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/storage_lifecycle.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/tokenizer_tests.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/trace_id_write_seam.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.
- `semantic-memory/tests/vector_only_hnsw.rs` — **Support file**. Keep unless the parent issue cluster requires test/example updates.

## stack-ids
- `stack-ids/Cargo.toml` — **Reviewed context file**. No new material issue from static inspection beyond crate-level issues.
- `stack-ids/src/digest.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `stack-ids/src/ids.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `stack-ids/src/lib.rs` — **Reviewed, no new material issue**. No standalone issue captured; file is covered by parent crate-level work or looked healthy in static inspection.
- `stack-ids/src/scope.rs` — **Action required**. Tracked in matrix: I003
- `stack-ids/src/trace.rs` — **Action required**. Tracked in matrix: I002

