# P28 Phase 08 Report

## Scope

Hardened supported-local tool behavior for symlink listing, file stat hashing, repository search pruning, patch write safety already started in Phase 01, timeout partial-output marking, `repo_list` truncation disclosure, and command timeout wait behavior.

## Files changed

- `crates/aidens-tool-kit/src/lib.rs`
- `crates/aidens-contracts/src/lib.rs`
- `handoffs/p28/PHASE_08_REPORT.md`

## Claims made

- Claim: `repo_list` uses symlink metadata and does not follow symlinked targets while listing.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_tool_kit_p28_repo_list_phase08.log`
- Claim: `file_stat` fails on unreadable digest input instead of silently producing an empty digest.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_tool_kit_p28_file_stat_phase08.log`
- Claim: `repo_search` skips `.git` and `target` as path components anywhere in the searched tree.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_tool_kit_p28_repo_search_phase08.log`
- Claim: command timeout output is classified as partial/timeout.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_tool_kit_p28_run_command_phase08.log`
- Claim: `repo_list` truncation discloses total entries and a full-list digest.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_tool_kit_p28_repo_list_phase08_closeout.log`
- Claim: command timeout wait uses adaptive bounded sleep instead of fixed short polling.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_tool_kit_p28_command_timeout_phase08_closeout.log`

## Evidence

- `target/p28/audit/cargo_fmt_phase08.log`
- `target/p28/audit/cargo_check_phase08.log`
- `target/p28/audit/cargo_test_aidens_tool_kit_p28_repo_list_phase08.log`
- `target/p28/audit/cargo_test_aidens_tool_kit_p28_file_stat_phase08.log`
- `target/p28/audit/cargo_test_aidens_tool_kit_p28_repo_search_phase08.log`
- `target/p28/audit/cargo_test_aidens_tool_kit_p28_run_command_phase08.log`
- `target/p28/audit/cargo_fmt_phase08_closeout.log`
- `target/p28/audit/cargo_check_phase08_closeout.log`
- `target/p28/audit/cargo_test_aidens_tool_kit_p28_repo_list_phase08_closeout.log`
- `target/p28/audit/cargo_test_aidens_tool_kit_p28_command_timeout_phase08_closeout.log`

## Tests run

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p aidens-tool-kit p28_repo_list
cargo test -p aidens-tool-kit p28_file_stat
cargo test -p aidens-tool-kit p28_repo_search
cargo test -p aidens-tool-kit p28_run_command
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p aidens-tool-kit p28_repo_list
cargo test -p aidens-tool-kit p28_command_timeout
```

## Failures / degraded checks

- None in Phase 08 validation.

## Open risks

- None known in Phase 08 scope.

## Next phase readiness

Ready: Phase 08 exit gate passed with hardening checks and closeout refinements recorded.
