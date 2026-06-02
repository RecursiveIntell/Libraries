# TurboQuant × Semantic-Memory Integration Spec

## Objective

Add TurboQuant as a pluggable, optional, evidence-bearing compressed vector codec for semantic-memory.

## Canonical ownership

| Concept | Owner | Rule |
|---|---|---|
| TurboQuant math | `turbo-quant` | Do not duplicate in semantic-memory. |
| Polar/QJL/rotation internals | `turbo-quant` | Expose stable profiles/artifacts only. |
| Vector codec trait | `semantic-memory` | Generic abstraction for raw/SQ8/Turbo. |
| Search/query semantics | `semantic-memory` | Approximation must be disclosed. |
| Evaluation records | `semantic-memory` | Persist/evaluate codec outcomes. |
| Source IDs/digests/traces | existing stack crates where available | Reuse canonical IDs, do not invent replacement identity systems. |

## Functional requirements

### R1 — TurboQuant profile contract

`turbo-quant` must expose a profile type equivalent to:

```rust
pub struct TurboQuantCodecProfileV1 {
    pub codec_family: String,       // "turbo-quant"
    pub codec_version: String,      // crate semver
    pub dim: usize,
    pub bits: u8,
    pub projections: usize,
    pub seed: u64,
    pub rotation_kind: RotationKindV1,
    pub radius_encoding: RadiusEncodingV1,
    pub angle_encoding: AngleEncodingV1,
    pub qjl_encoding: QjlEncodingV1,
    pub distance_metric: DistanceMetricV1,
    pub canonicalization_version: u16,
    pub profile_digest: String,
}
```

The digest must be deterministic from canonical profile fields, not from ephemeral runtime state.

### R2 — Encoded vector artifact

`turbo-quant` must expose an encoded artifact equivalent to:

```rust
pub struct EncodedVectorArtifactV1 {
    pub profile_digest: String,
    pub dim: usize,
    pub encoded_bytes: Vec<u8>,
    pub checksum: String,
    pub encoded_len: usize,
}
```

The artifact must reject profile mismatch and corrupted bytes.

### R3 — Query workspace

TurboQuant must provide a query-prepared scoring path:

```rust
let query_state = quantizer.prepare_query(&query)?;
let score = quantizer.inner_product_prepared(&code, &query_state)?;
let cosine = quantizer.cosine_estimate_prepared(&code, &query_state)?;
```

Do not regenerate expensive projection/rotation state for every candidate when one query is scoring many candidates.

### R4 — Semantic-memory vector codec trait

`semantic-memory` must introduce an internal trait equivalent to:

```rust
pub trait VectorCodec {
    fn family(&self) -> &'static str;
    fn profile_digest(&self) -> Option<&str>;
    fn encode(&self, vector: &[f32]) -> Result<EncodedVector>;
    fn prepare_query(&self, query: &[f32]) -> Result<QueryState>;
    fn score_encoded(&self, encoded: &EncodedVector, query: &QueryState) -> Result<CodecScore>;
    fn decode_lossy(&self, encoded: &EncodedVector) -> Result<Option<Vec<f32>>>;
}
```

Do not force TurboQuant-specific types into generic search APIs.

### R5 — Feature gate

Add:

```toml
[features]
turbo-quant-codec = ["dep:turbo-quant"]
```

Only if `turbo-quant` is a real dependency. Do not add an absolute path. Preferred location:

```toml
turbo-quant = { path = "../turbo-quant", optional = true }
```

This requires `turbo-quant` as sibling of `semantic-memory` under `/home/sikmindz/Coding/Libraries/`.

### R6 — Shadow mode

Add config options equivalent to:

```rust
pub struct VectorCodecConfig {
    pub shadow_turbo_quant: bool,
    pub turbo_bits: u8,
    pub turbo_projections: usize,
    pub turbo_seed: u64,
    pub persist_encoded_vectors: bool,
    pub evaluate_shadow_search: bool,
    pub allow_approximate_results: bool,
}
```

Default must preserve existing behavior.

### R7 — Storage

If persisted, TurboQuant encoded vectors must be stored separately from raw embeddings and SQ8. Required fields:

- entity type;
- entity key;
- codec family;
- profile digest;
- encoded bytes;
- checksum;
- created recorded time;
- encode receipt JSON;
- degradation flags if any.

### R8 — Evaluation

Add an evaluation artifact/record equivalent to:

```rust
pub struct VectorCodecEvaluationRunV1 {
    pub run_id: String,
    pub codec_family: String,
    pub profile_digest: String,
    pub corpus_snapshot: String,
    pub query_count: usize,
    pub recall_at_10: f32,
    pub top_k_agreement: f32,
    pub score_correlation: Option<f32>,
    pub avg_encoded_bytes: usize,
    pub raw_f32_bytes: usize,
    pub sq8_bytes: Option<usize>,
    pub latency_summary: LatencySummary,
    pub degradation_count: usize,
}
```

Evaluation output may initially be JSON under `target/vector-codec-evals/` if DB persistence is too invasive. Prefer DB tables only if migration impact is controlled.

### R9 — Search disclosure

Any approximate TurboQuant-backed result must expose:

- codec family;
- profile digest;
- approximate score;
- exact/f32 rerank score if present;
- approximation class;
- degradation flags;
- rerank status.

### R10 — Failure behavior

Must explicitly fail or degrade on:

- missing codec profile;
- profile mismatch;
- dimension mismatch;
- checksum mismatch;
- unsupported codec version;
- missing raw vector when f32 rerank required;
- shadow encode failure.

Shadow encode failure must not break authoritative write path unless strict mode is enabled.
