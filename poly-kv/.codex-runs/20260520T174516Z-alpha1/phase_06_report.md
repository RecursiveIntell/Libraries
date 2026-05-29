# Phase 06 Report - Reader Attach, Decode, Memory, Receipts

Status: passed.

Implemented:

- `PoolReader`
- `ReaderConfig`
- `decode_layer`
- `decode_slice`
- `decode_slice_exact_fallback`
- `MemoryAccounting`

Tests:

- reader attach does not duplicate encoded pool bytes
- explicit fallback receipt on fallback decode
- typed rejection for invalid spans and mismatched shapes

Guardrail result:

- Reader state is isolated from shared encoded blocks.
- Shared encoded bytes are counted once.
- Per-reader scratch bytes are counted separately.
- Runtime-specific layouts are rejected by the alpha reader instead of coerced.

Blockers: none.
