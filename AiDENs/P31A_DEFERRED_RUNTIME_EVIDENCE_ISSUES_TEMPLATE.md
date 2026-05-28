# P31A Deferred Runtime Evidence Issues for P31B

This file records runtime issues found during P31A that must not be fixed in P31A.

| ID | Issue | Evidence | Risk | P31B fix direction | Acceptance test |
|---|---|---|---|---|---|
| P31B-001 | Process-local/display-only IDs may enter durable receipts |  | Durable replay/evidence identity drift | Inspect stack-ids and aidens-contracts; reuse canonical IDs; add compiler-enforced type split | deterministic replay IDs/digests test |
| P31B-002 | Provider retries represented as warnings instead of attempt-family receipts |  | retry lineage hidden | use existing canonical receipt/effect envelope; do not invent duplicate receipt families | two failures then success emits three attempt records |
| P31B-003 | Blocked tool paths may lack control receipts |  | blocked execution hidden | route through existing control/operator receipt surface | unexposed/denied/recursive tool calls emit control evidence |
| P31B-004 | Patch rollback/durability claims may be too broad |  | false no-write/no-damage claim | receipt outcome taxonomy; rollback-failed quarantine | simulated rollback failure cannot say no files written |
| P31B-005 | Repo search skipped-file evidence incomplete |  | search result completeness false claim | skipped-file records | unreadable/non-UTF8/denied files reported |
