# Master Inventory — AiDENs + Libraries Anti-Duplication Pass

Generated: `2026-04-27T18:04:11Z`

## Archive summary

| archive | files | unique_paths | duplicate_paths | uncompressed_bytes | rust_files | cargo_manifests | md_txt |
| --- | --- | --- | --- | --- | --- | --- | --- |
| libraries | 1251 | 1251 | 0 | 7331537 | 422 | 45 | 239 |
| libraries2 | 1251 | 1251 | 0 | 5679905 | 285 | 21 | 390 |
| aidens | 504 | 504 | 0 | 2424975 | 38 | 32 | 138 |
| research_zip | 52 | 52 | 0 | 1589178 | 0 | 0 | 52 |

## Stack ID crate decision

| archive | package | version | path | status |
| --- | --- | --- | --- | --- |
| libraries | stack-ids | 0.1.0 | stack-ids/Cargo.toml | workspace_member |
| libraries2 | stack-ids | 0.2.0 | stack-ids/Cargo.toml | workspace_member |

**Canonical:** `libraries/stack-ids/Cargo.toml` (`package = stack-ids`, version `0.1.0`).  
**Not canonical:** any `Libraries2` stack-ids path, overlay, or scaffold.

## `libraries.zip` workspace members

| package | version | path | rs_files | deps |
| --- | --- | --- | --- | --- |
| assurance-runtime | 0.1.0 | assurance-runtime/Cargo.toml | 15 | 5 |
| attestation-exchange | 0.1.0 | attestation-exchange/Cargo.toml | 6 | 5 |
| authority-delegation | 0.1.0 | authority-delegation/Cargo.toml | 11 | 4 |
| constitutional-memory | 0.1.0 | constitutional-memory/Cargo.toml | 5 | 6 |
| constraint-compiler | 0.1.0 | constraint-compiler/Cargo.toml | 2 | 8 |
| continuity-runtime | 0.1.0 | continuity-runtime/Cargo.toml | 13 | 4 |
| contract-schema-gen | 0.1.0 | contract-schema-gen/Cargo.toml | 3 | 27 |
| discovery-portfolio | 0.1.0 | discovery-portfolio/Cargo.toml | 5 | 5 |
| effect-runtime | 0.1.0 | effect-runtime/Cargo.toml | 12 | 7 |
| federated-settlement | 0.1.0 | federated-settlement/Cargo.toml | 5 | 4 |
| forge-memory-bridge | 0.1.0 | forge-memory-bridge/Cargo.toml | 7 | 9 |
| forge-pilot | 0.1.0 | forge-pilot/Cargo.toml | 77 | 37 |
| kernel-conformance | 0.1.0 | kernel-conformance/Cargo.toml | 9 | 25 |
| kernel-execution | 0.1.0 | kernel-execution/Cargo.toml | 1 | 6 |
| kernel-oracles | 0.1.0 | kernel-oracles/Cargo.toml | 1 | 11 |
| knowledge-runtime | 0.1.0 | knowledge-runtime/Cargo.toml | 32 | 17 |
| forge-engine | 0.2.0 | living-memory/living-memory/Cargo.toml | 56 | 36 |
| llm-tool-runtime | 0.1.0 | llm-tool-runtime/Cargo.toml | 9 | 10 |
| mechanism-runtime | 0.1.0 | mechanism-runtime/Cargo.toml | 5 | 6 |
| profile-runtime | 0.1.0 | profile-runtime/Cargo.toml | 13 | 13 |
| recursive-kernel-core | 0.1.0 | recursive-kernel-core/Cargo.toml | 2 | 4 |
| remote-oracle-admission | 0.1.0 | remote-oracle-admission/Cargo.toml | 3 | 4 |
| semantic-memory-forge | 0.1.0 | semantic-memory-forge/Cargo.toml | 10 | 9 |
| semantic-memory | 0.5.0 | semantic-memory/Cargo.toml | 60 | 17 |
| spec-execution | 0.1.0 | spec-execution/Cargo.toml | 4 | 7 |
| stack-ids | 0.1.0 | stack-ids/Cargo.toml | 8 | 7 |
| verification-adjudication | 0.1.0 | verification-adjudication/Cargo.toml | 6 | 7 |
| verification-calibration | 0.1.0 | verification-calibration/Cargo.toml | 2 | 5 |
| verification-control | 0.1.0 | verification-control/Cargo.toml | 7 | 7 |
| verification-policy | 0.1.0 | verification-policy/Cargo.toml | 12 | 6 |

## AiDENs workspace members

| package | path | rs_files | deps |
| --- | --- | --- | --- |
| aidens-app-kit | crates/aidens-app-kit/Cargo.toml | 2 | 9 |
| aidens-arbiter-kit | crates/aidens-arbiter-kit/Cargo.toml | 1 | 5 |
| aidens-boundary-kit | crates/aidens-boundary-kit/Cargo.toml | 1 | 6 |
| aidens-budget-kit | crates/aidens-budget-kit/Cargo.toml | 1 | 3 |
| aidens-capability-kit | crates/aidens-capability-kit/Cargo.toml | 1 | 3 |
| aidens-cli | crates/aidens-cli/Cargo.toml | 3 | 17 |
| aidens-config | crates/aidens-config/Cargo.toml | 1 | 7 |
| aidens-contracts | crates/aidens-contracts/Cargo.toml | 1 | 6 |
| aidens-daemon-kit | crates/aidens-daemon-kit/Cargo.toml | 1 | 8 |
| aidens-delegation-kit | crates/aidens-delegation-kit/Cargo.toml | 1 | 4 |
| aidens-governance-kit | crates/aidens-governance-kit/Cargo.toml | 1 | 4 |
| aidens-kernel-kit | crates/aidens-kernel-kit/Cargo.toml | 1 | 5 |
| aidens-memory-kit | crates/aidens-memory-kit/Cargo.toml | 1 | 6 |
| aidens-permit-kit | crates/aidens-permit-kit/Cargo.toml | 1 | 5 |
| aidens-plan-kit | crates/aidens-plan-kit/Cargo.toml | 1 | 3 |
| aidens-profile-coding | crates/aidens-profile-coding/Cargo.toml | 1 | 3 |
| aidens-profile-daemon | crates/aidens-profile-daemon/Cargo.toml | 1 | 3 |
| aidens-profile-desktop | crates/aidens-profile-desktop/Cargo.toml | 1 | 3 |
| aidens-profile-memory | crates/aidens-profile-memory/Cargo.toml | 1 | 3 |
| aidens-profile-research | crates/aidens-profile-research/Cargo.toml | 1 | 3 |
| aidens-provider-kit | crates/aidens-provider-kit/Cargo.toml | 1 | 9 |
| aidens-queue-kit | crates/aidens-queue-kit/Cargo.toml | 1 | 5 |
| aidens-receipts | crates/aidens-receipts/Cargo.toml | 1 | 5 |
| aidens-repair-kit | crates/aidens-repair-kit/Cargo.toml | 1 | 6 |
| aidens-runner | crates/aidens-runner/Cargo.toml | 3 | 12 |
| aidens-schedule-kit | crates/aidens-schedule-kit/Cargo.toml | 1 | 5 |
| aidens-security-kit | crates/aidens-security-kit/Cargo.toml | 1 | 3 |
| aidens-testkit | crates/aidens-testkit/Cargo.toml | 2 | 4 |
| aidens-tool-kit | crates/aidens-tool-kit/Cargo.toml | 2 | 10 |
| aidens-wake-kit | crates/aidens-wake-kit/Cargo.toml | 1 | 4 |
| aidens | crates/aidens/Cargo.toml | 1 | 7 |

## Duplicate package-name decisions

| package | canonical | decision | locations |
| --- | --- | --- | --- |
| attestation-exchange | libraries:attestation-exchange/Cargo.toml | canonical = libraries workspace member. | libraries:attestation-exchange/Cargo.toml@0.1.0; libraries2:attestation-exchange/Cargo.toml@0.1.0; libraries2:repo_overlay/attestation-exchange/Cargo.toml@0.1.0; libraries2:scaffolds/attestation-exchange/Cargo.toml@0.1.0 |
| constraint-compiler | libraries:constraint-compiler/Cargo.toml | canonical = libraries workspace member. | libraries:constraint-compiler/Cargo.toml@0.1.0; libraries2:constraint-compiler/Cargo.toml@0.1.0 |
| discovery-portfolio | libraries:discovery-portfolio/Cargo.toml | canonical = libraries workspace member. | libraries:discovery-portfolio/Cargo.toml@0.1.0; libraries2:discovery-portfolio/Cargo.toml@0.1.0 |
| federated-settlement | libraries:federated-settlement/Cargo.toml | canonical = libraries workspace member. | libraries:federated-settlement/Cargo.toml@0.1.0; libraries2:federated-settlement/Cargo.toml@0.1.0 |
| profile-runtime | libraries:profile-runtime/Cargo.toml | canonical = libraries workspace member. | libraries:profile-runtime/Cargo.toml@0.1.0; libraries2:profile-runtime/Cargo.toml@0.1.0 |
| remote-oracle-admission | libraries:remote-oracle-admission/Cargo.toml | canonical = libraries workspace member. | libraries:remote-oracle-admission/Cargo.toml@0.1.0; libraries2:remote-oracle-admission/Cargo.toml@0.1.0; libraries2:scaffolds/remote-oracle-admission/Cargo.toml@0.1.0 |
| spec-execution | libraries:spec-execution/Cargo.toml | canonical = libraries workspace member. | libraries:spec-execution/Cargo.toml@0.1.0; libraries2:spec-execution/Cargo.toml@0.1.0 |
| stack-ids | libraries:stack-ids/Cargo.toml | CANONICAL_BY_USER: use libraries/stack-ids, not Libraries2/stack-ids. | libraries:stack-ids/Cargo.toml@0.1.0; libraries2:stack-ids/Cargo.toml@0.2.0 |

## Static dependency result

Current AiDENs root workspace dependencies include actual stack package names:
**15** canonical sibling crates at `Cargo.toml:L58-L72`.

Current AiDENs member crates consuming those stack dependencies through
`workspace = true`: **6** (`aidens-contracts`, `aidens-governance-kit`,
`aidens-kernel-kit`, `aidens-memory-kit`, `aidens-receipts`, and
`aidens-testkit`).

## Exact source duplicates

Exact duplicate source/manifest groups detected: **26**. See `EXACT_SOURCE_DUPLICATES.csv`.

## Research/spec documents indexed

Research/spec documents indexed: **95**. See `RESEARCH_SOURCE_INDEX.md`.
