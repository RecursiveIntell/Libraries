# P04 Capability Gate, Permits, And Approval Spine Handoff

## Scope

Implemented P04 only. Durable permit/receipt storage remains deferred to P05.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-permit-kit/src/lib.rs`
- `crates/aidens-tool-kit/src/lib.rs`
- `crates/aidens-capability-kit/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `tests/fixtures/p04/capability_gate_decision_v1.json`
- `tests/fixtures/p04/tool_exposure_plan_v2.json`
- `tests/fixtures/p04/approval_request_v1.json`
- `tests/fixtures/p04/approval_decision_v1.json`
- `tests/fixtures/p04/permit_grant_v1.json`
- `tests/fixtures/p04/permit_use_receipt_v1.json`
- `ARTIFACT_SCHEMA_REGISTRY.md`
- `STATUS.md`

## Tests Added

- Contract tests for scoped permits, capability gate evidence, and P04 fixture deserialization.
- Permit policy tests for risk/tool/sandbox scoped grant matching.
- Tool-kit tests for lifecycle distinctions, permit-required exposure blocking, and receipt-bearing side-effect invocation denial.
- CLI tests for lifecycle output and typed permit request/approval/denial/revocation command output.

## Commands Run

- `cargo check -p aidens-contracts -p aidens-permit-kit -p aidens-tool-kit -p aidens-cli -p aidens-security-kit`
- `cargo fmt --all`
- `cargo test -p aidens-contracts -p aidens-permit-kit -p aidens-tool-kit -p aidens-security-kit -p aidens-cli`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `bash scripts/verify.sh`

`cargo clippy --workspace --all-targets --all-features -- -D warnings` initially flagged one useless iterator conversion in the CLI doctor tool section. The conversion was removed and the final `bash scripts/verify.sh` gate passed.

## Blockers

None for P04 acceptance.

Deferred by build order:

- Durable approval/permit/receipt ledgers remain P05 work.
- Broader coding tool execution and sandboxed write/shell/network behavior remain P10 work.
- Compiler-grade boundary validation remains P06 work.

## Next-Pass Readiness

P05 can start from typed approval requests, approval decisions, scoped permit grants, permit-use receipts, and capability gate decisions. The current CLI emits these artifacts but does not persist them durably; that persistence boundary belongs to P05.
