# PHASE 05 REPORT — Command execution receipts

## Scope

- Backlog rows selected: 45 rows, `AHD-0186` through `AHD-0230`, all with `Suggested_Phase = Phase 05 command execution receipts`.
- Files/crates touched: `crates/aidens-tool-kit/src/lib.rs`, `crates/aidens-cli/src/agent.rs`, `matrices/SUPER_PASS_BACKLOG_1020.csv`, `matrices/SUPER_PASS_BACKLOG_1020.json`.
- Non-goals: this phase does not broaden the allowlist or add cloud command execution.

## Changes made

| Area | Files | Summary |
|---|---|---|
| Structured argv | `crates/aidens-tool-kit/src/lib.rs` | `run-checks` schema and parser now require `command` to be an argv array; string/shell command parsing is rejected. |
| Agent mock command input | `crates/aidens-cli/src/agent.rs` | Local agent check mock now emits `["bash", "scripts/verify.sh"]` instead of a whitespace-parsed string. |
| Output caps | `crates/aidens-tool-kit/src/lib.rs` | stdout/stderr are capped before output/receipt publication; truncation emits reason codes and `partial_output_capped`. |
| Hostile fixture | `crates/aidens-tool-kit/src/lib.rs` | Added a behavioral test proving string commands fail schema validation and the cap helper enforces the configured byte limit. |

## Tests/commands run

| Command | Result | Evidence/log path |
|---|---|---|
| `cargo test -p aidens-tool-kit phase05_run_checks_rejects_string_commands_and_caps_output` | pass | `target/super-pass/audit/phase05-cargo-test-tool-command-focused.log` |
| `cargo test -p aidens-cli agent_run_persists_v3_bundle_in_receipt_store_and_inspects_after_restart` | pass | `target/super-pass/audit/phase05-cargo-test-cli-agent-run.log` |
| `cargo check -p aidens-tool-kit -p aidens-cli` | pass | `target/super-pass/audit/phase05-cargo-check-tool-cli.log` |

## Issue matrix updates

| Status | Count | IDs |
|---|---:|---|
| fixed | 45 | `AHD-0186` through `AHD-0230` |
| quarantined | 0 |  |
| deferred | 0 |  |
| superseded | 0 |  |
| open-blocking | 0 |  |

## Gate result

- Phase gate: Command execution gate.
- Result: Pass for structured argv and output cap surfaces. Existing timeout fixtures remain active.
- Remaining risk: Grandchild process-group kill and deep environment/toolchain fingerprinting are not fully proven by this focused pass; final labels must wait for later gates.

## Notes for next phase

Phase 06 should focus on provider route honesty, especially `Local` vs `mock` disclosure and typed mock responses.
