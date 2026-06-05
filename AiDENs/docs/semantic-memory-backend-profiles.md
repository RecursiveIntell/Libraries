# proveKV / semantic-memory backend profiles for AiDENs

AiDENs does not integrate proveKV directly. AiDENs selects a semantic-memory backend profile; semantic-memory owns authoritative f32 embeddings, derived candidate artifacts, and exact rerank.

Profiles:

small:
  derived_vector_backend = disabled
  exact_rerank = true
  use for tiny local memories and tests.

medium:
  derived_vector_backend = turbo_quant_candidate_only
  exact_rerank = true
  use for moderate corpora where per-vector derived artifacts are enough.

large:
  derived_vector_backend = provekv_pool_candidate_only
  exact_rerank = true
  use for large project/session/corpus memories where generation-level shared-pool economics matter.

Boundary rules:
- AiDENs profile crates must not depend directly on poly-kv, fib-quant, turbo-quant, or proveKV APIs.
- Candidate backends never decide identity, authorization, promotion, or verification.
- Missing/stale derived artifacts must be receipted as fallback, not hidden.
- The large profile means `provekv_pool_candidate_then_exact_f32`, not provider/framework KV-cache compression.
