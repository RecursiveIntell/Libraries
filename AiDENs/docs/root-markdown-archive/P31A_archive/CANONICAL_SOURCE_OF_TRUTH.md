# Canonical Source-of-Truth Rules

## Non-negotiable

Use `stack-ids` from `libraries/stack-ids`.

## Precedence

1. `libraries` owns concepts first.
2. `libraries/stack-ids` wins specifically; never use stale stack-id overlays or scaffolds.
3. Former supplemental crates salvaged from the staging root now resolve from `libraries`.
4. When both source pools appear to own the same non-ID concept, follow `CANONICAL_OWNER_MAP.md` instead of creating an AiDENs-local owner.
4. AiDENs crates are application/profile/facade crates unless proven unique.
5. Do not build local ID, envelope, receipt, promotion, time, or memory semantics in AiDENs when a canonical stack crate owns them.

## Package decisions

| package | canonical_archive | canonical_path | decision |
| --- | --- | --- | --- |
| agent-graph | libraries | agent-graph/Cargo.toml | salvaged canonical Libraries package; standalone/excluded from workspace. |
| ai-batch-queue | libraries | ai-batch-queue/Cargo.toml | salvaged canonical Libraries package; standalone/excluded from workspace. |
| assurance-runtime | libraries | assurance-runtime/Cargo.toml | canonical = libraries workspace member. |
| attestation-exchange | libraries | attestation-exchange/Cargo.toml | canonical = libraries workspace member. |
| authority-delegation | libraries | authority-delegation/Cargo.toml | canonical = libraries workspace member. |
| cea-core | libraries | Primitives/cea-core/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| cea-sqlite | libraries | Primitives/cea-sqlite/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| cea-store | libraries | Primitives/cea-store/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| check-runner | libraries | Primitives/check-runner/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| comfyui-rs | libraries | comfyui-rs/Cargo.toml | salvaged canonical Libraries package; standalone/excluded from workspace. |
| constitutional-memory | libraries | constitutional-memory/Cargo.toml | canonical = libraries workspace member. |
| constraint-compiler | libraries | constraint-compiler/Cargo.toml | canonical = libraries workspace member. |
| continuity-runtime | libraries | continuity-runtime/Cargo.toml | canonical = libraries workspace member. |
| contract-schema-gen | libraries | contract-schema-gen/Cargo.toml | canonical = libraries workspace member. |
| demo-tauri-libraries | libraries | _salvage_from_libraries2/.../demo-tauri-libraries/src-tauri/Cargo.toml | archived demo only; not a canonical dependency. |
| discovery-portfolio | libraries | discovery-portfolio/Cargo.toml | canonical = libraries workspace member. |
| effect-runtime | libraries | effect-runtime/Cargo.toml | canonical = libraries workspace member. |
| effect-signature | libraries | Primitives/effect-signature/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| federated-settlement | libraries | federated-settlement/Cargo.toml | canonical = libraries workspace member. |
| forge-engine | libraries | living-memory/living-memory/Cargo.toml | canonical = libraries workspace member. |
| forge-memory-bridge | libraries | forge-memory-bridge/Cargo.toml | canonical = libraries workspace member. |
| forge-pilot | libraries | forge-pilot/Cargo.toml | canonical = libraries workspace member. |
| forge-policy | libraries | Primitives/forge-policy/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| forge-policy-fuzz | libraries | Primitives/forge-policy/fuzz/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| job-queue | libraries | job-queue/Cargo.toml | salvaged canonical Libraries package; standalone/excluded from workspace. |
| kernel-conformance | libraries | kernel-conformance/Cargo.toml | canonical = libraries workspace member. |
| kernel-execution | libraries | kernel-execution/Cargo.toml | canonical = libraries workspace member. |
| kernel-oracles | libraries | kernel-oracles/Cargo.toml | canonical = libraries workspace member. |
| knowledge-runtime | libraries | knowledge-runtime/Cargo.toml | canonical = libraries workspace member. |
| llm-output-parser | libraries | llm-output-parser/Cargo.toml | salvaged canonical Libraries package; standalone/excluded from workspace. |
| llm-pipeline | libraries | llm-pipeline/Cargo.toml | salvaged canonical Libraries package; standalone/excluded from workspace. |
| llm-tool-runtime | libraries | llm-tool-runtime/Cargo.toml | canonical = libraries workspace member. |
| mechanism-runtime | libraries | mechanism-runtime/Cargo.toml | canonical = libraries workspace member. |
| mindstate-core | libraries | Primitives/mindstate-core/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| ollama-vision | libraries | ollama-vision/Cargo.toml | salvaged canonical Libraries package; standalone/excluded from workspace. |
| profile-runtime | libraries | profile-runtime/Cargo.toml | canonical = libraries workspace member. |
| recursive-kernel-core | libraries | recursive-kernel-core/Cargo.toml | canonical = libraries workspace member. |
| remote-oracle-admission | libraries | remote-oracle-admission/Cargo.toml | canonical = libraries workspace member. |
| sandbox-workspace | libraries | Primitives/sandbox-workspace/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| sandbox-workspace-fuzz | libraries | Primitives/sandbox-workspace/fuzz/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| semantic-memory | libraries | semantic-memory/Cargo.toml | canonical = libraries workspace member. |
| semantic-memory-forge | libraries | semantic-memory-forge/Cargo.toml | canonical = libraries workspace member. |
| spec-execution | libraries | spec-execution/Cargo.toml | canonical = libraries workspace member. |
| stabilizer-core | libraries | Primitives/stabilizer-core/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| stack-ids | libraries | stack-ids/Cargo.toml | CANONICAL_BY_USER: use libraries/stack-ids. |
| tauri-queue | libraries | tauri-queue/Cargo.toml | salvaged canonical Libraries package; standalone/excluded from workspace. |
| typed-patch | libraries | Primitives/typed-patch/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| typed-patch-fuzz | libraries | Primitives/typed-patch/fuzz/Cargo.toml | canonical = libraries copy (non-workspace/excluded). |
| verification-adjudication | libraries | verification-adjudication/Cargo.toml | canonical = libraries workspace member. |
| verification-calibration | libraries | verification-calibration/Cargo.toml | canonical = libraries workspace member. |
| verification-control | libraries | verification-control/Cargo.toml | canonical = libraries workspace member. |
| verification-policy | libraries | verification-policy/Cargo.toml | canonical = libraries workspace member. |
