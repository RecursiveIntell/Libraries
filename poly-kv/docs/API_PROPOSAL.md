# API Proposal

## Minimal 0.1.0-alpha.1 API

```rust
use poly_kv::*;
use quant_codec_core::*;

let shape = KvTensorShape::gqa(
    32,     // layers
    8,      // key_heads
    8,      // value_heads
    128,    // seq_len
    64,     // head_dim
    KvLayout::LayersHeadsTokensDim,
    DType::F32,
)?;

let policy = CompressionPolicyV1::alpha_reference();

let pool = SharedKvPool::builder()
    .model_fingerprint(ModelFingerprint::new("synthetic:test-model")?)
    .tokenizer_fingerprint(TokenizerFingerprint::new("synthetic:test-tokenizer")?)
    .shape(shape)
    .policy(policy)
    .exact_fallback(ExactFallback::from_blocks(exact_blocks.clone()))
    .key_codec(Q8KeyCodec::symmetric_per_block())
    .value_codec(RawExactValueCodec::default())
    .build_from_blocks(exact_blocks)?;

let reader = pool.attach_reader(ReaderConfig::default())?;
let decoded = reader.decode_slice(KvSliceRequest::layer_span(LayerId(0), TokenSpan::new(0, 16)?))?;

assert!(pool.build_receipt().quality_gate.passed);
assert_eq!(pool.reader_count(), 1);
```

## Advanced future API, not required in alpha

```rust
let pool = SharedKvPool::builder()
    .value_codec(TurboQuantValueCodec::from_profile(profile)?)
    .quality_gate(QualityGate::attention_drift(0.01))
    .build_from_runtime_cache(cache)?;

let decision = governor.choose(CompressionRequest::kv_pool(&pool_context))?;
```

## Error model

Use typed errors:

```rust
pub enum PolyKvError {
    InvalidShape { reason: String },
    InvalidSpan { start: u64, end: u64 },
    MissingFallback,
    Codec(String),
    QualityGateFailed(String),
    Manifest(String),
    Serialization(String),
}
```

## Feature flags

```toml
[features]
default = ["std", "serde"]
std = []
serde = ["dep:serde"]
turbo-quant-adapter = ["dep:turbo-quant"]
fibquant-adapter = ["dep:fibquant"]
bench = ["dep:criterion"]
proptest = ["dep:proptest"]
```

Feature flags must not be enabled by default for optional adapters.
