# Phase 2 — semantic-memory VectorCodec Abstraction

## Goal

Create a clean internal codec boundary without changing default behavior.

## Required changes

1. Add vector codec module:
   - `VectorCodec` trait;
   - `EncodedVector`;
   - `QueryState`;
   - `CodecScore`;
   - `VectorScoreProvenance`;
   - `ApproximationClass`;
   - error variants.

2. Implement current behavior:
   - `RawF32Codec` for exact baseline;
   - `Sq8Codec` wrapping existing quantization behavior.

3. Add config:
   - `VectorCodecConfig` nested under current config where appropriate;
   - defaults preserve current behavior exactly.

4. Add tests:
   - SQ8 existing tests pass;
   - raw codec exact cosine/inner product sanity;
   - default config produces no TurboQuant side effects;
   - serialization/serde tests if public config changes.

## Required discipline

- Do not add TurboQuant dependency in this phase unless Phase 0 confirmed layout.
- Do not break existing `SearchResult` users. Prefer optional metadata or parallel explained APIs.
