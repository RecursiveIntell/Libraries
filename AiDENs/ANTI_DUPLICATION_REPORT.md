# Anti-Duplication Report

## Primary finding

The original manifest gap has been closed at the workspace level: the AiDENs
root `Cargo.toml` now resolves canonical stack crates from sibling paths under
`~/Coding/Libraries`, including `stack-ids`, Forge, bridge, memory, runtime,
tool runtime, kernel, authority, assurance, and verification crates.

The remaining duplication risk is semantic. Later phases must collapse local
AiDENs contract, receipt, memory, governance, and kernel surfaces into
canonical-crate adapters or finite compatibility shims.

## Highest-risk duplicate domains

| Domain | AiDENs-local surface | Canonical / actual surface | Risk |
|---|---|---|---|
| Primitive IDs | `aidens-contracts`, receipts/config types | `libraries/stack-ids` | Split identity and trace spaces. |
| Evidence/memory | `aidens-memory-kit`, `aidens-receipts` | `semantic-memory-forge`, `forge-memory-bridge`, `semantic-memory`, `knowledge-runtime` | Parallel evidence/projection semantics. |
| Kernel | `aidens-kernel-kit` | `recursive-kernel-core`, `constraint-compiler`, `kernel-execution`, `kernel-oracles` | Local kernel drift. |
| Governance | `aidens-governance-kit`, `aidens-permit-kit`, `aidens-arbiter-kit` | `verification-*`, `authority-delegation`, `assurance-runtime` | Shadow policy/promotion law. |
| Execution receipts | `aidens-receipts`, runner/scheduler/budget kits | `llm-tool-runtime`, `verification-control`, `forge-pilot` | Execution context remains non-canonical. |
| Boundary/schema | `aidens-boundary-kit`, `aidens-contracts` | `contract-schema-gen`, `semantic-memory-forge`, `forge-memory-bridge` | Contract drift. |

## AiDENs crate-to-stack target map

| aidens_crate | target_stack_crates | note |
| --- | --- | --- |
| aidens-contracts | stack-ids; contract-schema-gen; semantic-memory-forge; forge-memory-bridge | Boundary/schema contracts; canonical IDs first. |
| aidens-boundary-kit | contract-schema-gen; forge-memory-bridge; semantic-memory-forge; llm-tool-runtime | Boundary adapter over canonical contract/bridge/tool crates. |
| aidens-receipts | llm-tool-runtime; verification-control; forge-pilot; semantic-memory-forge | Execution receipts and evidence artifacts. |
| aidens-memory-kit | semantic-memory; semantic-memory-forge; forge-memory-bridge; knowledge-runtime; forge-engine | Memory/evidence/projection integration. |
| aidens-kernel-kit | recursive-kernel-core; constraint-compiler; kernel-execution; kernel-oracles; kernel-conformance | Recursive kernel facade. |
| aidens-repair-kit | verification-control; semantic-memory; typed-patch | Repair artifacts and typed patches. |
| aidens-delegation-kit | authority-delegation; verification-policy | Delegation/policy authority. |
| aidens-governance-kit | verification-policy; verification-control; verification-adjudication; assurance-runtime | Governance/control law. |
| aidens-permit-kit | verification-policy; authority-delegation | Permit policy surfaces. |
| aidens-budget-kit | forge-pilot; verification-control; llm-tool-runtime | Budget/deadline lineage. |
| aidens-schedule-kit | forge-pilot; kernel-execution; llm-tool-runtime | Scheduling over real runtimes. |
| aidens-provider-kit | llm-tool-runtime; remote-oracle-admission; attestation-exchange | Provider routing/admission. |
| aidens-tool-kit | llm-tool-runtime; semantic-memory-forge; attestation-exchange | Tool calls and canonical receipts. |
| aidens-queue-kit | job-queue; AI-Batch-Queue; Tauri-Queue; forge-pilot | Supplemental queue candidates; not truth semantics. |
| aidens-arbiter-kit | verification-adjudication; verification-calibration; verification-control | Arbitration/adjudication. |
| aidens-capability-kit | authority-delegation; verification-policy; attestation-exchange | Capability policy/admission. |
| aidens-runner | forge-pilot; knowledge-runtime; llm-tool-runtime; verification-control | Runner as consumer/orchestrator. |
| aidens-cli | aidens-runner; forge-pilot; knowledge-runtime | CLI app layer. |

## Top exact duplicate groups

| sha256 | size | count | archives | files |
| --- | --- | --- | --- | --- |
| d69e64c595b9511f | 15326 | 2 | libraries,libraries2 | libraries:federated-settlement/src/lib.rs; libraries2:federated-settlement/src/lib.rs |
| c13ce6ba86cba0ed | 13611 | 2 | libraries,libraries2 | libraries:profile-runtime/src/rules.rs; libraries2:profile-runtime/src/rules.rs |
| ce9c78761998532f | 9794 | 2 | libraries,libraries2 | libraries:profile-runtime/src/constitution.rs; libraries2:profile-runtime/src/constitution.rs |
| b5032db4030554ad | 9435 | 2 | libraries,libraries2 | libraries:profile-runtime/src/profile_set.rs; libraries2:profile-runtime/src/profile_set.rs |
| c8a5ce841d9e3631 | 6141 | 2 | libraries,libraries2 | libraries:profile-runtime/tests/reference_composition.rs; libraries2:profile-runtime/tests/reference_composition.rs |
| ec3764e4cc66b878 | 4082 | 2 | libraries,libraries2 | libraries:profile-runtime/tests/example_roundtrip.rs; libraries2:profile-runtime/tests/example_roundtrip.rs |
| 51655c2284ea0bfd | 3928 | 2 | libraries,libraries2 | libraries:remote-oracle-admission/tests/v25_local_constitution_refs.rs; libraries2:remote-oracle-admission/tests/v25_local_constitution_refs.rs |
| aae090fbb2c4ab72 | 3879 | 2 | libraries,libraries2 | libraries:profile-runtime/src/applicability.rs; libraries2:profile-runtime/src/applicability.rs |
| f0b6a4ba9b19786e | 3874 | 2 | libraries,libraries2 | libraries:remote-oracle-admission/tests/validation_tests.rs; libraries2:remote-oracle-admission/tests/validation_tests.rs |
| 784db8d312e8b233 | 3642 | 2 | libraries,libraries2 | libraries:federated-settlement/tests/v25_local_constitution_refs.rs; libraries2:federated-settlement/tests/v25_local_constitution_refs.rs |
| daf9f4b693efd704 | 3560 | 2 | libraries,libraries2 | libraries:profile-runtime/tests/fixture_conformance.rs; libraries2:profile-runtime/tests/fixture_conformance.rs |
| 4d41c2749d1934b5 | 3080 | 2 | libraries2 | libraries2:repo_overlay/snippets/contract_schema_gen_profile_registry.rs; libraries2:snippets/contract_schema_gen_profile_registry.rs |
| 3a377c91f93b7f87 | 2566 | 2 | libraries2 | libraries2:attestation-exchange/src/lib.rs; libraries2:repo_overlay/attestation-exchange/src/lib.rs |
| 80e0b8b20803ccdf | 2291 | 2 | libraries,libraries2 | libraries:assurance-runtime/src/profile_p5_hazard.rs; libraries2:repo_overlay/assurance-runtime/src/profile_p5_hazard.rs |
| acbd583284a13ab3 | 2290 | 2 | libraries,libraries2 | libraries:assurance-runtime/src/profile_p4_regulated.rs; libraries2:repo_overlay/assurance-runtime/src/profile_p4_regulated.rs |
| dc7d5503a984c2f9 | 2175 | 2 | libraries2 | libraries2:attestation-exchange/src/profile_p6_vendor.rs; libraries2:repo_overlay/attestation-exchange/src/profile_p6_vendor.rs |
| 7186b4bd569c148f | 2065 | 2 | libraries,libraries2 | libraries:discovery-portfolio/tests/portfolio_slice.rs; libraries2:discovery-portfolio/tests/portfolio_slice.rs |
| 81fbb601c1a6aee6 | 1849 | 2 | libraries,libraries2 | libraries:profile-runtime/src/exception.rs; libraries2:profile-runtime/src/exception.rs |
| d809386d955ebe87 | 1445 | 2 | libraries,libraries2 | libraries:discovery-portfolio/tests/budget_exhaustion_slice.rs; libraries2:discovery-portfolio/tests/budget_exhaustion_slice.rs |
| adb67fb65e1034a2 | 1425 | 2 | libraries,libraries2 | libraries:discovery-portfolio/tests/value_aware_selection_slice.rs; libraries2:discovery-portfolio/tests/value_aware_selection_slice.rs |
| 21a4c5bb653df13d | 1406 | 2 | libraries2 | libraries2:repo_overlay/snippets/stack_ids_profile_additions.rs; libraries2:snippets/stack_ids_profile_additions.rs |
| 74780cb66332f20b | 1317 | 2 | libraries2 | libraries2:scaffolds/remote-oracle-admission/src/dispute.rs; libraries2:scaffolds/verification-control/src/dispute.rs |
| ad291057479ac0c2 | 1176 | 2 | libraries,libraries2 | libraries:profile-runtime/tests/fixture_manifest.rs; libraries2:profile-runtime/tests/fixture_manifest.rs |
| 914d773e87bd2cd0 | 632 | 2 | libraries,libraries2 | libraries:profile-runtime/src/lib.rs; libraries2:profile-runtime/src/lib.rs |
| a06c44f645733506 | 598 | 2 | libraries2 | libraries2:repo_overlay/snippets/verification_policy_profile_catalog.rs; libraries2:snippets/verification_policy_profile_catalog.rs |
| 78be4ab47f0e0895 | 413 | 2 | libraries2 | libraries2:attestation-exchange/Cargo.toml; libraries2:repo_overlay/attestation-exchange/Cargo.toml |

## Required classification per AiDENs kit

Each kit should be classified as one of: `facade`, `adapter`, `delete/merge`, or `unique app layer` before adding more features.
