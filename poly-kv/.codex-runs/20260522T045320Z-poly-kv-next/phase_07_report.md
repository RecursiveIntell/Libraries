# Phase 07 Report

## Scope

Updated public docs and claim boundaries.

## Changed files

- `README.md`
- `crates/poly-kv/README.md`
- `docs/PY_SIDECAR_SPEC.md`
- `docs/BENCHMARK_AND_HARNESS_SPEC.md`
- `docs/NEXT_RELEASE_PLAN.md`
- `docs/BENCHMARK_TIERS.md`
- `docs/CLAIM_BOUNDARY.md`

## Implementation

- Updated status tables for Shape V2, persisted eval receipts, realized accounting, Python sidecar, and harness JSON artifacts.
- Updated README example to use additive `build_from_exact_blocks`.
- Updated Python sidecar spec to match the implemented bulk JSON-compatible sidecar and removed future adapter files from the current layout.
- Added release plan, benchmark tier, and claim-boundary docs.
- Kept benchmark language receipt-oriented and avoided speed, quality, compatibility, or fixed memory-reduction claims.

## Validation

Command: `python3 scripts/check_public_claims.py`

Result: pass.

## Notes

The docs label Python sidecar status as optional/alpha and record that native wheel validation is pending until `maturin` is available.
