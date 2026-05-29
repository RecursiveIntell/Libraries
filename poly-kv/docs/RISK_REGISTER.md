# Risk Register

| ID | Severity | Surface | Failure mode | Fix direction | Acceptance gate |
|---|---|---|---|---|---|
| R01 | S0 | crate architecture | `poly-kv` reimplements TurboQuant/FibQuant math | use traits/adapters only | no local Turbo/Fib algorithm modules |
| R02 | S0 | correctness | lossy compression has no exact fallback | exact fallback mandatory | fallback tests pass |
| R03 | S1 | memory accounting | per-reader pool duplication hidden | shared `Arc`/pool inner; separate scratch bytes | reader-count memory test |
| R04 | S1 | shape handling | layout coerced silently | typed shape/layout errors | malformed shape tests |
| R05 | S1 | public claims | README implies production readiness or reproduced paper metrics | claim boundary checker | no banned claims |
| R06 | S1 | licensing | copied GPL/reference code | clean-room only | license audit checklist |
| R07 | S2 | adapters | feature-gated adapters assume API incorrectly | inspect APIs or stub with compile-time error | adapters off by default |
| R08 | S2 | deterministic replay | digest changes across runs | stable serialization and seeds | deterministic fixture test |
| R09 | S2 | scope creep | Codex builds `quant-governor` too soon | enforce pass boundary | no governor crate in workspace |
| R10 | S2 | hidden runtime authority | codec chooses policy/fallback silently | passive receipts only | no routing engine in codec crates |
