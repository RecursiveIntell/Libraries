# Hostile Audit Checklist

## Path and dependency hygiene

- [ ] No absolute Cargo path dependencies.
- [ ] TurboQuant is a real dependency or unavailable precondition is reported.
- [ ] semantic-memory does not contain copied TurboQuant math.
- [ ] Existing workspace/package validation still passes or failures are documented.

## TurboQuant hardening

- [ ] Profile contract exists.
- [ ] Profile digest deterministic.
- [ ] Encoded artifact has checksum.
- [ ] Bitpacked QJL signs implemented or honestly deferred.
- [ ] Prepared query scoring exists.
- [ ] Cosine/norm-aware scoring exists.
- [ ] Corruption/profile mismatch tests exist.
- [ ] Byte-size accounting is honest.

## semantic-memory integration

- [ ] VectorCodec abstraction exists.
- [ ] SQ8/raw existing behavior preserved.
- [ ] Turbo adapter feature-gated.
- [ ] Default config inert.
- [ ] Shadow mode non-authoritative.
- [ ] Sidecar storage/eval records include profile digest.
- [ ] Approximate scoring is disclosed.

## Evaluation

- [ ] Evaluation harness compares raw f32/SQ8/Turbo.
- [ ] Records recall@k or top-k agreement.
- [ ] Records bytes/vector.
- [ ] Records latency.
- [ ] Does not claim production superiority without data.

## Search behavior

- [ ] TurboQuant cannot affect ranking unless explicitly enabled.
- [ ] f32 rerank status is visible.
- [ ] fallback/degradation flags are visible.
- [ ] existing search tests pass.

## Final report

- [ ] Commands run are exact.
- [ ] Failed/skipped commands are named.
- [ ] Changed files listed.
- [ ] Rollback instructions included.
- [ ] Next pass recommendations included.
