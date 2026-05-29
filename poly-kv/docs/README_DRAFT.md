# poly-kv

Shared compressed KV-cache pool infrastructure for Rust experiments.

## Status

`poly-kv` is an alpha-stage research-to-Rust implementation crate. It is designed to provide typed manifests, exact fallback, receipt-bearing shared-pool operations, q8 key compression experiments, and pluggable value-codec boundaries.

It is not a production serving runtime.

## What this crate is

- A shared KV-pool manifest and reader abstraction.
- A reference q8 key codec.
- An exact fallback path for alpha fixtures.
- A receipt-bearing compression/decode surface.
- A future primitive for adaptive compression experiments.

## What this crate is not

- Not the original PolyKV authors' reference implementation.
- Not a vLLM, llama.cpp, Candle, Burn, or tch runtime adapter.
- Not a production KV-cache serving engine.
- Not a completed adaptive compression controller.
- Not a claim of reproduced PolyKV paper metrics.

## Example

```rust
use poly_kv::*;
use quant_codec_core::*;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let shape = KvTensorShape::gqa(
    2, 2, 2, 8, 4,
    KvLayout::LayersHeadsTokensDim,
    DType::F32,
)?;

let blocks: Vec<ExactKvBlock> = Vec::new(); // provide synthetic or exported KV blocks

let pool = SharedKvPool::builder()
    .model_fingerprint(ModelFingerprint::new("synthetic:test-model")?)
    .tokenizer_fingerprint(TokenizerFingerprint::new("synthetic:test-tokenizer")?)
    .shape(shape)
    .policy(CompressionPolicyV1::alpha_reference())
    .exact_fallback(ExactFallback::from_blocks(blocks.clone()))
    .build_from_blocks(blocks)?;

let reader = pool.attach_reader(ReaderConfig::default())?;
let _slice = reader.decode_slice(
    KvSliceRequest::layer_span(LayerId(0), TokenSpan::new(0, 1)?).for_role(KvRole::Key),
)?;
# Ok(())
# }
```

## Feature status

| Feature | Status |
|---|---|
| shared pool manifest | alpha target |
| exact fallback | alpha target |
| q8 key codec | alpha target |
| raw exact value codec | alpha target |
| TurboQuant adapter | optional, experimental, API-dependent |
| FibQuant adapter | optional, experimental, API-dependent |
| real model benchmarks | not claimed |
| production serving | not claimed |

## Safety model

Every lossy path must have:

- declared codec profile;
- exact fallback reference;
- quality/eval receipt where applicable;
- typed decode/fallback receipt;
- no hidden fallback;
- no silent shape coercion.

## Attribution

This crate is inspired by PolyKV-style shared compressed KV-cache pool ideas. It is independent and does not claim affiliation with or endorsement by the PolyKV paper authors.

## License

Choose MIT OR Apache-2.0 unless external dependencies force otherwise. Do not include GPL-derived implementation code.
