# semantic-memory harness — DEPRECATED for retrieval-quality claims (P27)

**Status: deprecated 2026-06-10 by the P27 real-workload audit.**

The P26 harness in this directory runs a synthetic 1,000×384 unit-vector
corpus with 50 queries and reports `recall@10 = 1.0`. P27 ran the same
codec on a real BEIR `scifact` corpus (5,181 docs, 300 test queries, 339
qrels) and got `exact_rerank_recovery_at_1 = 0.307` — i.e. the codec
misses the top-1 ground truth 70% of the time, even with 4× oversample.

**The synthetic harness is retained for fast CI smoke tests**, not as
deployment evidence. Any claim that derives from this synthetic run
should be replaced by a P27-class real-workload receipt, or removed
entirely.

## Replacement harness

For real-workload evidence, use the P27 BEIR benchmark:

- Source: `docs/codex-runs/P27/build_corpus.py`, `examples/real_bench.rs`
- Receipt: `docs/codex-runs/P27/REAL_BENCH_RECEIPT.json`
- Audit: `docs/codex-runs/P27/REAL_BENCH_AUDIT.md`
- Skill: `~/.hermes/skills/mlops/turbo-quant-beir-bench-harness/`

The skill describes the full pipeline (BEIR download → Ollama embed →
TQCB binary → Rust bench → RealBenchmarkReceiptV1) and the pass/fail
rubric (top-k overlap ≥ 0.30 AND exact_rerank_recovery_at_1 ≥ 0.80).

## Rules (unchanged from the P26 template)

- This harness is local proof infrastructure, not part of the publishable
  `turbo-quant` crate.
- It may depend on `semantic-memory` by local path.
- It must be excluded from crates.io package scope.
- It must not copy semantic-memory internals into `turbo-quant/src`.
- It must emit `SemanticMemoryProofReceiptV1` JSON.

## KV-shadow evidence (still valid)

The P26 evidence in `docs/codex-runs/P26/SEMANTIC_MEMORY_PROOF_RECEIPT.json`
covers the KV-cache shadow mode, which is a different problem (per-vector
reconstruction, not ranking). The P27 audit does **not** invalidate the
KV-shadow evidence — see the audit writeup for the distinction.
