# `poly-kv` Implementation Spec

## Purpose

`poly-kv` implements a shared compressed KV-cache pool primitive. It must be useful without runtime adapters and safe to use as the third primitive under a future adaptive compression controller.

## Required modules

```text
crates/poly-kv/src/
  lib.rs
  error.rs
  manifest.rs
  receipts.rs
  pool.rs
  reader.rs
  memory.rs
  metrics.rs
  codecs/
    mod.rs
    raw_exact.rs
    q8_keys.rs
    value.rs
  adapters/
    mod.rs
    turbo_quant.rs
    fibquant.rs
```

Adapters are feature-gated and may be stubbed if the external APIs are not inspected.

## Required concepts

### `SharedKvPool`

Immutable shared pool built once from exact KV blocks or synthetic fixtures. Readers attach without duplicating compressed pool bytes.

Required API shape:

```rust
pub struct SharedKvPool { /* immutable blocks + manifest + receipts */ }

impl SharedKvPool {
    pub fn builder() -> PoolBuilder;
    pub fn manifest(&self) -> &KvPoolManifestV1;
    pub fn build_receipt(&self) -> &PoolBuildReceiptV1;
    pub fn attach_reader(&self, config: ReaderConfig) -> Result<PoolReader, PolyKvError>;
    pub fn exact_fallback_ref(&self) -> Option<&ExactFallbackRef>;
    pub fn encoded_bytes(&self) -> u64;
}
```

### `PoolReader`

Per-reader decode/attach surface. It may maintain per-reader decode scratch state but must not mutate or duplicate the shared compressed pool.

```rust
pub struct PoolReader { /* Arc<SharedKvPoolInner> + ReaderId + config */ }

impl PoolReader {
    pub fn decode_layer(&self, layer: LayerId) -> Result<DecodedLayer, PolyKvError>;
    pub fn decode_slice(&self, req: KvSliceRequest) -> Result<DecodedKvSlice, PolyKvError>;
    pub fn injection_receipt(&self) -> &ReaderInjectionReceiptV1;
}
```

### Codecs

- `RawExactCodec`: exact storage and decode. Required.
- `Q8KeyCodec`: reference q8 key quantization. Required.
- `ValueCodec` trait: required.
- `TurboQuantValueCodec`: optional feature, only after API inspection.
- `FibQuantValueCodec`: optional feature, only after API inspection.

### Receipts

All material operations must produce typed receipts or return them to caller.

Required receipt families:

- `PoolBuildReceiptV1`
- `ReaderInjectionReceiptV1`
- `DecodeReceiptV1`
- `FallbackReceiptV1`
- `CompressionEvalReceiptV1`

## Invariants

1. Pool blocks are immutable after construction.
2. Reader state is isolated.
3. Compressed pool bytes are counted once regardless of reader count.
4. Decoded scratch memory is counted separately from encoded shared memory.
5. Shape mismatch returns typed error, never silent coercion.
6. Lossy path must expose degradation/eval state.
7. Exact fallback must be available for alpha release fixtures.
8. Public APIs must avoid panics for invalid input.
9. No unsafe code by default.

## Required tests

- synthetic exact fallback roundtrip;
- q8 key quantization/dequantization drift bounded by documented threshold;
- reader attach does not duplicate pool bytes;
- memory accounting exact vs encoded vs per-reader scratch;
- mismatched shape rejected;
- malformed token span rejected;
- deterministic build receipt for same input/profile;
- serialization roundtrip for manifests/receipts;
- feature-gated adapters compile off by default.

## Public claim boundary

Safe wording:

> `poly-kv` is a Rust research-to-implementation crate for shared KV-cache pool manifests, q8 key compression, exact fallback, and receipt-bearing compressed-pool experiments.

Unsafe until reproduced:

- production-ready serving runtime;
- vLLM/llama.cpp/Candle/Burn/tch compatibility;
- fixed percentage memory reduction;
- quality preservation on real models;
- benchmark superiority;
- affiliation with PolyKV authors.
