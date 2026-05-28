# Phase 05 — Receipts, candidate trace, digests, canonical JSON

## Goal

Make the receipt replayable and hostile-auditable.

## Tasks

1. Add `ActionCandidateV1` / `CandidateTraceV1`.
2. Receipt includes:
   - all candidates
   - selected candidate id
   - rejected candidates with exact losing reasons
   - candidate source kind/ref
   - threshold/score/precedence basis
3. Add raw input digest support in CLI:
   - hash raw bytes before parsing,
   - typed canonical input digest after parsing/validation,
   - both appear in receipt when input came from JSON.
4. Replace/rename `evaluator_algorithm_hash`.
   - If it only hashes source entrypoint, call it that.
   - Better: add `EvaluatorBuildDigestV1` covering `scr-reference/src/lib.rs`, `scr-reference/src/policy.rs`, `scr-kernel/src/lib.rs`, `Cargo.lock` when available.
5. Define `scr-canonical-json-v1` in docs.
6. Add tests:
   - policy canonical hash stable under key order changes,
   - policy hash changes under semantic changes,
   - raw digest changes under unknown extra raw fields even if parsing rejects,
   - candidate trace complete.

## Acceptance gate

- A receipt can explain every selected/rejected candidate without recomputing policy.
- Digest fields are honest and documented.
