# P28 Code Change Targets

## High-risk files

- `AiDENs/crates/aidens-contracts/src/lib.rs`
- `AiDENs/crates/aidens-contracts/src/schema_catalog.rs`
- `AiDENs/crates/aidens-contracts/src/app_status.rs`
- `AiDENs/crates/aidens-runner/src/lib.rs`
- `AiDENs/crates/aidens-tool-kit/src/lib.rs`
- `AiDENs/crates/aidens-boundary-kit/src/lib.rs`
- `AiDENs/crates/aidens-receipts/src/lib.rs`
- `AiDENs/crates/aidens-cli/src/lib.rs`
- `AiDENs/z.py`
- `AiDENs/scripts/verify_current.sh`

## Recommended module splits

### `aidens-contracts`

Create modules:

- `artifact.rs` — artifact envelope, manifest, lifecycle, transition receipt
- `execution.rs` — execution context, tool receipt, operator invocation receipt
- `operator.rs` — operator contract/effects/material operation registry
- `boundary.rs` — boundary compiler profile/receipts/treatment integrity
- `proof.rs` — proof profile/debt/waiver/promotion eligibility
- `semantic.rs` — semantic state/view/degradation/exactness
- `schema_catalog.rs` — schema docs, compatibility reports, registry
- `legacy.rs` — type aliases/deprecation/migration records

### `aidens-runner`

Create modules:

- `context.rs`
- `turn.rs`
- `tool_loop.rs`
- `receipts.rs`
- `finalization.rs`
- `replay.rs`
- `provider_route.rs`

### `aidens-tool-kit`

Create modules:

- `sandbox.rs`
- `repo_read.rs`
- `repo_search.rs`
- `patch.rs`
- `checks.rs`
- `receipts.rs`

## Avoid

- adding another giant file
- weakening public APIs silently
- moving canonical ownership into AiDENs
- using random IDs for replay-sensitive artifacts
- using hardcoded singleton IDs for run-specific reports
- treating display reports as canonical receipts without fields
