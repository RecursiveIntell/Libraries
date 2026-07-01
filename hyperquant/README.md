# hyperquant

Experimental lattice quantization primitives for vectors and model-adjacent compression research.

`hyperquant` is a small, auditable Rust crate for testing lattice-shaped quantization building blocks under explicit claim boundaries. It is designed to be used as a primitive layer for evidence-first compression work: benchmark it, attach receipts, route it through policy, and only then integrate it into larger systems.

This crate does **not** claim HyperQuant paper parity, model-quality preservation, production readiness, CUDA support, or HuggingFace integration. The current release is a CPU-local primitive crate with deterministic tests and receipts.

## What this gives you

`hyperquant` provides:

- **Z1 scalar lattice quantization** — independent integer-lattice quantization per coordinate.
- **A2 triangular-lattice quantization** — nearest-point search over the A2 basis for 2D pairs, with deterministic tie behavior.
- **D4 checkerboard-lattice quantization** — nearest-point repair over 4D chunks, with deterministic A2/Z1 handling for trailing dimensions.
- **Explicit roadmap E8 variant** — `E8` is named as a known lattice target but returns `UnsupportedLattice` until a real implementation lands.
- **Rice byte accounting** — signed code streams can be zig-zag/Rice encoded and profiled for real bit/byte estimates instead of fixed i16-only accounting.
- **Seeded RHT helper** — CPU-local per-tile sign flips plus normalized Hadamard transforms for preconditioning experiments.
- **Feature-gated codec adapter** — `compat` exposes HyperQuant through `quant-codec-core` traits without making downstream crates depend on internals.
- **Result-bound receipts** — every successful quantization result can produce a `HyperQuantReceiptV1` tied to the input digest, config digest, code length, MSE, scale, and claim boundary.
- **Conservative failure modes** — empty input, NaN/Inf input, unsupported lattices, and non-finite artifacts are typed errors, not silent fallbacks.
- **Serde-friendly outputs** — configs, results, receipts, and lattice kinds derive `Serialize` / `Deserialize`.
- **Deterministic test surface** — integration tests cover quantization behavior, receipt binding, serialization, unsupported lattices, and overflow rejection.

## Claim boundary

Safe to claim today:

- CPU-local Rust lattice quantization primitives.
- Z1 is implemented.
- A2 nearest-point quantization for pairs is implemented.
- D4 nearest-point quantization for 4D chunks is implemented.
- E8 is explicitly unsupported, not a fake placeholder.
- Rice encoding/profiling is implemented for local code streams.
- Seeded CPU-local RHT helper is implemented.
- `quant-codec-core` adapter is available behind the `compat` feature.
- Receipts are bound to the quantization result.
- Tests, clippy, and publish dry-run passed locally before release.

Not safe to claim today:

- HyperQuant paper parity.
- Rate-distortion optimality across model layers.
- LLM or diffusion model quality preservation.
- Superiority over GPTQ, AWQ, TurboQuant, FibQuant, or any other codec.
- CUDA/GPU acceleration.
- HuggingFace/model integration.
- Production readiness for semantic-memory or KV-cache compression.

That boundary is intentional. This crate is meant to generate evidence, not marketing claims.

## Install

Add to `Cargo.toml`:

```toml
[dependencies]
hyperquant = "0.1.1"
```

Or, while working inside the RecursiveIntell Libraries workspace:

```toml
[dependencies]
hyperquant = { path = "../hyperquant" }
```

## Quick start

```rust
use hyperquant::{quantize_a2, quantize_z1, HyperQuantConfig, LatticeKind};

fn main() -> hyperquant::Result<()> {
    let input = [0.125, -0.5, 1.25, 2.0];

    let z1 = quantize_z1(&input, 8.0)?;
    println!("Z1 codes: {:?}", z1.codes);
    println!("Z1 mse: {}", z1.mse);

    let a2 = quantize_a2(&input, 8.0)?;
    println!("A2 codes: {:?}", a2.codes);
    println!("A2 receipt: {:?}", a2.receipt());

    let config = HyperQuantConfig::new(LatticeKind::A2, 8.0);
    let via_config = config.quantize(&input)?;
    assert_eq!(via_config.kind, LatticeKind::A2);

    Ok(())
}
```

## API overview

### `LatticeKind`

```rust
pub enum LatticeKind {
    Z1,
    A2,
    D4,
    E8,
}
```

Current implementation status:

| Lattice | Status | Notes |
|---|---|---|
| `Z1` | implemented | Scalar integer lattice per coordinate |
| `A2` | implemented | Triangular lattice over coordinate pairs |
| `D4` | implemented | Checkerboard lattice over 4D chunks; trailing dims use A2/Z1 |
| `E8` | unsupported | Returns `HyperQuantError::UnsupportedLattice` |

### Rice coding

`hyperquant` includes standalone Rice helpers for local byte accounting:

```rust
use hyperquant::{estimate_best_rice_profile, rice_decode_i16, rice_encode_i16};

let codes = [0, 0, 1, -1, 2, -2];
let profile = estimate_best_rice_profile(&codes, 0..=8)?;
let stream = rice_encode_i16(&codes, profile.k)?;
let decoded = rice_decode_i16(&stream)?;

assert_eq!(decoded, codes);
println!("{} bits/scalar", profile.bits_per_scalar);
```

This is evidence plumbing, not a model-level rate-distortion optimizer. It lets `quant-eval` report actual Rice payload bytes for current HyperQuant code streams.

### RHT helper

`rht_tile` applies deterministic seeded sign flips plus normalized Hadamard transforms over power-of-two tiles:

```rust
use hyperquant::rht_tile;

let mut values = [0.25, -0.5, 1.0, 2.0];
let receipt = rht_tile(&mut values, 4, 123)?;

assert_eq!(receipt.tile_dim, 4);
```

This is a CPU-local primitive helper. It does not claim HyperQuant paper parity or model-quality preservation.

### `quant-codec-core` compat feature

Enable the adapter with:

```toml
[dependencies]
hyperquant = { version = "0.1.1", features = ["compat"] }
```

Inside the Libraries workspace:

```toml
[dependencies]
hyperquant = { path = "../hyperquant", features = ["compat"] }
```

The adapter exposes `HyperQuantCodec` through `quant_codec_core::{CodecProfile, VectorCodec}`. It is intentionally lossy and reports a fixed 16-bit code surface until downstream users opt into Rice payloads explicitly.

### `HyperQuantConfig`

```rust
let config = HyperQuantConfig::new(LatticeKind::A2, 8.0);
let result = config.quantize(&[0.25, 0.5])?;
```

Scale handling is deterministic:

- positive finite scale: used as provided;
- zero, negative, NaN, or infinite scale: normalized to `1.0`;
- the normalized effective scale is recorded in the result and receipt.

### `HyperQuantResult`

A successful quantization returns:

- `kind` — lattice kind used;
- `codes` — integer codes;
- `reconstructed` — lossy reconstructed vector;
- `mse` — mean squared reconstruction error;
- `effective_scale` — normalized positive scale;
- `input_len` — input vector length;
- `input_digest` — BLAKE3 digest of the input values;
- `config_digest` — BLAKE3 digest of lattice kind + effective scale.

### `HyperQuantReceiptV1`

```rust
let result = quantize_z1(&[1.0, 2.0], 4.0)?;
let receipt = result.receipt();

assert_eq!(receipt.input_len, 2);
assert_eq!(receipt.code_len, 2);
```

Receipts are result-bound. They are derived from the stored result metadata, not from arbitrary caller-provided input. This prevents a receipt from accidentally describing a different vector than the one actually quantized.

Receipt fields:

- `kind`
- `input_len`
- `code_len`
- `effective_scale`
- `mse`
- `input_digest`
- `config_digest`
- `claim_boundary`

The current claim boundary is always:

```rust
ClaimBoundary::ExperimentalPrimitiveOnly
```

## Error handling

`hyperquant` uses typed errors:

| Error | Meaning |
|---|---|
| `EmptyInput` | Quantization requires at least one value |
| `NonFiniteInput { index }` | Input contained NaN or ±Inf |
| `UnsupportedLattice(kind)` | Requested lattice is known but not implemented |
| `NonFiniteArtifact { stage }` | A derived artifact such as MSE overflowed or became non-finite |

Example:

```rust
use hyperquant::{quantize_z1, HyperQuantError};

let err = quantize_z1(&[1.0, f32::NAN], 8.0).unwrap_err();
assert_eq!(err, HyperQuantError::NonFiniteInput { index: 1 });
```

## A2 behavior

A2 uses the basis:

```text
b1 = (1, 0)
b2 = (1/2, sqrt(3)/2)
```

For each pair of scaled coordinates, `hyperquant` searches nearby integer basis coordinates and chooses the nearest lattice point. Odd trailing dimensions are not dropped; they fall back to the Z1 scalar rule.

## D4 behavior

D4 uses the checkerboard lattice:

```text
D4 = { x in Z^4 : sum(x_i) is even }
```

For each 4D chunk, `hyperquant` rounds to the nearest integer vector, then repairs parity by adjusting the coordinate with the smallest error penalty when needed. Trailing two dimensions use A2; a single trailing dimension uses Z1. E8 remains explicitly unsupported until a real nearest-lattice implementation lands.

This is a deterministic primitive implementation. It is not yet a full model-level quantization pipeline and does not include rate-distortion allocation.

## Integration path

The intended stack order is:

```text
hyperquant primitive
  -> quant-eval benchmark receipts
  -> quant-codec-core adapter
  -> quant-governor policy/admissibility
  -> turbo-quant / fib-quant comparative backends
  -> poly-kv or semantic-memory only with exact fallback and disclosure
```

Do not wire `hyperquant` directly into truth-bearing stores or runtime search paths without benchmark receipts, exact fallback, and policy gates.

### `quant-eval`

The first integration target is `quant-eval`, which evaluates Z1/A2/D4 fixtures and now uses Rice profile byte accounting for compressed payload estimates before downstream adoption.

### `quant-codec-core`

The `compat` feature exposes HyperQuant as a codec backend through shared traits instead of making downstream crates depend on its internals.

### `quant-governor`

A future policy layer should treat HyperQuant as experimental unless benchmark receipts explicitly admit it for a specific use case.

### `semantic-memory`

If used with semantic-memory, HyperQuant should only compress derived/advisory projections. Canonical evidence, claims, contradiction records, bitemporal truth, source anchors, and exact rerank paths must remain exact/protected.

## Verification

Release gate used for this crate:

```bash
cargo fmt -p hyperquant
cargo test -p hyperquant -- --nocapture
cargo test -p hyperquant --features compat -- --nocapture
cargo check -p hyperquant --all-targets
cargo clippy -p hyperquant --all-targets -- -D warnings
cargo publish -p hyperquant --dry-run --allow-dirty
```

Additional checks performed during crate creation:

- TDD red test first failed on missing public API.
- Independent review found two blockers.
- Blockers fixed:
  - receipts are result-bound rather than arbitrary-input-bound;
  - non-finite artifacts are rejected instead of serialized.
- Final independent review approved.
- Security scan found no hardcoded secrets or obvious shell/eval/SQL injection patterns.

## Development

From the Libraries workspace:

```bash
cargo test -p hyperquant -- --nocapture
cargo test -p hyperquant --features compat -- --nocapture
cargo run -p hyperquant --example receipt_export
cargo clippy -p hyperquant --all-targets -- -D warnings
```

Run the dependent quant-eval harness after changing public behavior:

```bash
cargo test -p quant-eval hyperquant_eval -- --nocapture
```

## Roadmap

Near-term:

- Extend `quant-eval` fixtures with recall-style comparisons.
- Add benchmark receipt JSON fixtures to checked-in docs.
- Add policy/admissibility gates around HyperQuant profiles in `quant-governor`.

Medium-term:

- Implement real E8 nearest-lattice quantization.
- Add rate-distortion allocation experiments only after local fixture coverage exists.

Out of scope for now:

- CUDA/GPU kernels.
- HuggingFace model loading.
- Model-layer quantization pipelines.
- Public superiority claims.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.
