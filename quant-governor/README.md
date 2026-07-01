# quant-governor

Governance policy routing for governed compression.

`quant-governor` decides which compression codec to use for a given
content payload, given the content type, size, accuracy requirements,
and the caller's policy. The decision is deterministic, receipted, and auditable.
Every routing decision can produce:
- a `DegradationReceipt` (when fidelity is reduced)
- an `ExactFallbackReceipt` (when the policy had to fall back to raw)
- a `GovernanceDecisionReceipt` (explicit policy rationale, admissibility blocks,
  and allow/block signals for HyperQuant rollout)

This is the policy layer for the RecursiveIntell compression stack — every
call to `turbo-quant`, `fib-quant`, or `gpu-backend` is routed through
`quant-governor` first.

## Why a governance layer?

In a governed system, **"we compressed your data"** is not acceptable.
The caller needs to know:

- which codec was used (Raw, Q8, Q4, Turbo, Fib, Hyperquant, Polar)
- whether the policy had to fall back to a lower-fidelity codec
- what the estimated accuracy impact is
- whether the original was retained as an exact fallback

`quant-governor` answers all of this with a single `CodecDecision` and attached
receipts. The decision is **deterministic** — same `GovernanceRequest` + same
`GovernancePolicy` always yields the same `CodecDecision`. This makes the routing
auditable and replayable.

## Quick Start

```rust
use quant_governor::{
    AdmissibilityClass, ContentType, evaluate, GovernancePolicy, GovernanceRequest,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a request describing the payload.
    let request = GovernanceRequest {
        content_type: ContentType::Embedding,
        size_bytes: 4_096,
        accuracy_requirement: 0.95,
        latency_tolerance_ms: 50,
        admissibility: AdmissibilityClass::Standard,
    };

    // Evaluate against a policy.
    let policy = GovernancePolicy::default();
    let decision = evaluate(request, &policy)?;

    // The decision carries the selected codec, the degradation
    // budget, and a receipt.
    println!("Selected codec: {:?}", decision.codec);
    println!("Degradation budget: {}", decision.degradation_budget);
    println!("Receipt: {:?}", decision.receipt);
    Ok(())
}
```

Run it: `cargo run --example basic_policy`.

## Codec profiles

| Profile | Description | Use case |
|---|---|---|
| `Raw`   | Uncompressed | Critical accuracy, small payloads |
| `Q8`    | 8-bit scalar quantization | Balanced storage / accuracy |
| `Q4`    | 4-bit scalar quantization | Storage constrained |
| `Turbo` | TurboQuant (symmetric, reconstructs the original) | Low latency, batched |
| `Fib`   | Fibonacci-weighted (symmetric, reconstructs the original) | Precision sensitive |
| `Hyperquant` | Lattice VQ (reconstructable) | Large embeddings with controlled error |
| `Polar` | Polar-only (asymmetric, only `score_inner_product` / `score_l2` work) | Search-only |

`Turbo`, `Fib`, and `Hyperquant` are the primary high-throughput lossy
options for embeddings and vector-like payloads. `Turbo` is the lowest-latency
option (no decode needed — scoring is done in the encoded space). `Fib` is the
most accurate. `Hyperquant` is reconstructable and intended for governed rollouts
where lattice quantization receipts are available. `Polar` is the search-only
option (you can never decode, you can only score).

## Admissibility classes

`AdmissibilityClass` is the caller's stated need, in priority order:

| Class | Behavior |
|---|---|
| `Critical`   | Always routes to `Raw`. No degradation. |
| `HighPriority` | Routes to `Q8` minimum. Degradation budget ≤ 0.05. |
| `Standard` (default) | Routes to `Q8` or `Q4` depending on size. Degradation budget ≤ 0.20. |
| `Compressible` | Routes to `Q4` or `Turbo`. Degradation budget ≤ 0.40. |
| `BestEffort` | Routes to `Turbo` or `Polar`. Degradation budget ≤ 0.80. |

The `default` `GovernancePolicy` is conservative — it routes
`Standard` to `Q8` and bumps to `Q4` only for payloads > 1 MB.

## Receipts

`quant-governor` emits three receipt types that downstream audit and rollback
layers can act on:

### `ExactFallbackReceipt`

Emitted when the policy selects a lossy codec but the caller supplied
`admissibility = Critical` (or the `accuracy_requirement` was above the policy's
`raw_min_accuracy`). The receipt carries:

- `raw_digest: Digest` — BLAKE3 hash of the original payload.
- `compressed_digest: Digest` — BLAKE3 hash of the compressed payload.
- `fallback_retention: bool` — whether the system kept the
  compressed form alongside the raw.
- `fallback_reason: Option<String>` — the policy reason string.

### `DegradationReceipt`

Emitted when the policy downgraded from a higher-fidelity codec to a lower-fidelity
one (for example, `Q8` → `Q4` because the payload exceeded a size threshold).
The receipt carries:

- `degradation_type: DegradationType`
- `degraded_by: f64` — the actual degradation applied (0.0 to 1.0).
- `bytes_saved: u64` — estimated bytes saved by the degradation.
- `accuracy_impact: f64` — estimated accuracy impact (0.0 to 1.0, higher = more impact).

### `GovernanceDecisionReceipt`

Emitted when a routing decision carries policy rationale needed for explicit audit,
for example when HyperQuant is allowed or blocked by governance. The receipt carries:

- `policy_name: String` — policy variant used for routing.
- `content_type: String` — content type selected in that decision.
- `admissibility: String` — admissibility class from the request.
- `rationale: String` — branch-level policy reason string.
- `blocked_profiles: Vec<CodecProfile>` — profiles rejected by current policy constraints.
- `candidate_profiles: Vec<CodecProfile>` — codec candidates evaluated in the branch.

These receipts are the audit handle. A system that does governed compression should
persist them and expose them to the caller.

## What this crate is not

- **Not a codec.** It does not encode or decode anything. It tells
  you which codec to use and emits a receipt. The actual codec
  implementation lives in `turbo-quant`, `fib-quant`,
  `gpu-backend`, or `quant-codec-core`.
- **Not a transport.** It does not move bytes. It produces a
  routing decision as data.
- **Not a scheduling system.** It does not decide *when* to
  compress; it decides *which* codec.

## Test coverage

- **12 integration tests** in `tests/` covering:
  - Policy evaluation determinism (same input → same output)
  - Profile behavior (digest stability and routing under stable policy)
  - Admissibility class transitions (Critical → Raw, etc.)
  - Fallback receipt generation (lossy → raw when `admissibility` is Critical)
  - Degradation triggers (size threshold crossing, accuracy requirement increase, latency
    budget shrink)
  - ContentType routing (Text, Image, Audio, Video, Structured, Embedding, Model, Other)
- **3 doctests** in the lib.rs doc-comment.
- **1 example**: `examples/basic_policy.rs`.
- `cargo test` clean, `cargo clippy --all-targets -- -D warnings` clean.

## Performance

`evaluate()` is O(1) — it inspects the request and policy, applies
routing rules, and produces a decision. No loops over the payload (the
payload isn't even passed in).

The receipt construction is also O(1) — BLAKE3 over fixed-size struct fields,
no I/O.

## MSRV

Rust 1.75 (2021 edition). Stable-only features.

## Dependencies

- `serde`, `thiserror`, `chrono`, `sha2`, `blake3`.
- `tokio` (dev only, for the integration tests).
- `lib` profile specifies `[profile.release]` with
  `opt-level = 3, lto = true, codegen-units = 1`.

Zero platform-specific code. Zero FFI. Zero async runtime in the
production crate (the routing function is sync).

## License

MIT. See `LICENSE-MIT` for the full text.

## Changelog

See `CHANGELOG.md` for the release history.

## Where it's used

`quant-governor` is the policy layer for:

- [`turbo-quant`](../turbo-quant) — the experimental
  vector compression sidecar (TurboQuant, PolarQuant, QJL).
- [`semantic-memory`](../semantic-memory) — every
  projection import routes through `evaluate()` to decide which sidecar to use.
- [`scr-runtime-compression`](../scr-runtime-compression) — the
  cross-runtime compression scheduler uses `quant-governor` to pick the right
  codec per payload.

Any system that does governed compression (legal/medical/audit
contexts where "we compressed your data" must be a typed, receipted
operation) can adopt `quant-governor` directly.