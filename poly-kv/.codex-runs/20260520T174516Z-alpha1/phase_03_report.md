# Phase 03 Report - quant-codec-core

Status: passed.

Implemented:

- `CodecId`, `ModelFingerprint`, `TokenizerFingerprint`
- `CodecProfileDigest`, `ArtifactDigest`
- `DType`, `KvRole`, `KvLayout`, `LayerId`, `HeadId`, `TokenSpan`, `KvTensorShape`, `KvSliceRequest`
- `CodecProfile`, `VectorCodec`, `KvCacheCodec`
- `EvalReport`
- typed `QuantCodecError`

Tests:

- shape/span validation
- serde roundtrip
- digest stability
- trait mock compile test

Commands run:

- `cargo test -p quant-codec-core --all-targets`
- `cargo clippy -p quant-codec-core --all-targets -- -D warnings`

Guardrail result:

- Codec/profile/shape/eval types are owned only by `quant-codec-core`.
- No pool state or runtime authority introduced.
- No hidden fallback or shape coercion path introduced.

Blockers: none.
