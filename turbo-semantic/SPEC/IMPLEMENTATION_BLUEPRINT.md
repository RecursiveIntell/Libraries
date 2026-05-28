# Implementation Blueprint

This is a target shape, not a mandate to ignore existing repo structure.

## turbo-quant target files

```text
src/profile.rs              # codec profile, digest, enum contracts
src/artifact.rs             # encoded artifact, checksum, serde
src/bitpack.rs              # bitpacking utilities
src/query.rs                # prepared query workspace
src/error.rs                # add profile/corruption errors
src/qjl.rs                  # add packed QJL support
src/polar.rs                # storage accounting / compact angle plan
src/turbo.rs                # profile/artifact/prepared scoring integration
src/lib.rs                  # exports
tests/profile_contract.rs
tests/encoded_artifact.rs
tests/bitpack.rs
tests/prepared_scoring.rs
tests/storage_accounting.rs
```

## semantic-memory target files

```text
semantic-memory/src/vector_codec.rs
semantic-memory/src/vector_codec/types.rs
semantic-memory/src/vector_codec/raw.rs
semantic-memory/src/vector_codec/sq8.rs
semantic-memory/src/vector_codec/turbo.rs        # cfg(feature = "turbo-quant-codec")
semantic-memory/src/vector_codec/storage.rs
semantic-memory/src/vector_codec/eval.rs
semantic-memory/tests/vector_codec_abstraction.rs
semantic-memory/tests/turbo_quant_shadow.rs       # cfg(feature = "turbo-quant-codec")
semantic-memory/tests/vector_codec_eval.rs
semantic-memory/tests/search_score_provenance.rs
```

## Minimal `VectorCodec` type sketch

```rust
pub trait VectorCodec {
    fn family(&self) -> &'static str;
    fn profile_digest(&self) -> Option<&str>;

    fn encode(&self, vector: &[f32]) -> Result<EncodedVector>;

    fn prepare_query(&self, query: &[f32]) -> Result<QueryState>;

    fn score_encoded(
        &self,
        encoded: &EncodedVector,
        query: &QueryState,
    ) -> Result<CodecScore>;

    fn decode_lossy(&self, encoded: &EncodedVector) -> Result<Option<Vec<f32>>>;
}
```

## Minimal score provenance sketch

```rust
pub struct VectorScoreProvenance {
    pub codec_family: String,
    pub profile_digest: Option<String>,
    pub approximation_class: ApproximationClass,
    pub approximate_score: Option<f32>,
    pub exact_score: Option<f32>,
    pub reranked_from_f32: bool,
    pub degradation_flags: Vec<String>,
}
```

## Minimal shadow encode receipt sketch

```rust
pub struct VectorEncodeReceiptV1 {
    pub entity_type: String,
    pub entity_key: String,
    pub codec_family: String,
    pub profile_digest: String,
    pub encoded_len: usize,
    pub checksum: String,
    pub status: EncodeStatus,
    pub recorded_at: String,
    pub degradation_flags: Vec<String>,
}
```
