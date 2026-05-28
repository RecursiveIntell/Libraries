# P20.2 Risk Register

| Risk | Severity | Mitigation |
|---|---|---|
| Works locally but zip omits files | High | release zip unpack/recheck script |
| Testkit remains integration crate | High | purity scanner and split plan |
| Codex deletes tests to pass | High | guardrail injection and coverage equivalence requirement |
| Provider capability overclaim | High | provider matrix and unavailable-by-default rule |
| Agency evals become decorative | High | tests assert outcome/receipts/blocked behavior |
| AiDENs reimplements canonical crates | High | ownership map/scanner/quarantine rule |
| Stretch lane destabilizes v0.1 | Medium | stretch only after core gates; revert on regression |
