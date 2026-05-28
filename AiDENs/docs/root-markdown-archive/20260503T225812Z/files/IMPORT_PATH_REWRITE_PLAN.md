# Import Path Rewrite Plan

## Assumed layout

```text
/workspace/
  aidens/
  libraries/
  libraries2/    # supplemental library pool
```

## First patch

For AiDENs crates under `aidens/crates/<crate>/Cargo.toml`:

```toml
stack-ids = { path = "../../../libraries/stack-ids" }
```

## Dependency additions / review targets

| aidens_crate | dependency | source | canonical_path | action |
| --- | --- | --- | --- | --- |
| aidens-contracts | stack-ids | libraries | stack-ids/Cargo.toml | add path dependency; facade/adapter target |
| aidens-contracts | contract-schema-gen | libraries | contract-schema-gen/Cargo.toml | add path dependency; facade/adapter target |
| aidens-contracts | semantic-memory-forge | libraries | semantic-memory-forge/Cargo.toml | add path dependency; facade/adapter target |
| aidens-contracts | forge-memory-bridge | libraries | forge-memory-bridge/Cargo.toml | add path dependency; facade/adapter target |
| aidens-boundary-kit | contract-schema-gen | libraries | contract-schema-gen/Cargo.toml | add path dependency; facade/adapter target |
| aidens-boundary-kit | forge-memory-bridge | libraries | forge-memory-bridge/Cargo.toml | add path dependency; facade/adapter target |
| aidens-boundary-kit | semantic-memory-forge | libraries | semantic-memory-forge/Cargo.toml | add path dependency; facade/adapter target |
| aidens-boundary-kit | llm-tool-runtime | libraries | llm-tool-runtime/Cargo.toml | add path dependency; facade/adapter target |
| aidens-receipts | llm-tool-runtime | libraries | llm-tool-runtime/Cargo.toml | add path dependency; facade/adapter target |
| aidens-receipts | verification-control | libraries | verification-control/Cargo.toml | add path dependency; facade/adapter target |
| aidens-receipts | forge-pilot | libraries | forge-pilot/Cargo.toml | add path dependency; facade/adapter target |
| aidens-receipts | semantic-memory-forge | libraries | semantic-memory-forge/Cargo.toml | add path dependency; facade/adapter target |
| aidens-memory-kit | semantic-memory | libraries | semantic-memory/Cargo.toml | add path dependency; facade/adapter target |
| aidens-memory-kit | semantic-memory-forge | libraries | semantic-memory-forge/Cargo.toml | add path dependency; facade/adapter target |
| aidens-memory-kit | forge-memory-bridge | libraries | forge-memory-bridge/Cargo.toml | add path dependency; facade/adapter target |
| aidens-memory-kit | knowledge-runtime | libraries | knowledge-runtime/Cargo.toml | add path dependency; facade/adapter target |
| aidens-memory-kit | forge-engine | libraries | living-memory/living-memory/Cargo.toml | add path dependency; facade/adapter target |
| aidens-kernel-kit | recursive-kernel-core | libraries | recursive-kernel-core/Cargo.toml | add path dependency; facade/adapter target |
| aidens-kernel-kit | constraint-compiler | libraries | constraint-compiler/Cargo.toml | add path dependency; facade/adapter target |
| aidens-kernel-kit | kernel-execution | libraries | kernel-execution/Cargo.toml | add path dependency; facade/adapter target |
| aidens-kernel-kit | kernel-oracles | libraries | kernel-oracles/Cargo.toml | add path dependency; facade/adapter target |
| aidens-kernel-kit | kernel-conformance | libraries | kernel-conformance/Cargo.toml | add path dependency; facade/adapter target |
| aidens-repair-kit | verification-control | libraries | verification-control/Cargo.toml | add path dependency; facade/adapter target |
| aidens-repair-kit | semantic-memory | libraries | semantic-memory/Cargo.toml | add path dependency; facade/adapter target |
| aidens-repair-kit | typed-patch | libraries | Primitives/typed-patch/Cargo.toml | add path dependency; facade/adapter target |
| aidens-delegation-kit | authority-delegation | libraries | authority-delegation/Cargo.toml | add path dependency; facade/adapter target |
| aidens-delegation-kit | verification-policy | libraries | verification-policy/Cargo.toml | add path dependency; facade/adapter target |
| aidens-governance-kit | verification-policy | libraries | verification-policy/Cargo.toml | add path dependency; facade/adapter target |
| aidens-governance-kit | verification-control | libraries | verification-control/Cargo.toml | add path dependency; facade/adapter target |
| aidens-governance-kit | verification-adjudication | libraries | verification-adjudication/Cargo.toml | add path dependency; facade/adapter target |
| aidens-governance-kit | assurance-runtime | libraries | assurance-runtime/Cargo.toml | add path dependency; facade/adapter target |
| aidens-permit-kit | verification-policy | libraries | verification-policy/Cargo.toml | add path dependency; facade/adapter target |
| aidens-permit-kit | authority-delegation | libraries | authority-delegation/Cargo.toml | add path dependency; facade/adapter target |
| aidens-budget-kit | forge-pilot | libraries | forge-pilot/Cargo.toml | add path dependency; facade/adapter target |
| aidens-budget-kit | verification-control | libraries | verification-control/Cargo.toml | add path dependency; facade/adapter target |
| aidens-budget-kit | llm-tool-runtime | libraries | llm-tool-runtime/Cargo.toml | add path dependency; facade/adapter target |
| aidens-schedule-kit | forge-pilot | libraries | forge-pilot/Cargo.toml | add path dependency; facade/adapter target |
| aidens-schedule-kit | kernel-execution | libraries | kernel-execution/Cargo.toml | add path dependency; facade/adapter target |
| aidens-schedule-kit | llm-tool-runtime | libraries | llm-tool-runtime/Cargo.toml | add path dependency; facade/adapter target |
| aidens-provider-kit | llm-tool-runtime | libraries | llm-tool-runtime/Cargo.toml | add path dependency; facade/adapter target |
| aidens-provider-kit | remote-oracle-admission | libraries | remote-oracle-admission/Cargo.toml | add path dependency; facade/adapter target |
| aidens-provider-kit | attestation-exchange | libraries | attestation-exchange/Cargo.toml | add path dependency; facade/adapter target |
| aidens-tool-kit | llm-tool-runtime | libraries | llm-tool-runtime/Cargo.toml | add path dependency; facade/adapter target |
| aidens-tool-kit | semantic-memory-forge | libraries | semantic-memory-forge/Cargo.toml | add path dependency; facade/adapter target |
| aidens-tool-kit | attestation-exchange | libraries | attestation-exchange/Cargo.toml | add path dependency; facade/adapter target |
| aidens-queue-kit | job-queue | libraries2 | job-queue/Cargo.toml | add path dependency; facade/adapter target |
| aidens-queue-kit | AI-Batch-Queue | supplemental_or_missing | review required | review before use |
| aidens-queue-kit | Tauri-Queue | supplemental_or_missing | review required | review before use |
| aidens-queue-kit | forge-pilot | libraries | forge-pilot/Cargo.toml | add path dependency; facade/adapter target |
| aidens-arbiter-kit | verification-adjudication | libraries | verification-adjudication/Cargo.toml | add path dependency; facade/adapter target |
| aidens-arbiter-kit | verification-calibration | libraries | verification-calibration/Cargo.toml | add path dependency; facade/adapter target |
| aidens-arbiter-kit | verification-control | libraries | verification-control/Cargo.toml | add path dependency; facade/adapter target |
| aidens-capability-kit | authority-delegation | libraries | authority-delegation/Cargo.toml | add path dependency; facade/adapter target |
| aidens-capability-kit | verification-policy | libraries | verification-policy/Cargo.toml | add path dependency; facade/adapter target |
| aidens-capability-kit | attestation-exchange | libraries | attestation-exchange/Cargo.toml | add path dependency; facade/adapter target |
| aidens-runner | forge-pilot | libraries | forge-pilot/Cargo.toml | add path dependency; facade/adapter target |
| aidens-runner | knowledge-runtime | libraries | knowledge-runtime/Cargo.toml | add path dependency; facade/adapter target |
| aidens-runner | llm-tool-runtime | libraries | llm-tool-runtime/Cargo.toml | add path dependency; facade/adapter target |
| aidens-runner | verification-control | libraries | verification-control/Cargo.toml | add path dependency; facade/adapter target |
| aidens-cli | aidens-runner | aidens | crates/aidens-runner/Cargo.toml | add path dependency; facade/adapter target |
| aidens-cli | forge-pilot | libraries | forge-pilot/Cargo.toml | add path dependency; facade/adapter target |
| aidens-cli | knowledge-runtime | libraries | knowledge-runtime/Cargo.toml | add path dependency; facade/adapter target |
