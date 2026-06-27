# Research Pass Control — No HuggingFace, No CUDA

Date: 2026-06-26
Repo: /home/sikmindz/Coding/Libraries
Plan: /home/sikmindz/Coding/Libraries/.hermes/plans/2026-06-26_160109-research-implementation-plan-no-hf-no-cuda.md

## In scope

1. RoPE-aware KV-cache bit allocation in Rust/CPU code.
2. HyperQuant/lattice-inspired experimental codec surfaces where they do not require HuggingFace or CUDA.
3. semantic-memory retrieval quality improvements: hubness scoring, optional admission/downweight policy, centroid/cluster-first routing prototypes, dynamic-update receipts.
4. Bitemporal property graph model/query routing.
5. Contextual reinstatement and perspective-bounded recall.
6. Text-to-Cypher / structured graph query AST and parser helpers without model invocation.
7. TREC/RAG-style local fixture benchmark harness.
8. Formal-check and diagnostic-localization receipts.
9. Documentation, claim boundaries, and semantic-memory durable fact sync.

## Out of scope

1. HuggingFace model loading, datasets API, transformers integration, safetensors, tokenizers, or HF Hub download logic.
2. CUDA kernels, CUDA profiling, FlashAttention serving paths, H800/A100-specific optimization, or GPU reproducibility claims.
3. Public performance claims based only on paper results.
4. Publishing unless a separate release gate verifies package state and dependency readiness.

## Public claim boundary

Safe to claim after implementation and local tests:
- Experimental Rust CPU-side prototype exists.
- Local deterministic tests pass.
- Local synthetic benchmark receipt exists.
- API is additive and receipt-bearing.

Unsafe until locally reproduced:
- Paper speedups such as 400x over HNSW.
- Paper memory reductions such as fp16 memory comparisons.
- Block-GTQ-equivalent quality.
- fp16-comparable model quality.
- Production KV runtime readiness.
- CUDA/GPU serving performance.

## Validation rule

Every phase must emit receipts. A phase is not complete without at least one of:
- focused cargo test output,
- focused cargo check output,
- benchmark receipt,
- explicit blocked/failure receipt with command output.

## Agent coordination rule

Claude Code and Codex may be used for independent crates/files. Controller must verify all agent self-reports with cargo or file inspection. Do not trust agent summaries as receipts.
