# Phase 02 — Receipts and realized accounting

Fix receipt/accounting defects:

1. Persist compression eval receipts.
2. Replace manifest byte estimate with canonical serialized byte accounting.
3. Track actual active reader scratch bytes, not default scratch multiplied by count.
4. Add decode receipt fields:
   - `full_block_decoded`
   - `decoded_full_values`
   - `returned_values`
   - `copy_performed` where applicable
5. Add ideal-vs-realized byte fields for codecs:
   - `ideal_codec_bits_per_scalar: Option<f32>`
   - `realized_encoded_bytes: u64`
   - `metadata_bytes: u64`

Tests:

- lossy key blocks produce eval receipts;
- manifest bytes equal serialized length under defined canonicalization;
- mixed reader scratch budgets account correctly on attach/drop;
- slice decode receipts disclose full-block decode.

Gate: `cargo test -p poly-kv receipt accounting memory decode` passes.
