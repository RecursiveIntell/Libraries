# P29 P28 Failure Postmortem

## What P28 did well

P28 appears to have made useful implementation progress:

- modularized `aidens-contracts`;
- modularized `aidens-runner`;
- added v11A-style contract surfaces;
- added adversarial conformance tests;
- improved package hash semantics.

## What P28 failed

P28 failed the evidence/package boundary.

Observed failure class:

1. Archive sidecar current-run mismatch.
2. P28 files classified as stale.
3. `p28_verify.sh` archived/missing.
4. `verify_current.sh` delegated to a missing script.
5. Evidence manifest referenced absent phase reports and target logs.
6. Package validation did not catch broken extracted-package verification.

## Root cause

Codex had no manual phase-gate injection points and was allowed to trust stale classifier state.

## P29 correction

P29 must repair this first.

No feature work may proceed until:

- P29 current-run identity is locked;
- no P29 file is archived as stale;
- `scripts/p29_verify.sh` is active and included;
- `scripts/verify_current.sh` works inside extracted package;
- manifest paths resolve;
- package self-replay is mandatory.
