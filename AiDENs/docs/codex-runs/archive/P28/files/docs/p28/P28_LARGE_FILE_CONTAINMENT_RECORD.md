# P28 Large-File Containment Record

Status: passed for Phase 11 containment scope.

P28 split the v11A material-operation contract surface into owner-plane modules under `crates/aidens-contracts/src/` and split the runner into execution, receipt, provider/tool, finalization, replay, and test modules. Public exports are preserved through `pub use` so downstream callers keep the historical API.

Containment performed:

- `aidens-contracts/src/lib.rs` is now an export/orchestration file.
- Historical contract DTO families were moved into topic modules including provider, capability turn, agent bundle, tool artifacts, daemon queue, mechanism display, view runtime, release/completion, and reserved v11 surfaces.
- P28 v11A modules carry explicit AiDENs-local owner-plane docs and do not claim canonical truth ownership.
- `aidens-runner/src/lib.rs` now delegates receipt DTOs, Plan-Act-Verify execution helpers, provider/tool helpers, finalization text shaping, replay path helpers, and tests to dedicated modules.
- `aidens-cli` remains a display/adapter layer and was not given semantic truth ownership.

Compatibility evidence:

- `cargo check -p aidens-contracts --all-targets`
- `cargo test -p aidens-contracts p28`
- `cargo check -p aidens-runner --all-targets`
- `cargo test -p aidens-runner p26_plan_act_verify_loop`
- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`

Known residual risk:

- Some topic modules remain large because they preserve broad historical DTO/test families. This is a readability concern, not a P28 public API blocker.
