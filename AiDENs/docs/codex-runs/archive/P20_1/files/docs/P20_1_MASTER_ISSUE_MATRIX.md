# P20.1 Master Issue Matrix

| ID | Priority | Issue | Required fix | Acceptance proof |
|---|---:|---|---|---|
| P20.1-001 | P0 | Missing agency eval fixture | Restore `evals/p20_agency_eval_cases.jsonl`; validate with script and tests | `scripts/p20_validate_agency_cases.py` and cargo tests pass |
| P20.1-002 | P0 | Manifest names missing files | Restore or remove missing manifest entries | manifest check reports zero missing |
| P20.1-003 | P0 | `aidens-testkit` impure topology | Split pure reference testkit from production integration tests | dependency audit shows no reverse dev/normal loops |
| P20.1-004 | P0 | Scanner false confidence | ownership scanner fails when canonical baseline is empty/unavailable | scanner test covers missing canonical crate case |
| P20.1-005 | P0 | Cargo gates not certified | Run full cargo gates in real workspace | final audit includes logs |
| P20.1-006 | P1 | Final archive may omit generated audit outputs | Copy final audit into source-visible handoff or package artifact | final archive integrity script passes |
| P20.1-007 | P1 | Provider truth regression risk | Keep unavailable providers unavailable unless tested | provider matrix tests pass |
| P20.1-008 | P1 | Agency heuristic status | Label/evaluate as heuristic v0.1, not mature classifier | agency eval report generated |
| P20.1-009 | P1 | Scaffold promotion risk | keep scaffold crates deferred or implement minimally | scanner reports no scaffold promotion |
| P20.1-010 | P1 | Code/docs truth drift after repair | update docs only after code passes | docs-code truth report generated |
