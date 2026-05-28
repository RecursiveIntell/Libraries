# P27 Phase Report

## Phase

- Phase ID: 14
- Phase title: Megafile containment: contracts first
- Date: 2026-05-05T08:38:12Z

## Scope

- Intended work: split `aidens-contracts/src/lib.rs` into internal domain modules behind a stable re-export facade.
- Issue IDs in scope: `P27-006`, with `P27-013` observed for touched high-argument constructors.
- Explicit non-goals: no semantic contract rewrites, no crate explosion, no public API rename, no canonical-owner boundary change, no support-tier widening, no broad `too_many_arguments` cleanup beyond containment fallout.

## Files inspected

- `prompts/phases/P27_PHASE_14_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_14_BEFORE_PHASE_15.md`
- `P27_PHASE_PLAN.md`
- `P27_MASTER_ISSUE_MATRIX.md`
- `P27_ACCEPTANCE_GATES.md`
- `P27_11A_ALIGNMENT.md`
- `STATUS.md`
- `SUPPORT_PROFILE.md`
- `SOURCE_BASIS.md`
- `crates/aidens-contracts/Cargo.toml`
- `crates/aidens-contracts/src/lib.rs`
- Downstream `aidens_contracts` users under `crates/`

## Files changed

- `STATUS.md`
- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-contracts/src/app_status.rs`
- `crates/aidens-contracts/src/schema_catalog.rs`
- `crates/aidens-contracts/src/tests.rs`
- `scripts/assert_p27_contracts_megafile_containment.py`
- `scripts/p27_verify.sh`
- `handoffs/p27/PHASE_14_REPORT.md`

## Changes made

- Split `aidens-contracts/src/lib.rs` mechanically:
  - `app_status.rs`: app plan, doctor/status, support posture, memory/report-level DTOs.
  - `schema_catalog.rs`: display digest, boundary repair, schema registry/manifest/compatibility/reference-fixture DTOs and schema generation helpers.
  - `tests.rs`: existing contracts unit tests.
- Kept `lib.rs` as the stable facade with `pub use app_status::*;` and `pub use schema_catalog::*;`.
- Kept canonical re-exports and existing root public names stable for downstream crates.
- Made three existing sorting helpers `pub(crate)` because root contract DTOs and the schema module both use them after the split.
- Added `scripts/assert_p27_contracts_megafile_containment.py` and wired it into `scripts/p27_verify.sh`.
- Updated `STATUS.md` to record partial closure of `P27-006`; CLI megafile containment remains Phase 15 scope.

## Commands run

| Command | Result | Log |
|---|---|---|
| `cargo fmt` | pass | `target/p27/audit/phase14_cargo_fmt.log` |
| `cargo check -p aidens-contracts` | fail, fixed moved helper visibility/imports | `target/p27/audit/phase14_cargo_check_contracts.log` |
| `cargo test -p aidens-contracts` | fail, same pre-fix helper visibility/import issue | `target/p27/audit/phase14_cargo_test_contracts.log` |
| `cargo fmt` | pass | `target/p27/audit/phase14_cargo_fmt_after_helper_import.log` |
| `cargo check -p aidens-contracts` | pass | `target/p27/audit/phase14_cargo_check_contracts_after_helper_import.log` |
| `cargo test -p aidens-contracts` | pass | `target/p27/audit/phase14_cargo_test_contracts_after_helper_import.log` |
| `python3 -m py_compile scripts/assert_p27_contracts_megafile_containment.py` | pass | `target/p27/audit/phase14_py_compile_contracts_containment.log` |
| `python3 scripts/assert_p27_contracts_megafile_containment.py .` | pass | `target/p27/audit/phase14_assert_contracts_megafile_containment.log` |
| `cargo fmt --check` | pass | `target/p27/audit/phase14_cargo_fmt_check.log` |
| `cargo check -p aidens-contracts -p aidens-boundary-kit -p aidens-runner -p aidens-cli -p aidens-app-kit -p aidens` | pass | `target/p27/audit/phase14_cargo_check_affected_crates.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass | `target/p27/audit/phase14_verify_current_skip_cargo.log` |
| `python3 scripts/assert_support_claims.py .` | pass | `target/p27/audit/phase14_assert_support_claims.log` |
| `python3 scripts/assert_p27_current_run_truth.py .` | pass | `target/p27/audit/phase14_assert_p27_current_run_truth.log` |
| `python3 scripts/assert_p27_agents_md_current.py .` | pass | `target/p27/audit/phase14_assert_p27_agents_md_current.log` |

## Evidence emitted

- `target/p27/audit/phase14_contracts_line_counts.log`
- `target/p27/audit/phase14_assert_contracts_megafile_containment.log`
- `target/p27/audit/phase14_cargo_check_contracts_after_helper_import.log`
- `target/p27/audit/phase14_cargo_test_contracts_after_helper_import.log`
- `target/p27/audit/phase14_cargo_check_affected_crates.log`
- `target/p27/audit/phase14_verify_current_skip_cargo.log`
- `target/p27/audit/phase14_assert_support_claims.log`
- `target/p27/audit/phase14_assert_p27_current_run_truth.log`
- `target/p27/audit/phase14_assert_p27_agents_md_current.log`

## Containment result

- Before: `crates/aidens-contracts/src/lib.rs` was 10,971 lines.
- After:
  - `crates/aidens-contracts/src/lib.rs`: 6,776 lines.
  - `crates/aidens-contracts/src/app_status.rs`: 542 lines.
  - `crates/aidens-contracts/src/schema_catalog.rs`: 1,617 lines.
  - `crates/aidens-contracts/src/tests.rs`: 2,013 lines.
- The new verifier guard enforces the module split and a 7,000-line maximum for the contracts facade.

## 11A semantic impact

- Exact/approx labels touched: none.
- Degradation labels touched: none.
- Support labels touched: no `SUPPORT_PROFILE.md` support-tier claim changed. `STATUS.md` records `P27-006` as partially closed for contracts containment only.
- Proof/check hooks added: static contracts containment guard plus contracts and affected-crate compile/test evidence.

## Support profile impact

- No support-tier claim changed in `SUPPORT_PROFILE.md`.
- No supported-local capability was widened.
- `STATUS.md` now records the structural containment evidence and leaves CLI megafile work for Phase 15.

## Canonical-owner impact

- No canonical-owner boundary changed.
- Existing canonical owners remain delegated to sibling crates; this phase only reorganized AiDENs-local contract facade code.

## Issues closed

- `P27-006`: partially closed for `aidens-contracts` containment. Remaining CLI megafile containment is Phase 15 scope.

## New issues / risks

- `aidens-cli/src/lib.rs` remains a megafile and is explicitly deferred to Phase 15.
- Two existing `#[allow(clippy::too_many_arguments)]` constructors in run-bundle DTOs were preserved unchanged; broader `P27-013` cleanup remains later-scope unless touched.

## Decision

Rationale: contracts megafile containment is in place, public re-export compatibility is preserved by affected-crate checks, contracts tests pass, and the verifier now contains a regression guard for the contracts facade split.

Decision: continue
