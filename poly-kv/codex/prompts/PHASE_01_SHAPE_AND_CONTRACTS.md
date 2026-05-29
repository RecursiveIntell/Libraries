# Phase 01 — Shape V2 and contract hardening

Implement or stage `KvCacheShapeV2` in `quant-codec-core`.

Required fields:

- batch
- layers
- num_q_heads
- num_kv_heads
- seq_len
- head_dim
- layout
- dtype
- attention_kind: Mha, Mqa, Gqa, Unsupported(String)

Rules:

- `batch > 0`
- `layers > 0`
- `num_q_heads > 0`
- `num_kv_heads > 0`
- `seq_len > 0`
- `head_dim > 0`
- MHA: `num_q_heads == num_kv_heads`
- MQA: `num_kv_heads == 1` and `num_q_heads > 1`
- GQA: `num_q_heads > num_kv_heads`, `num_kv_heads > 1`, `num_q_heads % num_kv_heads == 0`
- unsupported/MLA/hybrid must fail closed unless explicitly adapter-owned

Add tests for MHA/MQA/GQA and invalid cases. Preserve existing tests or update them with migration helpers.

Gate: `cargo test -p quant-codec-core shape` passes; no downstream breakage hidden.
