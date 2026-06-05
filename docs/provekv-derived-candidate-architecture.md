# proveKV Derived Candidate Architecture

## Ownership boundaries

semantic-memory is the authoritative memory substrate. It stores text, metadata, projection records, temporal state, and f32 embeddings. Its f32 embeddings are the source of truth for final vector ranking.

proveKV/poly-kv is a rebuildable derived candidate artifact backend. It materializes generation-level compressed pools from deterministic semantic-memory embedding snapshots. The compressed pool can help produce candidates, but it is never final authority.

Downstream crates consume semantic-memory APIs and receipt/provenance fields. They do not depend directly on proveKV/poly-kv unless they are compression crates or compression benchmarks.

Forge owns raw evidence/export/fixity. claim-ledger owns append-only claim/evidence/provenance state. Kernel/oracle crates own verification. Similarity retrieval is candidate discovery only.

## Data flow

source systems / forge
  -> semantic-memory authoritative projections + f32 embeddings
  -> semantic-memory proveKV/poly-kv pool generations
  -> knowledge-runtime / llm-tool-runtime / agent-graph retrieval
  -> llm-pipeline prompt/provider receipts
  -> kernel/claim systems verify bounded inputs

## Candidate vs exact rerank vs verified premise

1. Candidate: a compressed or approximate backend returns a bounded set of possible matches.
2. Exact rerank: semantic-memory reloads authoritative f32 embeddings and reranks candidates exactly before returning final retrieval results.
3. Verified premise: a kernel/oracle/claim workflow evaluates evidence and explicitly promotes or verifies it. A retrieval candidate, even after exact rerank, is not a verified premise.

Required invariant: `DerivedVectorBackendPolicy::ProveKvPoolCandidateOnly` requires exact f32 rerank. Missing pool generations must be receipted as fallback, not silent behavior changes.

## Direct compression dependencies allowed

Allowed direct compression/proveKV-family dependencies:
- semantic-memory
- quant-governor
- scr-runtime-compression
- compression crates and benches, including poly-kv, fib-quant, turbo-quant, hnsw-bench, quant-eval, and related examples

Downstream provenance consumers:
- knowledge-runtime
- forge-memory-bridge
- llm-tool-runtime
- agent-graph
- llm-pipeline
- kernel-execution
- kernel-oracles
- semantic-memory-forge
- claim-ledger
- AiDENs profile crates

These downstream crates should use receipt structs, trace structs, or trait interfaces only.

## Receipt propagation map

semantic-memory emits:
- candidate_backend
- codec_family
- generation_id
- embedding_snapshot_digest
- pool_manifest_digest
- exact_rerank
- approximate
- fallback
- raw/post-filter/final candidate counts

knowledge-runtime carries route-aware derived candidate traces.

forge-memory-bridge carries post-import derived artifact lifecycle requests/statuses.

llm-tool-runtime, agent-graph, and llm-pipeline carry retrieved context / memory generation provenance.

kernel-execution, kernel-oracles, semantic-memory-forge, and claim-ledger separate candidate discovery from verified evidence/premises.

## Exclusions

Recall and Recall-Coding are explicitly excluded from this integration.

No doc or receipt may claim that this work reduces provider/framework KV-cache bytes directly. This integration is retrieval/artifact plumbing, not a PPL or inference-cache claim.
