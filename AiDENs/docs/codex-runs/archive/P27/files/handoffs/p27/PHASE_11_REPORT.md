# P27 Phase Report

## Phase

- Phase ID: 11
- Phase title: Coding agent loop uplift
- Date: 2026-05-05T02:40:11Z

## Scope

- Intended work: connect repo read/search/propose/apply/check receipts into a coherent local coding-agent run path with explicit blocked-write, failed-check, and successful patch+check evidence.
- Issue IDs in scope: `P27-010`, plus the `P27-019` operator UX thread for coding-agent run evidence.
- Explicit non-goals: no Claude Code parity claim, no broad autonomy, no hosted/cloud provider path, no native provider tool loop, no external patch parser expansion, no canonical tool/verification ownership change.

## Files inspected

- `prompts/phases/P27_PHASE_11_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_11_BEFORE_PHASE_12.md`
- `P27_PHASE_PLAN.md`
- `P27_MASTER_ISSUE_MATRIX.md`
- `P27_ACCEPTANCE_GATES.md`
- `STATUS.md`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-tool-kit/src/lib.rs`
- `crates/aidens-permit-kit/src/lib.rs`

## Files changed

- `STATUS.md`
- `crates/aidens-cli/src/lib.rs`
- `handoffs/p27/PHASE_11_REPORT.md`

## Changes made

- `run-coding-agent` now executes `aidens:run-checks:1` after patch proposal/apply.
- `--permit-json` now accepts either one `PermitGrantV1` / approved `ApprovalDecisionV1` or an array of grants/approved decisions, allowing separate scoped write and shell/check permits.
- Coding-agent step status now distinguishes successful checks from `check_failed`.
- Coding-agent reports now include:
  - `receipt_chain`
  - `loop_summary`
  - top-level `semantic_status`
  - patch changed files
  - check success state
  - permit-use receipt IDs
- Coding-agent failure taxonomy now reports:
  - expected side-effect blocks as non-degraded exact evidence,
  - failed checks as `tool-failed` with `degraded=true`,
  - successful patch+check runs as `patch-and-check-loop-succeeded`.
- Added tests for:
  - blocked unapproved write/check side effects,
  - failed check with scoped shell/admin permit,
  - successful patch plus successful `cargo check --workspace` with separate scoped write/check permits.
- Updated `STATUS.md` to close `P27-010` across Phase 10/11 v0 and record Phase 11 evidence.

## Commands run

| Command | Result | Log |
|---|---|---|
| `cargo fmt` | pass | `target/p27/audit/phase11_cargo_fmt.log` |
| `cargo fmt --check` | pass | `target/p27/audit/phase11_cargo_fmt_check.log` |
| `cargo test -p aidens-cli run_coding_agent_` | pass | `target/p27/audit/phase11_cargo_test_cli_run_coding_agent_final.log` |
| CLI successful patch/check smoke | pass | `target/p27/audit/phase11_cli_successful_patch_check_smoke_final.log` |
| `cargo check -p aidens-cli -p aidens-tool-kit -p aidens-runner` | pass | `target/p27/audit/phase11_cargo_check_coding_agent_path.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass | `target/p27/audit/phase11_verify_current_skip_cargo.log` |
| `python3 scripts/assert_support_claims.py .` | pass | `target/p27/audit/phase11_assert_support_claims_final.log` |
| `python3 scripts/assert_p27_current_run_truth.py .` | pass | `target/p27/audit/phase11_assert_p27_current_run_truth_final.log` |
| `python3 scripts/assert_p27_agents_md_current.py .` | pass | `target/p27/audit/phase11_assert_p27_agents_md_current.log` |

An exploratory CLI smoke using `--risk admin` failed before the coding-agent run because the CLI risk parser accepts `shell` for admin-class command permits. It is retained as diagnostic evidence at `target/p27/audit/phase11_permit_admin_probe.log`; the final smoke uses `--risk shell` and passed.

## Evidence emitted

- `target/p27/audit/phase11_cargo_test_cli_run_coding_agent_final.log`
- `target/p27/audit/phase11_cli_successful_patch_check_receipt.json`
- `target/p27/audit/phase11_cli_successful_patch_check_smoke_final.log`
- `target/p27/audit/phase11_cli_successful_patch_check_work/out/coding-agent-report.json`
- `target/p27/audit/phase11_cli_successful_patch_check_work/out/run-bundle.json`
- `target/p27/audit/phase11_cargo_check_coding_agent_path.log`
- `target/p27/audit/phase11_verify_current_skip_cargo.log`
- `target/p27/audit/phase11_assert_support_claims_final.log`
- `target/p27/audit/phase11_assert_p27_current_run_truth_final.log`
- `target/p27/audit/phase11_assert_p27_agents_md_current.log`

## 11A semantic impact

- Exact/approx labels touched: `coding-agent-report.json` now emits top-level `semantic_status`.
- Degradation labels touched: failed check runs emit `degraded_exact_check`; blocked side-effect runs and successful patch+check runs emit `exact_check`.
- Support labels touched: no `SUPPORT_PROFILE.md` support-tier claim was widened. `STATUS.md` records `P27-010` closure across Phase 10/11 v0.
- Proof/check hooks added: tests and CLI smoke prove blocked writes/checks, degraded failed checks, and successful patch+check receipts.

## Support profile impact

- No support-tier claim changed in `SUPPORT_PROFILE.md`.
- The existing supported-local coding-agent claim is narrowed by evidence: local tools only, permit-gated patch/check path only, no cloud/native provider-loop widening.

## Canonical-owner impact

- No canonical-owner boundary changed.
- Coding-agent reports are AiDENs-local operator evidence.
- Canonical tool receipt ownership remains delegated to `llm-tool-runtime`; check/verification semantics remain delegated to the `verification-*` sibling crates.

## Issues closed

- `P27-010`: closed across Phase 10/11 v0. Patch application is hardened, and the coding-agent path now ties read/search/propose/apply/check receipts into one evidence surface.
- `P27-019`: partially advanced for coding-agent operator UX via `receipt_chain` and `loop_summary`; broader UX remains a later-phase/final-report matter.

## New issues / risks

- `run-coding-agent` still emits `AiDENsRunBundleV2`; Phase 08’s V3 durable receipt store is not yet wired into this older command path.
- The check command is fixed to `cargo check --workspace` in this v0 path.
- This remains supported-local only and does not claim hosted provider execution or native provider tool-loop parity.

## Decision

Rationale: The local coding-agent path now exposes coherent patch/check evidence with permits, receipts, failed-check degradation, and successful patch+check proof while preserving canonical ownership boundaries.

Decision: continue
