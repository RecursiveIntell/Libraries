# File Audit Inventory — Architecture Finish V3

This inventory covers the unpacked `new2.zip` snapshot.

## Review depth legend
- **direct review** — file was opened and inspected directly for architecture-relevant behavior
- **pattern scan / inventory review** — file was included in grep/search passes and repo inventory, but not line-read end-to-end
- **inventory only** — file was inventoried for scope, but not architecture-audited directly in this pass

## Counts
- **AI-Batch-Queue**: 7 files (pattern scan / inventory review: 4, direct review: 3)
- **ComfyUI-RS**: 6 files (inventory only: 6)
- **LLM-Pipeline**: 35 files (pattern scan / inventory review: 33, direct review: 2)
- **Ollama-Vision-RS**: 7 files (inventory only: 7)
- **Tauri-Queue**: 4 files (pattern scan / inventory review: 3, direct review: 1)
- **agent-graph**: 36 files (pattern scan / inventory review: 33, direct review: 3)
- **forge-memory-bridge**: 7 files (pattern scan / inventory review: 4, direct review: 3)
- **job-queue**: 9 files (pattern scan / inventory review: 5, direct review: 4)
- **knowledge-runtime**: 26 files (pattern scan / inventory review: 23, direct review: 3)
- **living-memory**: 1 files (inventory only: 1)
- **semantic-memory**: 51 files (pattern scan / inventory review: 46, direct review: 5)
- **semantic-memory-forge**: 5 files (pattern scan / inventory review: 2, direct review: 3)
- **stack-ids**: 6 files (pattern scan / inventory review: 3, direct review: 3)
- **workspace**: 2 files (direct review: 2)

## AI-Batch-Queue

|File|Kind|Review depth|
|---|---|---|
|`AI-Batch-Queue/Cargo.toml`|source|pattern scan / inventory review|
|`AI-Batch-Queue/src/eta.rs`|source|pattern scan / inventory review|
|`AI-Batch-Queue/src/executor.rs`|source|pattern scan / inventory review|
|`AI-Batch-Queue/src/lib.rs`|source|direct review|
|`AI-Batch-Queue/src/queue.rs`|source|direct review|
|`AI-Batch-Queue/src/types.rs`|source|direct review|
|`AI-Batch-Queue/tests/integration_tests.rs`|test|pattern scan / inventory review|

## ComfyUI-RS

|File|Kind|Review depth|
|---|---|---|
|`ComfyUI-RS/Cargo.toml`|source|inventory only|
|`ComfyUI-RS/src/client.rs`|source|inventory only|
|`ComfyUI-RS/src/error.rs`|source|inventory only|
|`ComfyUI-RS/src/lib.rs`|source|inventory only|
|`ComfyUI-RS/src/types.rs`|source|inventory only|
|`ComfyUI-RS/src/workflow.rs`|source|inventory only|

## LLM-Pipeline

|File|Kind|Review depth|
|---|---|---|
|`LLM-Pipeline/Cargo.toml`|source|pattern scan / inventory review|
|`LLM-Pipeline/examples/basic_pipeline.rs`|example|pattern scan / inventory review|
|`LLM-Pipeline/examples/context_injection.rs`|example|pattern scan / inventory review|
|`LLM-Pipeline/examples/mock_example.rs`|example|pattern scan / inventory review|
|`LLM-Pipeline/examples/payload_chain.rs`|example|pattern scan / inventory review|
|`LLM-Pipeline/examples/streaming_pipeline.rs`|example|pattern scan / inventory review|
|`LLM-Pipeline/examples/thinking_mode.rs`|example|pattern scan / inventory review|
|`LLM-Pipeline/src/backend/backoff.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/backend/mock.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/backend/mod.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/backend/ollama.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/backend/openai.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/backend/recording.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/backend/sse.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/chain.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/client.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/diagnostics.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/error.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/events.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/exec_ctx.rs`|source|direct review|
|`LLM-Pipeline/src/lib.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/limits.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/llm_call.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/output_parser.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/output_strategy.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/parsing.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/payload.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/pipeline.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/prompt.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/retry.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/retry_policy.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/stage.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/streaming.rs`|source|pattern scan / inventory review|
|`LLM-Pipeline/src/trace.rs`|source|direct review|
|`LLM-Pipeline/src/types.rs`|source|pattern scan / inventory review|

## Ollama-Vision-RS

|File|Kind|Review depth|
|---|---|---|
|`Ollama-Vision-RS/Cargo.toml`|source|inventory only|
|`Ollama-Vision-RS/src/captioner.rs`|source|inventory only|
|`Ollama-Vision-RS/src/lib.rs`|source|inventory only|
|`Ollama-Vision-RS/src/parser.rs`|source|inventory only|
|`Ollama-Vision-RS/src/tagger.rs`|source|inventory only|
|`Ollama-Vision-RS/src/types.rs`|source|inventory only|
|`Ollama-Vision-RS/tests/truncation_tests.rs`|test|inventory only|

## Tauri-Queue

|File|Kind|Review depth|
|---|---|---|
|`Tauri-Queue/Cargo.toml`|source|pattern scan / inventory review|
|`Tauri-Queue/src/lib.rs`|source|direct review|
|`Tauri-Queue/tests/integration_tests.rs`|test|pattern scan / inventory review|
|`Tauri-Queue/tests/test_helpers.rs`|test|pattern scan / inventory review|

## agent-graph

|File|Kind|Review depth|
|---|---|---|
|`agent-graph/Cargo.toml`|source|pattern scan / inventory review|
|`agent-graph/src/checkpoint.rs`|source|pattern scan / inventory review|
|`agent-graph/src/checkpoint_store.rs`|source|direct review|
|`agent-graph/src/checkpointer.rs`|source|pattern scan / inventory review|
|`agent-graph/src/command.rs`|source|pattern scan / inventory review|
|`agent-graph/src/config.rs`|source|pattern scan / inventory review|
|`agent-graph/src/edge.rs`|source|pattern scan / inventory review|
|`agent-graph/src/error.rs`|source|pattern scan / inventory review|
|`agent-graph/src/event_sink.rs`|source|direct review|
|`agent-graph/src/executor.rs`|source|pattern scan / inventory review|
|`agent-graph/src/graph.rs`|source|direct review|
|`agent-graph/src/interrupt.rs`|source|pattern scan / inventory review|
|`agent-graph/src/join.rs`|source|pattern scan / inventory review|
|`agent-graph/src/lib.rs`|source|pattern scan / inventory review|
|`agent-graph/src/node.rs`|source|pattern scan / inventory review|
|`agent-graph/src/outcome.rs`|source|pattern scan / inventory review|
|`agent-graph/src/payload.rs`|source|pattern scan / inventory review|
|`agent-graph/src/prelude.rs`|source|pattern scan / inventory review|
|`agent-graph/src/reducer.rs`|source|pattern scan / inventory review|
|`agent-graph/src/retry.rs`|source|pattern scan / inventory review|
|`agent-graph/src/router.rs`|source|pattern scan / inventory review|
|`agent-graph/src/state.rs`|source|pattern scan / inventory review|
|`agent-graph/src/stream.rs`|source|pattern scan / inventory review|
|`agent-graph/tests/checkpointer_tests.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/execution_tests.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/integration_tests.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/interrupt_tests.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/parallel_tests.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/reducer_tests.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/retry_tests.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/routing_tests.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/runtime_tests.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/state_tests.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/step5_verification.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/streaming_tests.rs`|test|pattern scan / inventory review|
|`agent-graph/tests/subgraph_tests.rs`|test|pattern scan / inventory review|

## forge-memory-bridge

|File|Kind|Review depth|
|---|---|---|
|`forge-memory-bridge/Cargo.toml`|source|pattern scan / inventory review|
|`forge-memory-bridge/src/batch.rs`|source|direct review|
|`forge-memory-bridge/src/envelope.rs`|source|pattern scan / inventory review|
|`forge-memory-bridge/src/error.rs`|source|pattern scan / inventory review|
|`forge-memory-bridge/src/legacy.rs`|source|direct review|
|`forge-memory-bridge/src/lib.rs`|source|pattern scan / inventory review|
|`forge-memory-bridge/src/transform.rs`|source|direct review|

## job-queue

|File|Kind|Review depth|
|---|---|---|
|`job-queue/Cargo.toml`|source|pattern scan / inventory review|
|`job-queue/src/config.rs`|source|pattern scan / inventory review|
|`job-queue/src/db.rs`|source|direct review|
|`job-queue/src/error.rs`|source|pattern scan / inventory review|
|`job-queue/src/events.rs`|source|direct review|
|`job-queue/src/executor.rs`|source|direct review|
|`job-queue/src/lib.rs`|source|direct review|
|`job-queue/src/queue.rs`|source|pattern scan / inventory review|
|`job-queue/src/types.rs`|source|pattern scan / inventory review|

## knowledge-runtime

|File|Kind|Review depth|
|---|---|---|
|`knowledge-runtime/Cargo.toml`|source|pattern scan / inventory review|
|`knowledge-runtime/src/adapters/mod.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/adapters/semantic_memory.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/config.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/entity/code_ids.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/entity/mod.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/entity/registry.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/error.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/evidence/mod.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/evidence/support.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/ids.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/lib.rs`|source|direct review|
|`knowledge-runtime/src/obs/mod.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/obs/trace.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/projection/lifecycle.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/projection/mod.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/query/classify.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/query/merge.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/query/mod.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/query/route.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/runtime.rs`|source|direct review|
|`knowledge-runtime/src/temporal/claims.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/src/temporal/mod.rs`|source|pattern scan / inventory review|
|`knowledge-runtime/tests/cross_crate_proof.rs`|test|direct review|
|`knowledge-runtime/tests/invariant_tests.rs`|test|pattern scan / inventory review|
|`knowledge-runtime/tests/ugly_case_tests.rs`|test|pattern scan / inventory review|

## living-memory

|File|Kind|Review depth|
|---|---|---|
|`living-memory/Cargo.toml`|source|inventory only|

## semantic-memory

|File|Kind|Review depth|
|---|---|---|
|`semantic-memory/Cargo.toml`|source|pattern scan / inventory review|
|`semantic-memory/examples/basic_search.rs`|example|pattern scan / inventory review|
|`semantic-memory/examples/conversation_memory.rs`|example|pattern scan / inventory review|
|`semantic-memory/src/chunker.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/config.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/conversation.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/db.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/documents.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/embedder.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/episodes.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/error.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/graph.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/hnsw.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/knowledge.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/lib.rs`|source|direct review|
|`semantic-memory/src/pool.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/projection_import.rs`|source|direct review|
|`semantic-memory/src/projection_storage.rs`|source|direct review|
|`semantic-memory/src/quantize.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/search.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/storage.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/tokenizer.rs`|source|pattern scan / inventory review|
|`semantic-memory/src/types.rs`|source|pattern scan / inventory review|
|`semantic-memory/tests/brute_force_parity.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/chunker_tests.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/compaction.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/concurrent_access.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/conversation_search_tests.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/conversation_tests.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/db_tests.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/episode_identity.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/hardening_semantics.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/hardening_v5.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/hnsw_hotswap.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/hnsw_integration.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/hnsw_persistence.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/import_boundary_tests.rs`|test|direct review|
|`semantic-memory/tests/import_ugly_cases.rs`|test|direct review|
|`semantic-memory/tests/integration_tests.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/knowledge_tests.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/migration_v5.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/projection_v11_tests.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/quantization.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/quantization_pipeline.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/search_tests.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/step3_verification.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/step4_verification.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/storage_lifecycle.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/tokenizer_tests.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/trace_id_write_seam.rs`|test|pattern scan / inventory review|
|`semantic-memory/tests/vector_only_hnsw.rs`|test|pattern scan / inventory review|

## semantic-memory-forge

|File|Kind|Review depth|
|---|---|---|
|`semantic-memory-forge/Cargo.toml`|source|pattern scan / inventory review|
|`semantic-memory-forge/src/bundle.rs`|source|direct review|
|`semantic-memory-forge/src/envelope.rs`|source|direct review|
|`semantic-memory-forge/src/estimator.rs`|source|pattern scan / inventory review|
|`semantic-memory-forge/src/lib.rs`|source|direct review|

## stack-ids

|File|Kind|Review depth|
|---|---|---|
|`stack-ids/Cargo.toml`|source|pattern scan / inventory review|
|`stack-ids/src/digest.rs`|source|pattern scan / inventory review|
|`stack-ids/src/ids.rs`|source|pattern scan / inventory review|
|`stack-ids/src/lib.rs`|source|direct review|
|`stack-ids/src/scope.rs`|source|direct review|
|`stack-ids/src/trace.rs`|source|direct review|

## workspace

|File|Kind|Review depth|
|---|---|---|
|`CLAUDE.md`|source|direct review|
|`README.md`|source|direct review|
