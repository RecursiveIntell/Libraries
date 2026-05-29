# Phase 02 Report

## Scope

Hardened receipts and realized accounting in `poly-kv`.

## Changed files

- `crates/poly-kv/src/codecs/q8_keys.rs`
- `crates/poly-kv/src/manifest.rs`
- `crates/poly-kv/src/memory.rs`
- `crates/poly-kv/src/pool.rs`
- `crates/poly-kv/src/reader.rs`
- `crates/poly-kv/src/receipts.rs`
- `crates/poly-kv/tests/memory_accounting.rs`
- `crates/poly-kv/tests/receipt_roundtrip.rs`
- `crates/poly-kv/tests/synthetic_roundtrip.rs`
- `scripts/assert_realized_accounting.py`

## Implementation

- Persisted `CompressionEvalReceiptV1` values in `PoolBuildReceiptV1::compression_evals`.
- Added realized accounting fields to block manifests and compression eval receipts:
  - `ideal_codec_bits_per_scalar`
  - `realized_encoded_bytes`
  - `metadata_bytes`
- Replaced fixed manifest byte estimates with `KvPoolManifestV1::canonical_serialized_len()`.
- Added `active_reader_scratch_bytes` tracking on reader attach/drop and used it in `SharedKvPool::memory_accounting()`.
- Extended `DecodeReceiptV1` with:
  - `full_block_decoded`
  - `decoded_full_values`
  - `returned_values`
  - `copy_performed`
- Repaired `scripts/assert_realized_accounting.py`; it had a baseline unterminated newline literal and could not run.

## Validation

Commands and results:

- `cargo fmt --all`: pass
- `cargo test -p poly-kv receipt`: pass
- `cargo test -p poly-kv memory`: pass
- `cargo test -p poly-kv decode`: pass
- `cargo test -p poly-kv accounting`: pass, with matching `memory` tests because of test-name filtering
- `cargo test -p poly-kv`: pass, 11 tests
- `python3 scripts/assert_receipt_integrity.py`: pass
- first `python3 scripts/assert_realized_accounting.py`: failed with `SyntaxError` in the baseline script
- second `python3 scripts/assert_realized_accounting.py`: pass after script repair

## Notes

`copy_performed` is currently `true` for decode receipts because the alpha reader decodes to owned `Vec<f32>` and slice extraction returns an owned copy. No zero-copy claim is made.
