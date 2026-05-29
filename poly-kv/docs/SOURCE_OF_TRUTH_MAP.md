# Source-of-Truth Map

## Target crate map

| Crate | Owns | Must not own | Required outputs |
|---|---|---|---|
| `quant-codec-core` | codec IDs, profile digests, KV shape/layout types, codec traits, eval report types | pool state, runtime policy, app storage, Turbo/Fib math | `CodecId`, `CodecProfileDigest`, `KvTensorShape`, `VectorCodec`, `KvCacheCodec`, `EvalReport` |
| `poly-kv` | `SharedKvPool`, manifests, pool reader, q8 key codec, exact fallback, receipts, memory accounting | TurboQuant math, FibQuant math, adaptive routing, runtime authority, app integration | `KvPoolManifestV1`, `PoolBuildReceiptV1`, `ReaderInjectionReceiptV1`, tests |
| `turbo-quant` | existing TurboQuant algorithms | pool semantics, governor policy | optional feature-gated adapter only after API inspection |
| `fibquant` / `fib-quant` | FibQuant algorithms | pool semantics, governor policy | optional feature-gated adapter only after API inspection |
| future `quant-governor` | codec selection, compression policy, eval orchestration, decision receipts | raw codec math, app truth storage | not implemented in this pass |
| future `scr-runtime-compression` | permits, budgets, rollback, quarantine, runtime receipts | codec math | not implemented in this pass |

## Forbidden local substitutes

- Do not reimplement TurboQuant or FibQuant algorithms inside `poly-kv`.
- Do not build an adaptive controller in `poly-kv`.
- Do not create a hidden app-specific memory store.
- Do not use vector indexes or compressed KV blocks as canonical truth.
- Do not treat benchmarks or README examples as proof of production readiness.
