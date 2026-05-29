# Phase 17 Report - App Scaffold and Profile Readiness

## Scope

- Phase: `Phase 17 app/profile scaffolding hardening`
- Backlog rows: `AHD-0976` through `AHD-1000`
- Rows touched: 25
- Final row status: 25 `fixed`, 0 raw `open`

## Changes

- Reworked generated app scaffolding to stage all files in a temporary sibling directory, write each payload with create-new semantics, reject path escape, reject existing targets, clean staging on failure, and publish with a single rename into the destination.
- Added `aidens-scaffold-manifest.json` as the first scaffold payload. The manifest declares support tier, explicit mock fixture route, receipt store, sandbox root, canonical owners, forbidden claims, and receipt-first reason codes.
- Tightened generated README/operator/receipt docs to avoid readiness overclaims and to require durable receipts plus smoke tests before completion claims.
- Expanded generated `tests/smoke.rs` so scaffolded packages prove the manifest exists, the config is receipt-first, the default provider is explicitly mock, and no secret/API key fields or side-effect bundles are emitted by default.
- Added app-kit runtime default disclosure for profile, support tier, provider route, receipt store, sandbox root, enabled/disabled tool bundles, and permit policy.
- Replaced ambiguous profile-crate scaffold notes with explicit support-tier and non-goal status for coding, daemon, desktop, memory, and research profile crates.

## Files Changed

- `crates/aidens-app-kit/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-cli/src/tests.rs`
- `crates/aidens-profile-coding/src/lib.rs`
- `crates/aidens-profile-daemon/src/lib.rs`
- `crates/aidens-profile-desktop/src/lib.rs`
- `crates/aidens-profile-memory/src/lib.rs`
- `crates/aidens-profile-research/src/lib.rs`
- `matrices/SUPER_PASS_BACKLOG_1020.csv`
- `matrices/SUPER_PASS_BACKLOG_1020.json`

## Tests Run

- `cargo test -p aidens-cli scaffold -- --nocapture`
  - Log: `target/super-pass/audit/phase17-cargo-test-aidens-cli-scaffold.log`
- `cargo test -p aidens-cli`
  - Log: `target/super-pass/audit/phase17-cargo-test-aidens-cli.log`
- `cargo test -p aidens-app-kit`
  - Log: `target/super-pass/audit/phase17-cargo-test-aidens-app-kit.log`
- `cargo test -p aidens-profile-coding -p aidens-profile-daemon -p aidens-profile-desktop -p aidens-profile-memory -p aidens-profile-research`
  - Log: `target/super-pass/audit/phase17-cargo-test-aidens-profiles.log`
- Real generated scaffold replay: `cargo run -p aidens-cli -- new coding-agent target/super-pass/tmp/phase17-generated-agent`
  - Log: `target/super-pass/audit/phase17-generated-scaffold-create.log`
- Generated package smoke test: `cargo test --manifest-path target/super-pass/tmp/phase17-generated-agent/Cargo.toml`
  - Log: `target/super-pass/audit/phase17-generated-scaffold-cargo-test.log`
- `python3 scripts/assert_p29_audit_matrix_closure.py --completed-through 17`
  - Log: `target/super-pass/audit/phase17-audit-matrix-closure-through-17.log`
- `cargo fmt --all --check`
  - Log: `target/super-pass/audit/phase17-cargo-fmt-check.log`
- `cargo check --workspace --all-targets`
  - Log: `target/super-pass/audit/phase17-cargo-check-workspace-all-targets.log`
- `cargo clippy --workspace --all-targets -- -D warnings`
  - Log: `target/super-pass/audit/phase17-cargo-clippy-workspace-all-targets.log`
- `cargo test --workspace --all-targets`
  - Log: `target/super-pass/audit/phase17-cargo-test-workspace-all-targets.log`

## Rows Closed

- Fixed: `AHD-0976` through `AHD-1000`
- Quarantined: none
- Deferred: none
- Open-blocking: none

## Unresolved Risk

- The generated app remains a scaffolded supported-local starting point; it is not a completed app-run proof until its operator runs it and inspects the resulting durable receipts.
- Non-coding profile crates still expose status only unless later product work adds executable wiring and hostile fixtures.

## Exit Decision

Continue. Phase 17 exit gate passed: scaffold overwrite/path-escape/secret tests pass, a real generated scaffold package passes its own smoke tests, matrix closure through Phase 17 passes, and the broad workspace command bar is green.
