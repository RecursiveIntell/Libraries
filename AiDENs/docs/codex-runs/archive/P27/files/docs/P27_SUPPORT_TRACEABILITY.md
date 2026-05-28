# P27 Support Traceability

Record date: `2026-05-05`

This document maps P27 support claims to executable evidence. It is a local operator traceability surface, not a canonical truth source.

| Claim | Tier | Evidence | Semantic status |
|---|---|---|---|
| Current verifier wrappers and CI target real P27/current entrypoints. | supported-local | `handoffs/p27/PHASE_01_REPORT.md`; `target/p27/audit/verify_current_phase17_skip_cargo.log` | exact check with `P27_SKIP_CARGO=1` degradation |
| Active run docs agree on P27. | supported-local | `handoffs/p27/PHASE_02_REPORT.md`; `target/p27/audit/assert_p27_current_run_truth.log`; `target/p27/audit/assert_p27_agents_md_current.log` | exact static check |
| Package self-replay is honest about sibling prerequisites. | partial/degraded when cargo is skipped or siblings are absent | `handoffs/p27/PHASE_04_REPORT.md`; `SOURCE_BASIS.md`; `target/p27/audit/assert_sibling_workspace_layout_receipt.json` | exact environment classification |
| Ownership scanner fails closed without canonical baseline. | supported-local guard | `handoffs/p27/PHASE_04_REPORT.md`; `target/p27/audit/assert_p27_ownership_scanner_fail_closed.log` | exact static/fixture check |
| Scaffold profile crates do not inflate support. | supported-local guard | `handoffs/p27/PHASE_06_REPORT.md`; `target/p27/audit/assert_p27_no_scaffold_profile_inflation.log` | exact static check |
| Mock-provider Plan→Act→Verify E2E runs without cloud credentials. | supported-local / fixture-backed, not cloud | `handoffs/p27/PHASE_09_REPORT.md`; `target/p27/audit/cargo_test_integration_phase17_provider_e2e.log` | exact fixture-backed run |
| Optional Ollama path is skip-classified unless local service is available. | partial-local-chat | `handoffs/p27/PHASE_09_REPORT.md` | degraded/optional environment classification |
| Durable run receipts survive process exit and inspect from receipt root. | supported-local | `handoffs/p27/PHASE_08_REPORT.md`; `target/p27/audit/cargo_test_integration_phase17_run_bundle_store.log` | exact fixture-backed run |
| Patch apply/check path preflights and fails closed on ambiguity. | supported-local v0 | `handoffs/p27/PHASE_10_REPORT.md`; `handoffs/p27/PHASE_11_REPORT.md`; `target/p27/audit/cargo_test_aidens_cli_phase17_full.log` | exact local tests |
| Coding-agent path links read/search/propose/apply/check receipts. | supported-local v0 | `handoffs/p27/PHASE_11_REPORT.md`; `target/p27/audit/cargo_test_aidens_cli_phase17_full.log` | exact local tests |
| Memory grounding stays on canonical adapter/backpointer route. | partial canonical-seam evidence | `handoffs/p27/PHASE_12_REPORT.md`; `target/p27/audit/assert_p27_memory_no_local_truth.log` | exact guard plus partial runtime boundary |
| Evidence-bearing CLI reports expose exactness/support/degradation/proof labels. | supported-local disclosure layer | `handoffs/p27/PHASE_17_REPORT.md`; `target/p27/audit/assert_p27_semantic_disclosure_phase17.log` | exact static check |

## Deferred Claims

No P27 evidence promotes these surfaces:

- hosted/cloud provider execution;
- native provider tool loops over hosted APIs;
- production streaming loops;
- broad autonomous daemon scheduling;
- V10 regional runtime geometry;
- V11 full proof-governed runtime/reference interpreter;
- V12/federated/mechanism runtime.
