# P27 Phase Report

## Phase

- Phase ID: 10
- Phase title: Patch engine hardening v0
- Date: 2026-05-05T02:33:55Z

## Scope

- Intended work: harden the local patch-apply path so invalid and ambiguous patches fail closed with evidence before mutation.
- Issue IDs in scope: `P27-010`.
- Explicit non-goals: no broad patch parser, no unsafe fuzzy patch application, no cloud/autonomy widening, no canonical tool/verification ownership change, no Phase 11 coding-agent loop uplift.

## Files inspected

- `prompts/phases/P27_PHASE_10_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_10_BEFORE_PHASE_11.md`
- `P27_PHASE_PLAN.md`
- `P27_MASTER_ISSUE_MATRIX.md`
- `P27_ACCEPTANCE_GATES.md`
- `STATUS.md`
- `crates/aidens-tool-kit/src/lib.rs`
- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`

## Files changed

- `STATUS.md`
- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-tool-kit/src/lib.rs`
- `handoffs/p27/PHASE_10_REPORT.md`

## Changes made

- Extended `PatchApplyReportV1` with `changed_files`, `dry_run_checked`, `semantic_status`, `failure_kind`, and `rollback_advice`.
- Added `PatchApplyReportV1::checked` for check-only validation without mutation.
- Added `PatchApplyReportV1::denied_with_details` for failed-closed invalid/ambiguous patch receipts.
- Added `ToolInvocationReportV1::complete_failure_with_output` so failed tool invocations can carry typed failure output evidence.
- Hardened `patch_apply` to parse, sandbox-resolve, read, and validate all replacements before any write occurs.
- Added `check_only` / `dry_run` support to `aidens:patch-apply:1`.
- Rejected duplicate file targets, add-only patches without removal context, missing old-path headers, missing context, and ambiguous repeated context before mutation.
- Surfaced receipt reason codes in `ToolInvocationError` display output for operator-visible CLI failures.
- Added focused tests for successful permitted apply, check-only no-mutation validation, and ambiguous-diff failed-closed receipts.
- Updated `STATUS.md` to close `P27-010` as Phase 10 v0.

## Commands run

| Command | Result | Log |
|---|---|---|
| `cargo fmt` | pass | `target/p27/audit/phase10_cargo_fmt_after_cli_reason.log` |
| `cargo fmt --check` | pass | `target/p27/audit/phase10_cargo_fmt_check_final2.log` |
| `cargo test -p aidens-tool-kit p10_patch_apply` | pass | `target/p27/audit/phase10_cargo_test_tool_kit_patch_apply_final.log` |
| `cargo test -p aidens-contracts p10_coding_artifact_constructors` | pass | `target/p27/audit/phase10_cargo_test_contracts_coding_artifacts.log` |
| `cargo test -p aidens-runner p10_runner_can_apply_patch_with_scoped_permit` | pass | `target/p27/audit/phase10_cargo_test_runner_p10_scoped_patch.log` |
| `cargo test -p aidens-runner patch_apply` | pass | `target/p27/audit/phase10_cargo_test_runner_patch_apply.log` |
| `cargo check -p aidens-contracts -p aidens-tool-kit -p aidens-cli -p aidens-runner` | pass | `target/p27/audit/phase10_cargo_check_patch_path_final.log` |
| CLI ambiguous patch smoke with scoped permit | pass; patch failed closed and file remained unchanged | `target/p27/audit/phase10_cli_ambiguous_patch_receipt_final.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass | `target/p27/audit/phase10_verify_current_skip_cargo_final.log` |
| `python3 scripts/assert_support_claims.py .` | pass | `target/p27/audit/phase10_assert_support_claims_final.log` |
| `python3 scripts/assert_p27_current_run_truth.py .` | pass | `target/p27/audit/phase10_assert_p27_current_run_truth_final.log` |

An exploratory CLI smoke using `--diff <value>` failed at argument parsing because the diff begins with `---`; it did not exercise the patch executor and is retained only as diagnostic evidence at `target/p27/audit/phase10_cli_ambiguous_patch_receipt.log`. The final smoke used `--diff=<value>` and passed the intended validation.

## Evidence emitted

- `target/p27/audit/phase10_cargo_test_tool_kit_patch_apply_final.log`
- `target/p27/audit/phase10_cargo_test_contracts_coding_artifacts.log`
- `target/p27/audit/phase10_cargo_test_runner_p10_scoped_patch.log`
- `target/p27/audit/phase10_cargo_test_runner_patch_apply.log`
- `target/p27/audit/phase10_cargo_check_patch_path_final.log`
- `target/p27/audit/phase10_cli_ambiguous_patch_receipt.json`
- `target/p27/audit/phase10_cli_ambiguous_patch_receipt_final.log`
- `target/p27/audit/phase10_verify_current_skip_cargo_final.log`
- `target/p27/audit/phase10_assert_support_claims_final.log`
- `target/p27/audit/phase10_assert_p27_current_run_truth_final.log`

## 11A semantic impact

- Exact/approx labels touched: `PatchApplyReportV1.semantic_status` now records `exact_check` for validated apply/check-only receipts and `failed_exact_check` for failed-closed invalid/ambiguous receipts.
- Degradation labels touched: failed patch attempts are not approximate successes; they are explicit failed exact checks.
- Support labels touched: no `SUPPORT_PROFILE.md` support-tier claim was widened. `STATUS.md` records `P27-010` closed for local patch engine v0 hardening only.
- Proof/check hooks added: tests and CLI smoke prove no mutation on ambiguous patch failure, check-only validation without mutation, and successful permit-gated apply with changed-file receipt evidence.

## Support profile impact

- No support-tier claim changed in `SUPPORT_PROFILE.md`.
- `STATUS.md` changed to close `P27-010` with the limited claim that patch-apply v0 now preflights, supports check-only, and emits failed-closed receipts for invalid/ambiguous diffs.

## Canonical-owner impact

- No canonical-owner boundary changed.
- Patch receipts remain AiDENs-local operator/tool evidence.
- Canonical tool receipt semantics remain delegated to `llm-tool-runtime`; verification/control semantics remain delegated to the `verification-*` sibling crates.

## Issues closed

- `P27-010`: invalid and ambiguous patch cases now fail closed before mutation, with evidence-bearing tool receipts and operator-visible reason codes.

## New issues / risks

- The patch parser remains intentionally narrow. Multi-hunk same-file patches and add-only/create-file patches are rejected rather than guessed.
- CLI `coding patch-apply` does not expose check-only flags yet; check-only is available through the tool invocation input and is covered by tool-kit tests.
- Phase 11 still needs the broader coding-agent patch/check loop uplift.

## Decision

Rationale: Phase 10 acceptance is met at v0: patch apply is permit-gated, validates before writes, supports check-only, emits changed-file and rollback-advice receipt fields, and fails closed on ambiguous patches with evidence.

Decision: continue
