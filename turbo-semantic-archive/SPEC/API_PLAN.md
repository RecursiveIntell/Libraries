# API Plan

## Preserve existing APIs

Existing `MemoryStore` APIs must keep behavior unless caller opts in.

## New/extended internal APIs

Suggested module layout:

```text
semantic-memory/src/vector_codec.rs
semantic-memory/src/vector_codec/raw.rs
semantic-memory/src/vector_codec/sq8.rs
semantic-memory/src/vector_codec/turbo.rs   # cfg(feature = "turbo-quant-codec")
semantic-memory/src/vector_codec/eval.rs
semantic-memory/src/vector_codec/storage.rs
semantic-memory/src/vector_codec/types.rs
```

If the current module layout makes this awkward, Codex may choose a smaller layout, but must preserve these separations logically.

## Public or semi-public config

Add to `MemoryConfig` or nested search/vector config:

```rust
pub struct VectorCodecConfig {
    pub shadow_turbo_quant: bool,
    pub persist_encoded_vectors: bool,
    pub evaluate_shadow_search: bool,
    pub allow_approximate_results: bool,
    pub turbo_bits: u8,
    pub turbo_projections: Option<usize>,
    pub turbo_seed: u64,
}
```

Defaults:

```text
shadow_turbo_quant = false
persist_encoded_vectors = false
evaluate_shadow_search = false
allow_approximate_results = false
turbo_bits = 8
turbo_projections = None // resolve to dim / 4
turbo_seed = stable default, documented
```

## Explained result surface

Add optional fields to result metadata or a parallel explained result type:

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

Avoid breaking existing `SearchResult` if possible.
