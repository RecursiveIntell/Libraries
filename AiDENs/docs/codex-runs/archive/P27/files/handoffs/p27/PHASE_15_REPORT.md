# P27 Phase Report

## Phase

- Phase ID: 15
- Phase title: Megafile containment: CLI next
- Date: 2026-05-05T11:13:36Z

## Scope

- Intended work: split `aidens-cli/src/lib.rs` by command domain behind stable CLI behavior.
- Issue IDs in scope: `P27-006`, with `P27-013` observed for touched high-argument surfaces.
- Explicit non-goals: no semantic CLI rewrites, no public command behavior changes, no broad clippy allow sweep, no support-tier widening, no canonical-owner boundary change.

## Files inspected

- `prompts/phases/P27_PHASE_15_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_15_BEFORE_PHASE_16.md`
- `P27_PHASE_PLAN.md`
- `P27_MASTER_ISSUE_MATRIX.md`
- `P27_ACCEPTANCE_GATES.md`
- `P27_11A_ALIGNMENT.md`
- `STATUS.md`
- `SUPPORT_PROFILE.md`
- `SOURCE_BASIS.md`
- `crates/aidens-cli/Cargo.toml`
- `crates/aidens-cli/src/lib.rs`

## Files changed

- `STATUS.md`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-cli/src/agent.rs`
- `crates/aidens-cli/src/package.rs`
- `crates/aidens-cli/src/tests.rs`
- `scripts/assert_p27_cli_megafile_containment.py`
- `scripts/assert_p27_strict_structured_inputs.py`
- `scripts/p27_verify.sh`
- `handoffs/p27/PHASE_15_REPORT.md`

## Changes made

- Split `aidens-cli/src/lib.rs` mechanically:
  - `agent.rs`: AgentSpec validation/doctor/new/run/inspect command handlers and run-bundle inspection helpers.
  - `package.rs`: package examples/install-smoke/readiness/completion-audit command handlers and package evidence helpers.
  - `tests.rs`: existing CLI tests.
- Kept `lib.rs` as the stable facade with `pub use agent::*;` and `pub use package::*;`.
- Made the moved package gate-command constant `pub(crate)` for existing tests.
- Updated `scripts/assert_p27_strict_structured_inputs.py` to scan all CLI source modules after the split.
- Added `scripts/assert_p27_cli_megafile_containment.py` and wired it into `scripts/p27_verify.sh`.
- Updated `STATUS.md` to close `P27-006` across contracts and CLI containment.

## Commands run

| Command | Result | Log |
|---|---|---|
| `cargo fmt` | pass | `target/p27/audit/phase15_cargo_fmt.log` |
| `cargo check -p aidens-cli` | pass | `target/p27/audit/phase15_cargo_check_cli.log` |
| `cargo test -p aidens-cli schemas_ phase13_` | invalid command syntax, rerun as separate filters | `target/p27/audit/phase15_cargo_test_cli_targeted_initial.log` |
| `cargo test -p aidens-cli package_` | fail before moved constant visibility fix | `target/p27/audit/phase15_cargo_test_cli_package.log` |
| `cargo test -p aidens-cli phase13_` | fail before moved constant visibility fix | `target/p27/audit/phase15_cargo_test_cli_phase13.log` |
| `cargo fmt` | pass | `target/p27/audit/phase15_cargo_fmt_after_const_fix.log` |
| `cargo check -p aidens-cli` | pass | `target/p27/audit/phase15_cargo_check_cli_after_const_fix.log` |
| `cargo test -p aidens-cli package_` | pass | `target/p27/audit/phase15_cargo_test_cli_package_after_const_fix.log` |
| `cargo test -p aidens-cli phase13_` | pass | `target/p27/audit/phase15_cargo_test_cli_phase13_after_const_fix.log` |
| `cargo fmt` | pass | `target/p27/audit/phase15_cargo_fmt_after_agent_split.log` |
| `cargo check -p aidens-cli` | pass | `target/p27/audit/phase15_cargo_check_cli_after_agent_split.log` |
| `python3 -m py_compile scripts/assert_p27_cli_megafile_containment.py` | pass | `target/p27/audit/phase15_py_compile_cli_containment.log` |
| `python3 scripts/assert_p27_cli_megafile_containment.py .` | pass | `target/p27/audit/phase15_assert_cli_megafile_containment.log` |
| `cargo fmt --check` | pass | `target/p27/audit/phase15_cargo_fmt_check.log` |
| `cargo test -p aidens-cli` | pass | `target/p27/audit/phase15_cargo_test_cli_full.log` |
| `cargo check -p aidens-cli -p aidens` | pass | `target/p27/audit/phase15_cargo_check_affected_cli_crates.log` |
| `cargo run --quiet -p aidens-cli -- --help` | pass | `target/p27/audit/phase15_cli_help.log` |
| `cargo run --quiet -p aidens-cli -- package --help` | pass | `target/p27/audit/phase15_cli_package_help.log` |
| `cargo run --quiet -p aidens-cli -- agent --help` | pass | `target/p27/audit/phase15_cli_agent_help.log` |
| `cargo run --quiet -p aidens-cli -- package examples --root .` | pass | `target/p27/audit/phase15_cli_package_examples.json` |
| `cargo run --quiet -p aidens-cli -- agent new --template local-coding --out target/p27/audit/phase15_agent_fixture` | pass | `target/p27/audit/phase15_cli_agent_new.log` |
| `cargo run --quiet -p aidens-cli -- agent validate --spec target/p27/audit/phase15_agent_fixture/agent.json` | pass | `target/p27/audit/phase15_cli_agent_validate.json` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | fail before strict structured-input guard scanned split modules | `target/p27/audit/phase15_verify_current_skip_cargo.log` |
| `python3 -m py_compile scripts/assert_p27_strict_structured_inputs.py` | pass | `target/p27/audit/phase15_py_compile_strict_structured_guard.log` |
| `python3 scripts/assert_p27_strict_structured_inputs.py .` | pass | `target/p27/audit/phase15_assert_strict_structured_inputs_after_split.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass | `target/p27/audit/phase15_verify_current_skip_cargo_after_guard_fix.log` |
| `python3 scripts/assert_support_claims.py .` | pass | `target/p27/audit/phase15_assert_support_claims.log` |
| `python3 scripts/assert_p27_current_run_truth.py .` | pass | `target/p27/audit/phase15_assert_p27_current_run_truth.log` |
| `python3 scripts/assert_p27_agents_md_current.py .` | pass | `target/p27/audit/phase15_assert_p27_agents_md_current.log` |

## Evidence emitted

- `target/p27/audit/phase15_cli_line_counts_initial.log`
- `target/p27/audit/phase15_cli_line_counts_after_agent_split.log`
- `target/p27/audit/phase15_assert_cli_megafile_containment.log`
- `target/p27/audit/phase15_cargo_test_cli_full.log`
- `target/p27/audit/phase15_cargo_check_affected_cli_crates.log`
- `target/p27/audit/phase15_cli_help.log`
- `target/p27/audit/phase15_cli_package_help.log`
- `target/p27/audit/phase15_cli_agent_help.log`
- `target/p27/audit/phase15_cli_package_examples.json`
- `target/p27/audit/phase15_cli_agent_validate.json`
- `target/p27/audit/phase15_verify_current_skip_cargo_after_guard_fix.log`
- `target/p27/audit/phase15_assert_support_claims.log`

## Containment result

- Before: `crates/aidens-cli/src/lib.rs` was 8,114 lines.
- After:
  - `crates/aidens-cli/src/lib.rs`: 4,545 lines.
  - `crates/aidens-cli/src/agent.rs`: 995 lines.
  - `crates/aidens-cli/src/package.rs`: 825 lines.
  - `crates/aidens-cli/src/tests.rs`: 1,752 lines.
- The new verifier guard enforces the module split and a 5,000-line maximum for the CLI facade.

## 11A semantic impact

- Exact/approx labels touched: none.
- Degradation labels touched: none.
- Support labels touched: no `SUPPORT_PROFILE.md` support-tier claim changed. `STATUS.md` records structural closure of `P27-006`.
- Proof/check hooks added: CLI megafile containment guard; strict structured-input guard now scans split CLI modules.

## Support profile impact

- No support-tier claim changed in `SUPPORT_PROFILE.md`.
- No supported-local capability was widened.
- `STATUS.md` now records `P27-006` closed because both contracts and CLI megafile containment have verifier guards.

## Canonical-owner impact

- No canonical-owner boundary changed.
- This phase only reorganized AiDENs-local CLI facade code and tests.

## Issues closed

- `P27-006`: closed across Phase 14 and Phase 15 by splitting contracts and CLI facades with verifier guards.

## New issues / risks

- `P27-013` remains open for broader high-argument API cleanup; Phase 15 preserved existing APIs and did not add a broad clippy allow sweep.

## Decision

Rationale: CLI command-domain containment is in place, CLI behavior is preserved by full CLI tests and command smokes, affected crates check, and the current verifier now covers both CLI containment and strict structured-input hooks across split modules.

Decision: continue
