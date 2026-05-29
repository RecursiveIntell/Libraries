# Invariant Report

## Source-of-truth ownership

- `quant-codec-core` owns `KvCacheShapeV2`, `KvAttentionKind`, codec traits, profile/digest IDs, dtype, and eval report types.
- `poly-kv` owns shared pool semantics, exact fallback, q8 key reference path, manifests, memory accounting, decode/build/reader receipts, and synthetic tests.
- `poly-kv-python` owns only optional PyO3/Python bulk bindings and wrappers.

## Fallback and lossy operations

- Q8 key compression emits persisted `CompressionEvalReceiptV1` entries through `PoolBuildReceiptV1::compression_evals`.
- Exact fallback remains required in `build_from_blocks`; `build_from_exact_blocks` derives fallback from exact input blocks and delegates to the same build path.
- Fallback decode returns `FallbackReceiptV1` inside `DecodeReceiptV1`.

## Shape and decode behavior

- `KvCacheShapeV2` rejects invalid MHA/MQA/GQA contracts and unsupported attention kinds.
- Decode receipts disclose `full_block_decoded`, `decoded_full_values`, `returned_values`, and `copy_performed`.
- Shape/span mismatch tests pass with typed errors.

## Accounting

- Manifest bytes are calculated by `KvPoolManifestV1::canonical_serialized_len()`.
- Manifest/block/eval receipts include `ideal_codec_bits_per_scalar`, `realized_encoded_bytes`, and `metadata_bytes`.
- Active reader scratch bytes are tracked with `active_reader_scratch_bytes`, including mixed scratch budgets.

## Boundary exclusions

- No TurboQuant or FibQuant math was added.
- No adaptive controller, runtime authority, daemon mode, serving runtime adapter, semantic-memory, Gloss, Recall, AiDENs, or ClaimLedger integration was added.
- Rust core crates do not depend on PyO3, maturin, NumPy, or torch.
