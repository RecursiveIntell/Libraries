# Forbidden Final States and Label Policy

## Forbidden labels unless explicitly proven

- `v11A-complete`
- `v11B-complete`
- `v11C-complete`
- `production-cloud-ready`
- `broad-autonomy-ready`
- `canonical-truth-owner`
- `all-issues-fixed`
- `semantic-conformance-proven` unless reference fixture gates pass
- `done-state-receipt-safe` unless no-done-without-receipts fixtures pass

## Allowed likely labels after a successful super pass

Use only if gates pass:

- `v11A-supported-local-hardened-candidate`
- `v11B-minimal-executable-seed`
- `v11C-reserved-only`
- `package-self-replay-passed`
- `hostile-fixture-corpus-present`

## Final status matrix states

Every issue row must end with one of:

- `fixed`
- `quarantined`
- `deferred`
- `superseded`
- `unsupported-by-scope`
- `open-blocking`

Plain `open` is forbidden in final artifacts.

## Claim discipline

A claim is allowed only when its evidence is in the package or explicitly referenced by digest as external/degraded. If a command could not be run, say so and mark the corresponding label unsupported.
