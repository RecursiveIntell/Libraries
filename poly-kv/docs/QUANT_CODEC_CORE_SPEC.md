# `quant-codec-core` Implementation Spec

## Purpose

`quant-codec-core` is the smallest stable interface layer shared by compression crates. It must be boring, deterministic, dependency-light, and free of runtime authority.

## Required modules

```text
crates/quant-codec-core/src/
  lib.rs
  ids.rs
  digest.rs
  dtype.rs
  shape.rs
  codec.rs
  eval.rs
  error.rs
```

## Public types

### IDs and digests

```rust
pub struct CodecId(String);
pub struct CodecProfileDigest([u8; 32]);
pub struct ArtifactDigest([u8; 32]);
pub struct ModelFingerprint(String);
pub struct TokenizerFingerprint(String);
```

Rules:

- Validate empty IDs as errors.
- Provide `Display`, `Debug`, `Clone`, `Eq`, `Hash`, `Serialize`, `Deserialize`.
- Use stable canonical serialization for digest inputs.

### Shape model

```rust
pub enum KvRole { Key, Value }
pub enum DType { F32, F16, BF16, I8, U8, PackedBits }
pub enum KvLayout {
    LayersHeadsTokensDim,
    LayersTokensHeadsDim,
    RuntimeSpecific(String),
}

pub struct LayerId(pub u32);
pub struct HeadId(pub u32);
pub struct TokenSpan { pub start: u64, pub end: u64 }

pub struct KvTensorShape {
    pub layers: u32,
    pub key_heads: u32,
    pub value_heads: u32,
    pub seq_len: u64,
    pub head_dim: u32,
    pub layout: KvLayout,
    pub dtype: DType,
}
```

Validation:

- `layers > 0`
- `head_dim > 0`
- `seq_len > 0`
- token spans are half-open and non-empty
- GQA/MQA represented by `key_heads != value_heads`

### Trait surface

```rust
pub trait CodecProfile {
    fn codec_id(&self) -> CodecId;
    fn codec_version(&self) -> &str;
    fn profile_digest(&self) -> CodecProfileDigest;
    fn fixed_rate_bits(&self) -> Option<u16>;
    fn block_dim(&self) -> Option<u16>;
    fn is_lossy(&self) -> bool;
}

pub trait VectorCodec {
    type EncodedBlock;
    type Error;

    fn encode_block(&self, input: &[f32]) -> Result<Self::EncodedBlock, Self::Error>;
    fn decode_block(&self, block: &Self::EncodedBlock, out: &mut [f32]) -> Result<(), Self::Error>;
}

pub trait KvCacheCodec: VectorCodec {
    type EncodedCache;

    fn encode_kv_cache(
        &self,
        tensors: &[f32],
        shape: KvTensorShape,
    ) -> Result<Self::EncodedCache, Self::Error>;

    fn decode_slice(
        &self,
        cache: &Self::EncodedCache,
        request: KvSliceRequest,
        out: &mut [f32],
    ) -> Result<(), Self::Error>;
}
```

### Eval types

```rust
pub struct EvalReport {
    pub mse: Option<f64>,
    pub cosine_similarity: Option<f64>,
    pub max_abs_error: Option<f64>,
    pub bytes_exact: u64,
    pub bytes_encoded: u64,
    pub passed: bool,
    pub notes: Vec<String>,
}
```

## Dependencies

Allowed normal dependencies:

- `serde`
- `thiserror`
- `blake3` or equivalent digest crate

Allowed dev dependencies:

- `proptest`
- `serde_json`

Do not add heavy ML/runtime dependencies here.

## Required tests

- shape validation;
- digest stability;
- token span validation;
- serde roundtrip;
- trait mock compile test;
- no panic for invalid public inputs.

## Acceptance gate

```bash
cargo test -p quant-codec-core --all-targets
cargo clippy -p quant-codec-core --all-targets -- -D warnings
```
