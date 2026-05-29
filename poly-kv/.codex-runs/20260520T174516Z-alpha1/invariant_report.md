# Invariant Report

Validated invariants:

- Deterministic profile and artifact digests are implemented in `quant-codec-core`.
- Shape/span validation rejects zero dimensions, empty spans, out-of-range layers, and out-of-range heads.
- `SharedKvPool` stores immutable encoded blocks behind `Arc`.
- Reader state is represented by `PoolReader` and attach receipts, with shared encoded bytes counted once.
- Exact fallback is required for pool construction.
- Fallback decode emits `FallbackReceiptV1`.
- q8 key compression exposes eval data and is tested for finite bounded synthetic drift.
- Raw exact value path roundtrips exactly.
- Runtime-specific layouts are rejected by the alpha reader instead of coerced.
- No TurboQuant or FibQuant math is implemented in `poly-kv`.
- No governor, runtime authority, or app integration was added.

Receipt/schema validation:

- Manifest and build/reader/decode receipts serde-roundtrip in tests.
- `docs/POLY_KV_SCHEMA_PROPOSAL.json` passed `scripts/validate_schemas.py`.

Unsafe code:

- No unsafe code or unsafe tokens found by `scripts/check_forbidden_patterns.py`.
