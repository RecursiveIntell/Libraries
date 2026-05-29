# poly-kv

Rust research-to-implementation workspace for shared KV-cache pool experiments.

This repository contains:

- `quant-codec-core`: shared codec IDs, profile digests, KV shape/layout types, codec traits, and eval reports.
- `poly-kv`: typed manifests and receipts for shared KV-pool experiments, exact fallback, a q8 key reference path, raw exact value storage, reader memory accounting, and decode receipts.
- `poly-kv-python`: optional PyO3 sidecar skeleton for bulk JSON-compatible receipt experiments.

This workspace is an alpha implementation target, not a serving runtime.

## Status

| Capability | Status |
|---|---|
| shared pool manifests | implemented / tested |
| exact fallback | implemented / tested |
| q8 key reference path | implemented / tested on synthetic fixtures |
| raw exact value codec | implemented / tested |
| Shape V2 contracts | implemented / tested in `quant-codec-core` |
| persisted compression eval receipts | implemented / tested |
| realized byte accounting | implemented / tested on synthetic fixtures |
| Python sidecar | alpha / optional / native build not required for Rust core |
| Tier 0/Tier 1 harness JSON | implemented; receipts stored under `.codex-runs/` |
| TurboQuant value adapter | optional / experimental / unsupported stub until API inspection |
| FibQuant value adapter | optional / experimental / unsupported stub until API inspection |
| real model benchmarks | not yet reproduced |
| serving runtime integration | not implemented |
| adaptive controller | deferred |

## Scope Boundary

`poly-kv` owns shared pool semantics: immutable encoded blocks, exact fallback references, reader attach/decode receipts, memory accounting, and synthetic validation.

`quant-codec-core` owns codec/profile/shape/eval types shared by compression crates.

This pass does not implement `quant-governor`, `scr-runtime-compression`, semantic-memory adapters, Gloss/Recall/AiDENs/ClaimLedger integrations, CUDA kernels, or runtime-specific serving adapters.

## Safety Model

The alpha implementation rejects shape and span mismatches with typed errors. Lossy key compression is represented by an explicit q8 codec profile, quality/eval receipt data, and an exact fallback path. If exact fallback decoding is requested, the decode receipt includes a `FallbackReceiptV1`.

Decode receipts disclose whether the alpha reader decoded a full block, how many values were decoded and returned, and whether an owned copy was produced.

No unsafe Rust is used by default.

## Synthetic Example

```rust
use poly_kv::*;

let shape = KvTensorShape::gqa(
    2,
    2,
    2,
    8,
    4,
    KvLayout::LayersHeadsTokensDim,
    DType::F32,
)?;

let blocks: Vec<ExactKvBlock> = /* synthetic or exported KV blocks */;

let pool = SharedKvPool::builder()
    .model_fingerprint(ModelFingerprint::new("synthetic:test-model")?)
    .tokenizer_fingerprint(TokenizerFingerprint::new("synthetic:test-tokenizer")?)
    .shape(shape)
    .policy(CompressionPolicyV1::alpha_reference())
    .key_codec(Q8KeyCodec::symmetric_per_block())
    .value_codec(RawExactValueCodec)
    .build_from_exact_blocks(blocks)?;

let reader = pool.attach_reader(ReaderConfig::default())?;
let _slice = reader.decode_slice(
    KvSliceRequest::layer_span(LayerId(0), TokenSpan::new(0, 4)?).for_role(KvRole::Key),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Validation

Primary local gates:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
python3 scripts/validate_schemas.py
python3 scripts/check_public_claims.py
python3 scripts/validate_final_state.py
python3 scripts/bench_rust_synthetic.py --run-id "$RUN_ID"
python3 scripts/bench_boundary.py --run-id "$RUN_ID"
python3 scripts/compare_receipts.py --run-id "$RUN_ID"
```

## Attribution

This crate is an independent Rust research-to-implementation effort inspired by PolyKV-style shared compressed KV-cache pool ideas. It is not the original authors' reference implementation and does not claim affiliation with the PolyKV paper authors.
