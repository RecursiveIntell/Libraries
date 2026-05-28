# Master Issue Matrix — Next Run

| ID | Issue | Severity | Required fix | Acceptance |
|---|---|---:|---|---|
| AIDENS-NEXT-001 | Runner fake-completes | Critical | provider trait + disabled/mock behavior; remove placeholders | `assert_no_fake_completion.sh`, runner test |
| AIDENS-NEXT-002 | Provider-kit route truth but no execution | Critical | executable provider boundary | disabled fails, mock answers, optional ollama |
| AIDENS-NEXT-003 | Tool-kit descriptor only | High | repo-read dispatcher + receipts | tool dispatch test |
| AIDENS-NEXT-004 | CLI lacks plan compiler flow | High | profile/plan/doctor/run commands | CLI test, smoke |
| AIDENS-NEXT-005 | Generated app too scaffold-like | High | facade-only generated app | app-kit test |
| AIDENS-NEXT-006 | Advanced crates might look healthy | Medium | doctor reports disabled/deferred | doctor output check |
| AIDENS-NEXT-007 | Mock can become accidental production | Medium | mock explicit only | config/profile gates |
| AIDENS-NEXT-008 | Raw paths can escape sandbox | High | path validation before repo-read | traversal rejection |
| AIDENS-NEXT-009 | No exact pass evidence | Medium | update PASS_STATUS with commands | PASS_STATUS complete |
