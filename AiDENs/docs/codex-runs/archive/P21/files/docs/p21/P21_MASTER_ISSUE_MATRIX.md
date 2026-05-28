# P21 Master Issue Matrix

| ID | Priority | Area | Issue | Required outcome | Gate |
|---|---:|---|---|---|---|
| P21-000 | P0 | package | Source/archive can drift from local workspace | Package scanner and archive replay pass | Phase 00/08 |
| P21-001 | P0 | build | Need hard cargo evidence | fmt/check/test/clippy logs in target + handoff | Phase 01 |
| P21-002 | P0 | CLI | Test-agent only exists as integration test | Add `run-test-agent` CLI command | Phase 02 |
| P21-003 | P0 | receipts | Test-agent command must emit receipts/event log | Output bundle with durable receipts | Phase 02 |
| P21-004 | P0 | generator | `aidens new` must produce runnable project | Generated coding agent runs | Phase 03 |
| P21-005 | P1 | profile | Profiles need supported/partial/deferred clarity | `profile list/explain` truthful | Phase 04 |
| P21-006 | P1 | plan | Plan kit must assemble execution plan | plan compile/validate works | Phase 04 |
| P21-007 | P1 | provider | No fake provider support | provider-check matrix truthful | Phase 05 |
| P21-008 | P1 | tools | Tool exposure/permit truth must be inspectable | tools inspect output complete | Phase 05 |
| P21-009 | P1 | agency | Agency gate needs v0.2 eval coverage | expanded evals + runner enforcement | Phase 06 |
| P21-010 | P1 | Recall extraction | Recall/Recall-Coding patterns are not captured | extraction report + templates | Phase 07 |
| P21-011 | P1 | archive | Release zip must be replay-verified | p21 archive script passes | Phase 08 |
| P21-012 | P2 | stretch | First useful coding-agent workflow | repo-search/patch-propose/permit block proof | Phase 09 |
| P21-013 | P2 | daemon | Safe daemon smoke if prior gates pass | no timer storm, queue/safe-mode smoke | Phase 09 |
| P21-014 | P2 | provider | bounded provider expansion if green | OpenAI-compatible chat-only or explicit defer | Phase 09 |
| P21-015 | P0 | audit | Final answer must not fake completion | final hostile audit handoff | Phase 10 |
