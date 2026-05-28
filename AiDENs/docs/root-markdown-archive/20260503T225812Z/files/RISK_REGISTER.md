# Risk Register

| Risk | Impact | Detection gate | Mitigation |
|---|---|---|---|
| Codex preserves duplicate semantics | P0 duplicates remain public and production paths use them. | assert_no_shadow_truth; SHADOW_SEMANTICS_AUDIT gates | Phase 1 contract collapse before features. |
| Codex uses Libraries2/stack-ids | Split identity spaces. | assert_stack_paths fails on Libraries2 stack-ids path | Hard path rule; never point stack-ids to Libraries2. |
| Compatibility shims become permanent | Shim turns into architecture. | assert_compat_is_finite | Every shim has owner/removal/test. |
| Daemon/scheduler implemented early | Queue becomes shadow control/truth store. | Phase manifest stop condition | Block phase 7 until phases 03 and 06 pass. |
| Kernel features before spine | Kernel receipts/convergence become local fiction. | Phase manifest stop condition; kernel tests absent before phase 8 | Block phase 8 until vertical slice/governance. |
| Docs drift from Cargo reality | Agent receives stale instructions. | assert_docs_match_cargo | Update docs in same PR as manifests. |
| Tests verify mocks instead of canonical crates | False green tests. | adapter_delegation_proof; golden_vertical_slice requirements | Mocks may fake I/O only, not stack boundaries. |
| Vertical slice is faked | AiDENs remains AiDENs-shaped Rust. | golden_vertical_slice checks canonical crate calls | Require actual stack APIs or stop if APIs unknown. |
| Local receipts diverge | Execution evidence becomes noncanonical. | budget_exhaustion_receipt; adapter_delegation_proof | Use ToolReceipt/ForgeToolReceiptV2/ControlReceipt. |
| Local memory store becomes shadow DB | Projection truth forks. | bitemporal_asof_query; release_truth_audit | Delete local stores; use `semantic-memory` through `CanonicalMemoryAdapter`. |
