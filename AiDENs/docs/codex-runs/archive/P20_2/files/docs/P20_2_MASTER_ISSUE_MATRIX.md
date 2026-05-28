# P20.2 Master Issue Matrix

| ID | Phase | Priority | Issue | Required outcome | Proof |
|---|---:|---|---|---|---|
| P20.2-000 | 00 | P0 | Code/source truth preflight | Current actual code state inventoried; no doc-derived assumptions | `target/aidens-p20-2-audit/source_truth_report.md` |
| P20.2-001 | 01 | P0 | Missing agency eval file | `evals/p20_agency_eval_cases.jsonl` restored and validated | validator log |
| P20.2-002 | 01 | P0 | Missing include targets | All literal `include_str!`/`include_bytes!` targets exist | package scanner JSON |
| P20.2-003 | 01 | P0 | Manifest/archive inconsistency | manifest files corrected or files restored | scanner + release zip recheck |
| P20.2-004 | 02 | P0 | `aidens-testkit` impurity | testkit is pure/reference-only | purity scanner |
| P20.2-005 | 02 | P0 | Missing integration-test crate | production-dependent vertical tests moved to `aidens-integration-tests` | workspace + cargo test |
| P20.2-006 | 03 | P0 | Build gate uncertainty | fmt/check/test/clippy pass in real workspace | command logs |
| P20.2-007 | 04 | P1 | No canonical test-agent proof | deterministic test agent runs end-to-end | integration test + receipts |
| P20.2-008 | 05 | P1 | Provider/tool/permit receipt hardening | provider honesty and tool flow proved | test logs + capability matrix |
| P20.2-009 | 06 | P1 | Agency eval depth | evals cover high-impact, personalization, repeated nudges, tool urgency, delegated influence, relational boundary, manipulation, sycophancy | eval report |
| P20.2-010 | 07 | P1 | Operator examples | basic/coding/memory examples exist and are honest | smoke output |
| P20.2-011 | 08 | P1 | Scanner/conformance hardening | scanner prevents missing-package and shadow-ownership regressions | script output |
| P20.2-012 | 09 | P0 | Release certification | audit bundle and archive replay pass | release audit report |
| P20.2-013 | 10 | P2 | Guarded stretch lane | profile smoke / P21 plan only if all previous gates pass | stretch report |
