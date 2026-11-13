# Libraries documentation

Start with the [repository README](../README.md) for the component map and development entry points. Current source, manifests, tests, and source-bound evidence take precedence over dated plans and audit summaries.

## Current contracts and validation

- [Root Cargo workspace](../Cargo.toml) and [support lane](../support_lane.toml)
- [Support profile](../SUPPORT_PROFILE.md) and [scope notes](../SCOPE_NOTES.md)
- [Conformance gates](../CONFORMANCE_GATES.md), [Makefile](../Makefile), and [CI workflow](../.github/workflows/ci.yml)
- [Claim-to-evidence manifest](claims_manifest.json)
- [Recorded status evidence](../STATUS_EVIDENCE_MANIFEST.json) and [closeout receipt](../release/closeout_receipt_v1.json), checked by `make gate`
- [Semantic ownership registry](semantics/OWNER_REGISTRY_V1.yaml) and [semantic law](semantics/SEMANTIC_LAW_V1.md)
- [Module budget exceptions](module_budget_exceptions.md)

Recorded evidence is bound to a source revision and environment. It is not automatically current because it is checked into the repository.

## Designs, plans, and dated evidence

- [Libraries reconciliation plan, August 3, 2026](LIBRARIES_COMPLETION_PLAN_CURRENT_2026-08-03.md)
- [Memory firewall design](MEMORY_FIREWALL_DESIGN_V0_1.md)
- [Agent Evidence Workbench product specification](plans/2026-07-02-agent-evidence-workbench-product-spec.md)
- [Receipt schema specification](PHASE_1_RECEIPT_SCHEMA_SPEC.md)
- [Agent-security argument-provenance specification](PHASE_2_AGENT_SECURITY_ARG_PROVENANCE_SPEC.md)
- [Compression survivability specification](PHASE_3_COMPRESSION_SURVIVABILITY_SPEC.md)
- [FibQuant source basis](compression/FIBQUANT_SOURCE_BASIS.md), [math conformance](compression/FIBQUANT_MATH_CONFORMANCE.md), and [benchmark plan](compression/FIBQUANT_BENCHMARK_PLAN.md)
- [Dated receipts](receipts/) and [post-salvage validation report](post-salvage-validation/FINAL_REPORT.md)

Plans describe intended work; dated receipts describe the checks actually captured. Neither should be read as proof that every proposed capability is implemented or that a historical result still holds at HEAD.

## Historical packs

- [Archive supersession index](archive/SUPERSESSION_INDEX.md)
- [V21–V24 governance surface decision table](closeout_v21_v24/governance_surface_decision_table.md)
- [May 2026 completion-plan synthesis](LIBRARIES_COMPLETION_PLAN_MASTER.md)
- [FibQuant paper-core implementation prompt](../OPERATOR_PASTE_FIRST.md), [source basis](../SOURCE_BASIS.md), and [acceptance gates](../05_ACCEPTANCE_GATES.md)
- [Recorded coding-run index](codex-runs/CODEX_RUN_INDEX.md)

These files preserve design and audit history. Check the applicable component's source and current tests before following an old command, status assertion, or next-step list.
