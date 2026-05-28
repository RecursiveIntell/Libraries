
# PROMPT

You are finishing this repo from the current snapshot.

## Read first

1. `01_EXECUTIVE_SUMMARY.md`
2. `03_HOSTILE_AUDIT.md`
3. `04_CLAUDE_RECONCILIATION.md`
4. `05_MASTER_ISSUE_MATRIX.md`
5. `06_IMPLEMENTATION_PLAN.md`
6. `09_EXACT_FILE_TOUCH_MAP.md`
7. `AGENTS.md`

## Prime directive

Do not make the repo sound cleaner than it is.
Make it cleaner.

## Execution rules

- Work the issue rows in order.
- Start with release truth, not architecture.
- Do not claim green without the named proof command.
- Do not reopen v10+ horizon work to close this finish lane.
- Do not turn curated green checks into full-workspace claims.
- If a shipped script is broken, either fix it or retire it explicitly.
- If a gate is not run by the front door or CI, do not leave it marked green in the receipt.

## First moves

1. Add `04_MASTER_ISSUE_MATRIX.csv`.
2. Fix the archive manifest count.
3. Rewrite the dashboard/evidence manifest/receipt from current repo truth.
4. Align `make gate` with the recorded gate set.
5. Fix the panic-audit/test-layout mismatch.
6. Add CI.
7. Restore or retire the broken v25 surfaces.
8. Finish the remaining v25 production-closure gaps.
