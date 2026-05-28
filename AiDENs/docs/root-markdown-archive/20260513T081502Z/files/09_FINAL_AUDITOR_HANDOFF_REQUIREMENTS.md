# Final Auditor Handoff Requirements

The final handoff must include:

1. Exact source package filename and SHA-256.
2. Package report/manifest/findings/excluded/codex-archive sidecars.
3. Extracted-package self-replay command and result.
4. Full command bar commands and logs/digests.
5. Updated issue matrix status counts.
6. Rows still `open-blocking`, if any.
7. Quarantined/deferred/unsupported rows with rationale.
8. Known limitations register.
9. Label policy result.
10. Evidence that no user-visible done state can occur without receipts.
11. Evidence that v11B remains seed-only unless full gates pass.
12. Evidence that v11C remains reserved-only.

## Required summary table

| Gate | Result | Evidence path/digest | Notes |
|---|---|---|---|
| Rust command bar |  |  |  |
| Receipt/done-state |  |  |  |
| Sandbox hostile corpus |  |  |  |
| Patch transactionality |  |  |  |
| Provider honesty |  |  |  |
| Boundary compiler |  |  |  |
| Temporal/proof/view |  |  |  |
| v11B minimal region |  |  |  |
| HNSW/search/pool |  |  |  |
| Unaudited surfaces |  |  |  |
| Final package/replay |  |  |  |
