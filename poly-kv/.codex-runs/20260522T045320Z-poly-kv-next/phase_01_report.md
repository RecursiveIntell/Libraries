# Phase 01 Report

## Scope

Implemented staged Shape V2 contracts in `quant-codec-core`, preserving the existing alpha `KvTensorShape` API.

## Changed files

- `crates/quant-codec-core/src/shape.rs`
- `crates/quant-codec-core/src/lib.rs`
- `crates/quant-codec-core/tests/shape_validation.rs`

## Implementation

- Added `KvAttentionKind` with `Mha`, `Mqa`, `Gqa`, and `Unsupported(String)`.
- Added `KvCacheShapeV2` with required fields: `batch`, `layers`, `num_q_heads`, `num_kv_heads`, `seq_len`, `head_dim`, `layout`, `dtype`, `attention_kind`.
- Added constructors for `new`, `mha`, `mqa`, and `gqa`.
- Added validation rules:
  - all numeric dimensions must be greater than zero;
  - MHA requires query heads equal to KV heads;
  - MQA requires one KV head and more than one query head;
  - GQA requires query heads greater than KV heads, more than one KV head, and divisibility;
  - unsupported attention fails closed.
- Exported V2 types from `quant-codec-core`.

## Validation

Command: `cargo test -p quant-codec-core shape`

Result: pass. Four filtered shape tests passed, including V2 MHA/MQA/GQA valid cases, invalid attention contracts, and unsupported attention fail-closed behavior.

## Notes

No downstream `poly-kv` API was migrated in this phase. Existing alpha examples and tests keep using `KvTensorShape`.
