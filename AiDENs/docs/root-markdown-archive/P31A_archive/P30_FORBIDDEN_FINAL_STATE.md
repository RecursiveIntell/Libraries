# P30 Forbidden Final States

The pass fails if any of these are true at final handoff:

1. A P0 issue is neither fixed nor explicitly quarantined.
2. Tool-call parsing can drop malformed executable entries without receipts.
3. Executable tool calls can be produced by permissive repair without blocking approval/degradation evidence.
4. Tool-result serialization failure can produce empty provider content.
5. Patch apply treats read failure as empty file content.
6. Rollback failure is ignored.
7. Material artifact/receipt/operator IDs depend on process-local counters, random UUIDs, or constant IDs.
8. Advisory checks appear as verification `Succeeded` for risk-bearing outputs.
9. Failure paths can return no evidence when a durable receipt is required.
10. Packaging certifier success is described as build/semantic/conformance success.
11. Missing historical gates remain referenced without supersession.
12. Root Markdown ambiguity can steer current instructions.
13. AiDENs invents duplicate memory/truth/evidence/verification semantics owned by sibling crates.
14. Final handoff claims v11A/v11B compliance without release-bar evidence.
15. `scripts/p30_verify.sh` or equivalent guard is missing at final.
