# Phase 1 — TurboQuant Codec Hardening

## Goal

Make `turbo-quant` strong enough to be used as a real optional codec dependency.

## Required changes

1. Add codec/profile module:
   - `TurboQuantCodecProfileV1`;
   - profile digest;
   - profile serialization;
   - `RotationKindV1`, `RadiusEncodingV1`, `AngleEncodingV1`, `QjlEncodingV1`, `DistanceMetricV1`.

2. Add encoded artifact module:
   - `EncodedVectorArtifactV1`;
   - checksum/digest;
   - encoded length;
   - profile digest;
   - corruption detection.

3. Add bitpacking:
   - bitpack QJL signs;
   - round-trip tests;
   - keep old `QjlSketch` compatibility if needed but do not pretend `i8` signs are 1-bit storage.

4. Add storage-accounting methods:
   - exact serialized length;
   - theoretical compact length;
   - comparison helpers for raw f32 and current SQ8-like baseline.

5. Add query workspace:
   - precompute query-side rotation/projections once;
   - add prepared inner-product estimate;
   - add cosine estimate or norm-aware score.

6. Harden errors:
   - profile mismatch;
   - dimension mismatch;
   - checksum mismatch;
   - unsupported encoding/profile version;
   - corrupt payload.

7. Tests:
   - deterministic profile digest;
   - encoded artifact round-trip;
   - corruption rejection;
   - profile mismatch rejection;
   - bitpacking round-trip;
   - prepared scoring equals unprepared scoring within tolerance;
   - cosine sanity tests;
   - existing tests still pass.

## Non-goals

- Do not replace dense rotation with SRHT unless feasible without destabilizing the crate.
- Do not claim production compression win unless tests prove byte-size improvement.
