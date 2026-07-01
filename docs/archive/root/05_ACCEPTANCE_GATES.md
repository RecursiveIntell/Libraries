# 05 — Acceptance Gates

## Hard pass/fail gates

The pass succeeds only if all are true:

- `fib-quant` crate exists.
- `fib-quant` is in root workspace members.
- `fib-quant` is not in root `default-members` unless explicitly justified.
- `semantic-memory/src/**` unchanged.
- `turbo-quant/src/**` unchanged.
- No product integration.
- No FEUT/SCR variant.
- No performance claims beyond local tests.
- No “zero accuracy loss” phrase in new FibQuant code/docs.
- Lloyd-Max refinement implemented.
- k=2, k=3, and k>=4 direction paths exist.
- arbitrary `N` supported with `ceil(log2(N))` wire bits.
- paper rate and wire rate both surfaced.
- corrupt profile/codebook/index rejects.
- receipts exist.

## Required tests

- `profile_digest.rs`
- `spherical_beta_sampler.rs`
- `paper_k2_radius_closed_form.rs`
- `direction_generators.rs`
- `codebook_determinism.rs`
- `lloyd_refinement.rs`
- `bitpack_indices.rs`
- `encode_decode_roundtrip.rs`
- `corruption_rejection.rs`
- `paper_smoke_regression.rs`

## Test assertions

- Same profile -> same digest.
- Change any math-bearing field -> digest changes.
- Invalid d/k/N rejects.
- `d % k != 0` rejects.
- Radii monotone and bounded.
- Spherical-Beta sampler moment checks pass.
- Directions unit-normalize.
- Codebook deterministic.
- Lloyd best MSE does not worsen init MSE outside declared tolerance.
- Encode/decode output finite.
- Wrong profile/codebook/corrupt payload fails closed.

## Closeout evidence

Final report must include:

- changed files;
- command transcript summary;
- exact test pass/fail/skipped list;
- unresolved mathematical deviations;
- reason for each deviation;
- proof of default-off state;
- proof forbidden files were not modified.
