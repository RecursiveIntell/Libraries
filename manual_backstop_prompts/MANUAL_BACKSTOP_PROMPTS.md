# Manual Backstop Prompts

Use these only if hooks or automated guardrails are not active.

## After Phase 0

Revalidate: did you inspect current repo surfaces and write `docs/compression/FIBQUANT_SOURCE_BASIS.md` before code? If not, stop.

## After Phase 2

Revalidate: is the spherical-Beta source implemented as `R² ~ Beta(k/2,(d-k)/2)` and are k=2 radii closed-form tested? If not, stop.

## After Phase 5

Revalidate: did you implement Lloyd-Max refinement and deterministic empty-cell repair? If not, stop.

## After Phase 6

Revalidate: does the codec use fixed-rate indices and reject wrong profile/codebook/index states? If not, stop.

## Final

Revalidate: no `semantic-memory/src/**`, no `turbo-quant/src/**`, no default-on FibQuant, no performance claims. If any fail, do not claim completion.
