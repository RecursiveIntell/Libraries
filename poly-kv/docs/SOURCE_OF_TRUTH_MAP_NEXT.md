# Source-of-Truth Map — Next Pass

| Surface | Owner | Must not own | Notes |
|---|---|---|---|
| `quant-codec-core` | IDs, digests, dtype, shape V2, codec/eval traits | pool storage, Python bindings, runtime adapters, policy governor | Keep dependency-light. No PyO3, no HF, no Torch. |
| `poly-kv` | immutable shared pool, exact fallback, q8 key codec, raw exact value codec, manifests, receipts, realized accounting | TurboQuant/FibQuant math, adaptive routing, HF internals, daemon/service mode | Rust core must remain clean and optional-dependency-light. |
| `poly-kv-python` / `python/poly_kv` | Python ergonomics, PyO3 wrappers, exceptions, typing, NumPy/DLPack/HF prototype adapters | canonical Rust semantics, codec math, runtime truth store | Sidecar wraps core; it does not redefine semantics. |
| `turbo-quant` | TurboQuant value/key math if inspected | pool semantics, reader accounting, Python HF surface | Optional adapter only after API inspection. |
| `fib-quant` | FibQuant math if inspected | pool semantics, adaptive governor | Optional/experimental only. |
| future `quant-governor` | codec selection, adaptive budgets, decision receipts | raw codec math, pool immutability | Deferred. Do not build in this pass. |
| future runtime adapters | vLLM/llama.cpp/Candle/Burn/tch specifics | core crate compatibility claims | Deferred or prototype-only. |
| harness / ReceiptBench | benchmark runner and cross-system receipts | core crate API semantics | Can consume receipts; must not redefine them. |

## Forbidden substitutions

- No local TurboQuant or FibQuant reimplementation inside `poly-kv`.
- No hidden compatibility layer that pretends to support all HF models.
- No vector index, cache, benchmark script, or Python object becomes canonical truth.
- No adaptive controller inside `poly-kv`.
- No daemon-first rewrite.
- No production-serving claim.
