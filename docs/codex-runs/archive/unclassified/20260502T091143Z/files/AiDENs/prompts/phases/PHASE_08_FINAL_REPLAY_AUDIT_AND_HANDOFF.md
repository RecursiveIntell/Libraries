# Phase 08 — Final Replay Audit and Handoff

## Goal
Produce a hostile-auditable final state without making packaging the hero.

## Required final artifacts

- `handoffs/p23/FINAL_AUDIT_REPORT.md`
- `handoffs/p23/KNOWN_LIMITATIONS.md`
- `target/p23/audit/p23_verify.done`
- final package reports for each package role
- package replay report
- capability run report
- support-tier report

## Final audit questions

Answer explicitly:

1. What real AiDENs capability was added?
2. Which `z.py` issues are permanently closed?
3. Which stale-run artifacts remain active and why?
4. Which package roles exist and what do they include/exclude?
5. Can the package replay itself?
6. What cargo gates passed?
7. What remains partial/deferred?
8. What would a hostile auditor attack next?

## Acceptance gate

Final docs must bind to actual emitted artifacts and hashes. Do not hard-code stale package hashes from previous runs.
