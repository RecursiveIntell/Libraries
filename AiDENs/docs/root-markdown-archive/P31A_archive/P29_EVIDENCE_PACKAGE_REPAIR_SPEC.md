# P29 Evidence and Package Repair Spec

## Required agreement

These must all agree on P29:

- `docs/codex-runs/CURRENT_RUN.md`;
- `STATUS.md`;
- `SOURCE_BASIS.md`;
- `SUPPORT_PROFILE.md`;
- `P29_STATUS_EVIDENCE_MANIFEST.json`;
- codex archive sidecar;
- package report;
- final auditor handoff.

## Required package behavior

The final package must include:

- `scripts/p29_verify.sh`;
- `scripts/verify_current.sh`;
- `P29_STATUS_EVIDENCE_MANIFEST.json`;
- final audit report;
- known limitations register;
- support traceability report;
- phase reports or manifest-accessible evidence.

## Forbidden package behavior

- current-run files archived as stale;
- verifier missing from package;
- target logs referenced but excluded without external/degraded label;
- package passes validation without extracted verifier replay.
