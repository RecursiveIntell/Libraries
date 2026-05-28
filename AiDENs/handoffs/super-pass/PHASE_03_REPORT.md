# PHASE 03 REPORT — Tool/permit exposure parity

## Scope

- Backlog rows selected: 45 rows, `AHD-0071` through `AHD-0115`, all with `Suggested_Phase = Phase 03 tool/permit exposure parity`.
- Files/crates touched: `crates/aidens-tool-kit/src/lib.rs`, `matrices/SUPER_PASS_BACKLOG_1020.csv`, `matrices/SUPER_PASS_BACKLOG_1020.json`.
- Non-goals: no new tool authority was added; this phase tightens disclosure and persistence for the existing supported-local registry.

## Changes made

| Area | Files | Summary |
|---|---|---|
| Descriptor persistence | `crates/aidens-tool-kit/src/lib.rs` | Canonical bridge descriptors now set `ToolReceiptPersistence::ForgeRaw` for side-effecting tools that require permits; read-only tools remain `Ephemeral`. |
| Tool parity fixtures | `crates/aidens-tool-kit/src/lib.rs` | Existing semantic fixtures prove disabled tools are absent, declared/registered/executable/exposed states are distinct, provider schemas match exposed tools, and side-effect execution is permit-gated. Added a descriptor persistence assertion. |

## Tests/commands run

| Command | Result | Evidence/log path |
|---|---|---|
| `cargo test -p aidens-tool-kit` | pass | `target/super-pass/audit/phase03-cargo-test-aidens-tool-kit.log` |
| `cargo check -p aidens-tool-kit -p aidens-permit-kit -p aidens-contracts` | pass | `target/super-pass/audit/phase03-cargo-check-tool-permit-contracts.log` |

## Issue matrix updates

| Status | Count | IDs |
|---|---:|---|
| fixed | 45 | `AHD-0071` through `AHD-0115` |
| quarantined | 0 |  |
| deferred | 0 |  |
| superseded | 0 |  |
| open-blocking | 0 |  |

## Gate result

- Phase gate: Tool exposure and permit parity.
- Result: Pass for scoped registry and permit surfaces. Disabled/admin tools remain unreachable, exposed tools match provider schemas, side-effect tools require permits, and side-effect descriptors request durable raw receipt persistence.
- Remaining risk: Final release still needs later command, provider, queue, boundary, and package replay gates.

## Notes for next phase

Phase 04 should harden patch application beyond the current simple replacement engine and should reuse the permit receipt IDs already threaded through patch apply.
