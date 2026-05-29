# P30 Release Claim Policy

Do not claim what cannot be proven.

Allowed claims:

- `p30-hostile-audit-absorption-partial`: some hostile findings fixed and remaining findings are explicitly ledgered.
- `p30-hostile-audit-absorption-complete`: every hostile finding fixed or quarantined with release-safe debt.
- `v11A-conformant-core`: only if all declared v11A gates pass.
- `v11B-draft-runtime`: only if a tested executable v11B spine exists.
- `v11B-conformant-runtime`: only if the full v11B release bar passes. Do not use this by default.

Forbidden claims:

- packaging-clean equals build-clean;
- build-clean equals semantic-clean;
- v11B structs equal v11B runtime;
- receipts exist equals proof complete;
- proof waiver equals proof;
- advisory/degraded output equals verified truth.
