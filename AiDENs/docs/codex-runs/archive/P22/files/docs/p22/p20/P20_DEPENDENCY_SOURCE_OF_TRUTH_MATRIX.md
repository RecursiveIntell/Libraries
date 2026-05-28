# P20 Dependency / Source-of-Truth Matrix

| Use case | Use this | Never do this locally |
|---|---|---|
| Generate or parse stack IDs | `stack-ids` | invent new ID formats |
| Persist memory truth | `semantic-memory` | local SQLite/json memory truth in AiDENs |
| Convert forge export to projection import | `forge-memory-bridge` | adapter reinterpretation |
| Produce/query runtime views | `knowledge-runtime` | hidden runtime database |
| Compile inference graph | `constraint-compiler` | local graph compiler clone |
| Execute kernel/oracle path | `kernel-execution`, `kernel-oracles`, `kernel-conformance` | local witness/syndrome approximation labeled canonical |
| Verify/adjudicate policy | `verification-*` | local verification law clone |
| Provider/tool dispatch | `llm-tool-runtime` plus AiDENs adapter seam | implicit provider capability |
| Surface user-facing advice policy | `aidens-agency-kit` | prompt-only policy |
| Scan repo for P20 violations | `scripts/p20_scan_aidens.py` | manual-only review |
| Final release evidence | `scripts/p20_generate_audit_bundle.sh` | “I think it passed” prose |
