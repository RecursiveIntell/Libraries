# P10 Handoff: Coding Agent Tooling, Sandbox, and Codex Packets

Scope: P10 only. Later passes remain deferred.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`: added `RepoReadReceiptV1`, `RepoListReceiptV1`, `PatchProposalV1`, `PatchApplyReceiptV1`, `CommandRunReportV1`, `CodexPacketV1`, `SandboxCapabilityTruthV1`, schema registration, and P10 fixture tests.
- `crates/aidens-tool-kit/src/lib.rs`: added governed repo read/list/stat/search, non-mutating patch proposal, permit-gated patch apply, permit-gated allowlisted check execution, sandbox path denial receipts, and coding exposure policy.
- `crates/aidens-tool-kit/Cargo.toml`: added security-kit dependency for sandbox path checks.
- `crates/aidens-security-kit/src/lib.rs`: added sandbox truth helper and test coverage.
- `crates/aidens-permit-kit`: reused existing scoped permit law; no default risky grants added.
- `crates/aidens-runner/src/lib.rs`: threaded permit policy into turn execution and added a P10 patch-apply turn fixture.
- `crates/aidens-runner/Cargo.toml`: added permit-kit dependency.
- `crates/aidens-cli/src/lib.rs`: added `coding` commands for repo read/list/search, patch propose/apply, run checks, sandbox truth, and Codex packet export with command receipts and receipt IDs.
- `crates/aidens-cli/Cargo.toml`: added permit-kit dependency.
- `crates/aidens-receipts/src/lib.rs`: added durable append helpers for P10 coding receipts and packet artifacts.
- `crates/aidens-profile-coding/src/lib.rs` and `crates/aidens-app-kit/src/lib.rs`: expanded the coding profile bundles to governed P10 tools.
- `crates/aidens-testkit/src/lib.rs`: updated reference exposure expectations for P10 tool lifecycle states.
- `tests/fixtures/p10/*.json`: added golden fixtures for all P10 artifacts.
- `schemas/*`: generated schema set now includes P10 artifact families; manifest reports 61 schemas.
- `README.md`, `STATUS.md`, `ARTIFACT_SCHEMA_REGISTRY.md`: updated P10 status, artifact registry, and gate evidence.

## Tests Added

- Contract constructor and golden fixture tests for all P10 artifacts.
- Tool-kit tests for read-only repo operations, non-mutating patch proposal, traversal/sensitive-prefix denial receipts, permit-gated patch apply, and permit-gated allowlisted check execution.
- Runner test proving a scoped permit can authorize patch apply through the turn executor.
- CLI test covering read, propose, approve/apply, and Codex packet generation with command receipt and receipt IDs.
- Receipt store test for durable append of all P10 coding artifacts.
- Security-kit sandbox truth test.
- Testkit reference conformance update for exposed, hidden, and blocked P10 tool states.

## Commands Run

- `cargo run -p aidens-cli -- schemas generate`
- `cargo run -p aidens-cli -- schemas check`
- `cargo test -p aidens-contracts p10`
- `cargo test -p aidens-tool-kit p10`
- `cargo test -p aidens-runner p10_runner`
- `cargo test -p aidens-cli p10`
- `cargo test -p aidens-receipts p10`
- `cargo test -p aidens-security-kit sandbox_truth`
- `cargo test -p aidens-testkit safe_coding`
- `cargo test -p aidens-tool-kit`
- `cargo test -p aidens-runner`
- `cargo test -p aidens-cli`
- `cargo test -p aidens-testkit`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `bash scripts/verify.sh`
- `bash scripts/assert_no_fake_completion.sh .`
- `bash scripts/assert_no_scaffold_promoted.sh .`

## Acceptance Notes

- Coding profile exposes read-only repo read/list/stat/search and patch-propose tools.
- `aidens:patch-apply:1` and `aidens:run-checks:1` are registered and executable, but blocked without explicit scoped permits.
- Patch proposal does not mutate files.
- Patch apply records touched paths, before/after digests, permit IDs, and permit-use IDs.
- Run checks only executes the allowlisted cargo/verify commands and records stdout/stderr digests, exit code, timeout state, and permit evidence.
- Path traversal, sandbox escape, and sensitive-prefix access fail with typed receipt-bearing denials.
- Codex packets carry current pass, next pass, issue, source map, changed files, command receipts, receipt IDs, blockers, and notes.
- `list-tools` and `doctor` distinguish exposed read-only tools from blocked side-effect tools.

## Blockers

None.

## Next-Pass Readiness

P10 is complete against the required gate. The next pass is P11. P11 should start from the P10 status ledger and keep daemon/queue/schedule/wake surfaces deferred or blocked until their own pass implements lease/idempotency/safe-mode behavior.
