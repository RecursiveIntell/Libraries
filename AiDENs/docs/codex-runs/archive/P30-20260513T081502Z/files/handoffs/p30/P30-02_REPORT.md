# P30-02 Report

## Scope

Phase slice: patch safety, rollback truth, command sandbox, and permit fail-closed behavior in `crates/aidens-tool-kit`.

Issue IDs addressed from `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`:

- `P30-ABSORB-0006`: patch apply must not treat missing/unreadable files as empty input.
- `P30-ABSORB-0007`: rollback write failures must not be ignored.
- `P30-ABSORB-0016`: command runner must not rely on ambient `PATH`, `CARGO_HOME`, or `RUSTUP_HOME`.
- `P30-ABSORB-0017`: command timeout must terminate the process group on Unix, not only the immediate child.
- `P30-ABSORB-0030`: permit/sandbox scope must not fall back to wildcard.

## Changed Files

- `crates/aidens-tool-kit/src/lib.rs`
  - Replaced wildcard permit scope fallback with fail-closed blocking when a permit-required tool lacks a sandbox root.
  - Added Unix process-group creation for allowed commands.
  - Added timeout termination via fixed-path `/bin/kill` or `/usr/bin/kill` against the process group before falling back to direct child kill.
  - Added regression tests for missing sandbox scope and process-group timeout termination.

Observed existing implementation evidence:

- `crates/aidens-tool-kit/src/lib.rs:1186` already fails closed on patch target read errors.
- `crates/aidens-tool-kit/src/lib.rs:1393` already returns rollback errors instead of ignoring `write_file_atomically`.
- `crates/aidens-tool-kit/src/lib.rs:1508` resolves fixed executable paths and `env_clear()` remains in force.

## Tests Added Or Updated

- `side_effect_tool_without_sandbox_root_fails_closed_without_wildcard_scope`
  - Proves permit-required tools without sandbox root are blocked, have no approval request scoped to `"*"`, and carry `permit-scope-missing-sandbox-root`.
- `p30_command_timeout_terminates_process_group`
  - Proves a timed-out shell with a background child returns quickly, exercising process-group termination.

Existing relevant tests retained:

- `p30_patch_apply_missing_file_fails_closed_instead_of_empty_input`
- `p30_run_checks_uses_fixed_executable_paths_without_ambient_path`
- `p28_command_timeout_wait_uses_adaptive_backoff`

## Commands Run

- `cargo test --manifest-path Cargo.toml -p aidens-tool-kit p30_ -- --nocapture`
  - Result: pass, 4 targeted P30 tests passed.
- `cargo test --manifest-path Cargo.toml -p aidens-tool-kit side_effect_tool_without_sandbox_root_fails_closed_without_wildcard_scope -- --nocapture`
  - Result: pass, 1 targeted unit test passed.
- `cargo check --manifest-path Cargo.toml -p aidens-tool-kit --all-targets --locked`
  - Result: pass.
- `cargo test --manifest-path Cargo.toml -p aidens-tool-kit --all-targets --locked`
  - Result: pass, 31 unit tests and 4 integration tests passed for `aidens-tool-kit`.
- `cargo fmt --manifest-path Cargo.toml --all -- --check`
  - Result: pass.
- `python3 scripts/p30_guard.py --repo .`
  - Result: exit 0, `findings=1838 hard=0`.

## Unresolved Risks And Quarantines

- Unix process-group termination is covered. Non-Unix behavior still falls back to direct child termination because this workspace does not define a Windows job-object implementation.
- `p30_guard.py` warning count remains broad existing warning debt. No hard findings were reported.
- This phase did not run full workspace test/clippy/doc gates.

## Invariant Revalidation Checklist

- Patch read failures are receipt-bearing failures, not empty input.
- Rollback failures are returned and included in patch failure text.
- Command execution uses fixed executable resolution and `env_clear`.
- Timeout path attempts process-group termination on Unix.
- Permit-required tools cannot use wildcard sandbox scope when sandbox root is missing.
- Existing approval/permit flows with explicit sandbox roots still pass.

## Proceed Statement

P30-02 can proceed based on targeted code evidence and passing `aidens-tool-kit` validation. Remaining non-Unix process-tree semantics are explicit release debt, not claimed fixed.
