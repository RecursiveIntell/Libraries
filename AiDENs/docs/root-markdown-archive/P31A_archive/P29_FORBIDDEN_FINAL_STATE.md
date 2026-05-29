# P29 Forbidden Final State

The pass fails if any of these are true:

1. Current run identity is not P29 everywhere.
2. Any active P29 file is archived as stale.
3. `scripts/p29_verify.sh` is missing from the final package.
4. `scripts/verify_current.sh` fails inside extracted package.
5. Status evidence manifest references missing files without explicit external/degraded labels.
6. Final package cannot self-replay.
7. v11A local release-candidate is claimed without material-operation receipts.
8. v11B-complete is claimed.
9. v11C-complete is claimed.
10. AiDENs claims canonical sibling truth ownership.
11. Critical audit bugs are ignored without quarantine.
12. Manual gate reports are missing at required stops.
