# 03 — Target API Spec

This is the API Codex should implement unless the current repository proves a better local convention.

## Core types

```rust
pub struct FibQuantProfileV1 {
    pub schema_version: String,
    pub ambient_dim: u32,
    pub block_dim: u32,
    pub codebook_size: u32,
    pub paper_rate_bits_per_coord: f64,
    pub wire_index_bits: u8,
    pub wire_bits_per_coord: f64,
    pub norm_format: NormFormat,
    pub rotation_seed: u64,
    pub codebook_seed: u64,
    pub codebook_version: String,
    pub source_mode: SourceMode,
    pub radius_method: RadiusMethod,
    pub direction_method: DirectionMethod,
    pub lloyd_restarts: u32,
    pub lloyd_iterations: u32,
    pub training_samples: u32,
    pub empty_cell_policy: EmptyCellPolicy,
}

pub enum NormFormat {
    Fp16Paper,
    F32Reference,
}

pub enum SourceMode {
    CanonicalSphericalBeta,
    ReferenceGaussianProjection,
}

pub enum RadiusMethod {
    BetaQuantile,
    K2ClosedForm,
    LargeDSingleShellExplicit,
}

pub enum DirectionMethod {
    FibonacciSpiral,
    FibonacciSphere,
    RobertsKronecker,
}

pub enum EmptyCellPolicy {
    SplitHighestDistortion,
    FailClosed,
}
```

## Codebook

```rust
pub struct FibCodebookV1 {
    pub schema_version: String,
    pub profile: FibQuantProfileV1,
    pub profile_digest: String,
    pub codebook_digest: String,
    pub codewords: Vec<f32>,
    pub init_mse: f64,
    pub training_mse: f64,
    pub refinement_report: LloydReportV1,
}
```

## Encoded artifact

```rust
pub struct FibCodeV1 {
    pub schema_version: String,
    pub profile_digest: String,
    pub codebook_digest: String,
    pub ambient_dim: u32,
    pub block_dim: u32,
    pub norm_format: NormFormat,
    pub norm_payload: Vec<u8>,
    pub wire_index_bits: u8,
    pub block_count: u32,
    pub indices: Vec<u8>,
}
```

## Quantizer

```rust
pub struct FibQuantizer {
    profile: FibQuantProfileV1,
    codebook: FibCodebookV1,
    rotation: StoredRotationLike,
}

impl FibQuantizer {
    pub fn new(profile: FibQuantProfileV1) -> Result<Self>;
    pub fn from_codebook(codebook: FibCodebookV1) -> Result<Self>;
    pub fn encode(&self, x: &[f32]) -> Result<FibCodeV1>;
    pub fn decode(&self, code: &FibCodeV1) -> Result<Vec<f32>>;
    pub fn encode_with_receipt(&self, x: &[f32]) -> Result<(FibCodeV1, FibQuantCompressionReceiptV1)>;
    pub fn reconstruction_mse(&self, x: &[f32]) -> Result<f64>;
    pub fn cosine_similarity(&self, x: &[f32]) -> Result<f64>;
}
```

## Public helpers

```rust
pub fn beta_d_k(d: usize, k: usize) -> Result<f64>;
pub fn radius_quantile(d: usize, k: usize, n: usize, n_total: usize) -> Result<f64>;
pub fn radius_quantile_k2_closed_form(d: usize, q: f64) -> Result<f64>;
pub fn sample_spherical_beta(d: usize, k: usize, rng: &mut impl Rng) -> Result<Vec<f64>>;
pub fn sample_reference_projection(d: usize, k: usize, rng: &mut impl Rng) -> Result<Vec<f64>>;
pub fn fibonacci_spiral_2d(n: usize) -> Result<Vec<[f64; 2]>>;
pub fn fibonacci_sphere_3d(n: usize) -> Result<Vec<[f64; 3]>>;
pub fn roberts_kronecker(k: usize, n: usize) -> Result<Vec<Vec<f64>>>;
```

## Error taxonomy

`FibQuantError` must distinguish:

- `ZeroDimension`
- `InvalidBlockDim`
- `DimensionNotDivisible`
- `InvalidCodebookSize`
- `NonFiniteInput`
- `ZeroNorm`
- `ProfileDigestMismatch`
- `CodebookDigestMismatch`
- `CorruptPayload`
- `IndexOutOfRange`
- `NumericalFailure`
- `EmptyCellRepairFailed`
- `DependencyUnsupported`
