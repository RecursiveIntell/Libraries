# 02 — Phase Plan

## Phase 0 — Source basis and no-code inspection

Inputs:
- Current repo.
- FibQuant paper.
- Existing `semantic-memory` and `turbo-quant` codec surfaces.

Actions:
1. Inspect required files from the master prompt.
2. Create `docs/compression/FIBQUANT_SOURCE_BASIS.md`.
3. Record exact non-goals and source hierarchy.
4. Stop and summarize before coding.

Gate:
- No code before source basis exists.

## Phase 1 — Crate skeleton and profile law

Actions:
1. Add `fib-quant` as workspace member, not default-member.
2. Create profile/error/digest/lib files.
3. Implement `FibQuantProfileV1`.
4. Implement stable digest.

Gate:
- invalid `d`, `k`, `N` reject;
- `d % k != 0` rejects;
- `paper_rate_bits_per_coord != wire_bits_per_coord` can be represented;
- profile digest changes on every math-bearing field.

## Phase 2 — Spherical-Beta source and radii

Actions:
1. Implement Beta parameter helpers.
2. Implement radius quantile.
3. Implement k=2 closed-form radius.
4. Implement canonical sampler and reference Gaussian sampler.

Gate:
- radii monotonic;
- all radii in [0,1];
- empirical E[R²] approximately k/d;
- k=2 closed form matches general quantile within tolerance.

## Phase 3 — Directions

Actions:
1. k=2 planar Fibonacci spiral.
2. k=3 Fibonacci sphere.
3. k>=4 Roberts-Kronecker.

Gate:
- all directions unit-normalized;
- deterministic;
- no NaN/Inf;
- correct method selected by k.

## Phase 4 — Codebook initialization

Actions:
1. Generate radii.
2. Generate directions.
3. Compose row-major codewords.
4. Produce codebook digest.

Gate:
- N × k shape;
- codewords lie in unit ball;
- digest deterministic;
- same profile gives same init.

## Phase 5 — Lloyd-Max refinement

Actions:
1. Generate training samples from spherical-Beta source.
2. Multi-restart Lloyd-Max.
3. Empty-cell repair.
4. Select lowest-MSE restart.
5. Emit `LloydReportV1`.

Gate:
- best MSE <= init MSE or explicit numerical tolerance explanation;
- empty cells repaired or failure emitted;
- deterministic for same seed/profile.

## Phase 6 — Fixed-rate codec

Actions:
1. Encode norm as fp16 by default.
2. Apply rotation.
3. Split into k-blocks.
4. Nearest-codeword index.
5. Pack fixed-rate indices.
6. Decode with digest checks.

Gate:
- corrupt profile rejects;
- corrupt codebook rejects;
- out-of-range index rejects;
- wrong dimension rejects;
- zero-vector behavior explicit.

## Phase 7 — Metrics and tests

Actions:
1. Implement MSE/cosine metrics.
2. Implement fixed deterministic CI test matrix.
3. Add all required tests.

Gate:
- tests pass;
- metrics finite;
- no benchmark claims beyond local test matrix.

## Phase 8 — Receipts and docs

Actions:
1. Implement `FibQuantCompressionReceiptV1`.
2. Add math conformance doc.
3. Add benchmark plan doc.
4. Add rollback plan.

Gate:
- every encode path can emit receipt;
- receipt contains digest chain;
- source-reported claims are separated from local measurements.

## Phase 9 — Final validation and hostile self-audit

Actions:
1. Run validation commands.
2. Run `scripts/fibquant_final_assert.py --repo .` if installed.
3. Review diff.
4. Produce final report.

Gate:
- no forbidden files changed;
- `fib-quant` remains default-off;
- unresolved deviations are explicit.
