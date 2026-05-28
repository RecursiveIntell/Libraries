# 05 — Footgun Test Matrix

Each row should become at least one test.

| ID | Footgun | Required test |
|---|---|---|
| FTG-001 | Parser fallback reported as native | provider route receipt labels fallback explicitly |
| FTG-002 | Disabled tool still callable | disabled tool absent from registry, exposure, and dispatch |
| FTG-003 | Full tool registry exposed by default | exposure planner returns bounded, policy-filtered set |
| FTG-004 | Write tool without approval | write/shell/network tool fails without permit |
| FTG-005 | Raw LLM JSON reaches patch/write tool | boundary gate required before structured tool args |
| FTG-006 | Config secrets leak | redacted config hides tokens/keys/base auth |
| FTG-007 | Config apply half succeeds | validate-then-commit plan/receipt semantics |
| FTG-008 | Runner persists domain truth | runner has receipts only; memory writes use memory adapter |
| FTG-009 | Queue retry creates duplicate job | failover/retry is child attempt in same attempt family |
| FTG-010 | Schedule fires every second after bug | trigger spec requires misfire/overlap/cooldown law |
| FTG-011 | UI owns approval truth | UI adapter only submits/observes permit decisions |
| FTG-012 | Daemon/local split-brain | daemon mode routes runtime mutations through daemon authority |
| FTG-013 | Personal memory contaminates coding profile | coding profile defaults to repo/project memory scope only |
| FTG-014 | Reranker violates temporal scope | temporal filter before ranking; widening receipt if widened |
| FTG-015 | Provider false-ready | capability truth distinguishes configured from healthy/executable |
| FTG-016 | Tool name collision | tool identity includes namespace/name/version |
| FTG-017 | Background work starves interactive work | workload class + budget priority tests |
| FTG-018 | Shell path escape | canonicalized sandbox path tests |
| FTG-019 | Host wake becomes scheduler truth | wake bindings are projections/adapters only |
| FTG-020 | Profile silently grants shell/web/write | profile expansion displays risk summary and requires explicit grant |
