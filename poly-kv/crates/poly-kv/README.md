# poly-kv

Shared KV-cache pool primitives with typed shapes, per-reader access, explicit exact fallback, quality checks, and receipts.

## Use the checked-out source

This crate lives in the [poly-kv workspace](../../README.md), within RecursiveIntell Libraries. From a separate Rust project's manifest, use the actual path to this checkout:

```toml
[dependencies]
poly-kv = { path = "/path/to/Libraries/poly-kv/crates/poly-kv" }
```

The default feature is `serde`. Enable `fibquant-adapter` explicitly for the FibQuant value codec; it depends on the sibling `Libraries/fib-quant` source. `turbo-quant-adapter` currently exposes an unsupported stub, not a working TurboQuant value codec. Features named `turbo` and `fib` are not defined by this manifest.

## Build a reference pool

```rust
use poly_kv::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shape = KvTensorShape::gqa(
        1, 1, 1, 8, 4,
        KvLayout::LayersHeadsTokensDim,
        DType::F32,
    )?;
    let mut blocks = Vec::new();
    for role in [KvRole::Key, KvRole::Value] {
        let values = vec![0.125; shape.layer_element_count(role)?];
        blocks.push(ExactKvBlock::new(role, LayerId(0), shape.clone(), values)?);
    }

    let pool = SharedKvPool::builder()
        .model_fingerprint(ModelFingerprint::new("example:model")?)
        .tokenizer_fingerprint(TokenizerFingerprint::new("example:tokenizer")?)
        .shape(shape.clone())
        .build_from_exact_blocks(blocks)?;

    let reader = pool.attach_reader(ReaderConfig::default())?;
    let request = KvSliceRequest::layer_span(
        LayerId(0), TokenSpan::new(0, shape.seq_len)?,
    ).for_role(KvRole::Value);
    let decoded = reader.decode_slice_exact_fallback(request)?;
    assert!(decoded.receipt.fallback.is_some());
    Ok(())
}
```

The exact-fallback example establishes the API and ownership boundary. Choose a compression policy and codecs explicitly for compression experiments; inspect their evaluation receipts rather than assuming the compressed representation is smaller or preserves rankings.

## API map

- `PoolBuilder` / `SharedKvPool`: pool construction and shared ownership
- `PoolReader` / `ReaderConfig`: reader access and decode operations
- `Q8KeyCodec` / `RawExactValueCodec`: reference key/value paths
- `ValueCodec`: pluggable value-codec contract
- `KvPoolManifestV1` / `PoolBuildReceiptV1`: pool identity and build accounting
- `DecodeReceiptV1` / `FallbackReceiptV1`: observable decode and fallback behavior
- `KvPoolStore`, `encode_pool_bundle`, and `decode_pool_bundle_with_value_codec`: persistence and reload

## Validation

From `Libraries/poly-kv`:

```bash
cargo test -p poly-kv
cargo test -p poly-kv --features fibquant-adapter
cargo clippy -p poly-kv --all-targets --all-features -- -D warnings
```

The [tests](tests/) cover shape rejection, synthetic roundtrips, codec profiles, receipts, accounting, and persistence. These checks are separate from deployment-specific retrieval, serving, GPU, or multi-tenant isolation validation.

## Licensing

The [package manifest](Cargo.toml) declares `MIT OR Apache-2.0`. Consult the applicable license notices before redistribution.
