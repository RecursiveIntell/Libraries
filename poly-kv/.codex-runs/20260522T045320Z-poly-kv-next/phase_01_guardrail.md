# Phase 01 Boundary Guardrail

Guardrail sources:

- `codex/manual-injections/PHASE_BOUNDARY_GUARDRAIL.md`
- `codex/manual_injections/AFTER_PHASE_01.md`

## Revalidation

1. Source-of-truth ownership: shape contracts remain owned by `quant-codec-core` in `crates/quant-codec-core/src/shape.rs`.
2. Duplicate abstraction or shadow implementation: no `poly-kv` shape copy was introduced; V2 shape is canonical in `quant-codec-core`.
3. Silent schema widening, coercion, hidden fallback, fake compatibility: V2 is additive and explicit; unsupported attention returns `QuantCodecError::InvalidShape`.
4. Material operations and receipts: no pool/compression/decode operation changed in Phase 01; receipts not applicable.
5. Exact fallback: unchanged in `crates/poly-kv/src/codecs/raw_exact.rs`.
6. Optional adapters: unchanged; no external compatibility claim added.
7. Tests/fixtures/assertions: `crates/quant-codec-core/tests/shape_validation.rs` covers valid MHA/MQA/GQA, invalid contracts, and unsupported fail-closed.
8. Failed/skipped validation: none for Phase 01; `cargo test -p quant-codec-core shape` passed.
9. Scope exclusions: no governor, runtime authority, semantic-memory, Gloss, Recall, AiDENs, or ClaimLedger integration introduced.
10. Rollback path: revert `crates/quant-codec-core/src/shape.rs`, `crates/quant-codec-core/src/lib.rs`, and `crates/quant-codec-core/tests/shape_validation.rs` to remove Phase 01.

Decision: proceed to Phase 02.
