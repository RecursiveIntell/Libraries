# P20 Risk Register

| Risk | Severity | Mitigation | Gate |
|---|---:|---|---|
| `aidens-contracts` becomes shadow truth | Critical | ownership inventory + scanner | Phase 03/04 |
| docs overclaim implementation | Critical | docs truth rewrite + scanner | Phase 02 |
| provider capabilities lie | High | capability matrix + tests | Phase 05 |
| runner lacks vertical slice | High | fixture E2E test | Phase 06 |
| canonical crates not actually used | Critical | adapter proof tests | Phase 07 |
| agency/influence governance coverage remains scoped | Medium | agency kit + evals + runner gate tests | Phase 08 passed; broaden in later product work |
| reference semantics drift from docs | Mitigated | Phase 09 implements temporal reference behavior and hostile tests; unsupported surfaces remain demoted | Phase 09 passed |
| compatibility shims reappear | High | scanner + deletion rules | Phase 04 |
| build failures hidden by docs edits | Critical | cargo gate first | Phase 01 |
| final audit missing | Critical | audit script | Phase 10 |
