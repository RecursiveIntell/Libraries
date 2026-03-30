
# 08_RELEASE_AND_GOVERNANCE

## Release authority rules

There must be exactly one active release-truth story at the root.

That story must be the same across:
- `README.md`
- `PACK_README.md`
- `STATUS_DASHBOARD.md`
- `MASTER_ISSUE_MATRIX.md`
- `SUPPORT_PROFILE.md`
- `release/closeout_receipt_v1.json`

## Evidence rules

- The dashboard is not allowed to outrun the scripts.
- The receipt is not allowed to outrun the evidence manifest.
- The evidence manifest is not allowed to outrun the current repo state.
- Historically true but currently irreproducible claims must be labeled historical.

## Gate rules

- `make gate` is the front door.
- If the receipt lists a gate, `make gate` or the explicitly named CI workflow must run it.
- If a gate is not run anywhere, it cannot be marked green in the release story.

## Support-lane rules

- The support lane must remain narrow while truth is being repaired.
- Adjacent crates can be documented as satellites, but not silently upgraded into the build-certified claim.

## Forbidden shortcuts

- Do not silence failing scripts by deleting them without updating the release story.
- Do not “fix” the dashboard before the underlying repo truth is fixed.
- Do not hide thin shells behind curated metrics without saying the metrics are curated.
- Do not claim production closure if `.github/workflows/ci.yml` is missing.
