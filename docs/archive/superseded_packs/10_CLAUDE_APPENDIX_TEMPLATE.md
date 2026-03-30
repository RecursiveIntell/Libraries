# Claude appendix template

Paste the Claude analysis here when available.

## Merge protocol

1. Classify each Claude finding as:
   - confirmed-live
   - confirmed-live-but-already-covered by an existing `CCS-*` row
   - stale / superseded
   - speculative / insufficiently evidenced
2. Add new `CCS-*` rows only for **confirmed-live** findings not already covered.
3. Add new `REC-*` rows for stale claims worth recording so they do not get re-opened later.
4. Update `02_MASTER_ISSUE_MATRIX.md`, `MASTER_ISSUE_MATRIX.json`, and `MASTER_ISSUE_MATRIX.xlsx` together.
5. Never merge a Claude claim directly into the backlog without a file/path/evidence check against the current repo.
