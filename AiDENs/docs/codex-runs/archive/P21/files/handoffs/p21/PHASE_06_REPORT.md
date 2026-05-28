# P21 Phase 06 Report: Agency Governance v0.2

## Scope

Phase 06 focused only on agency governance v0.2:

- expanded agency eval coverage from 10 to 21 JSONL cases;
- kept agency semantics in `aidens-agency-kit` as AiDENs boundary policy, not canonical ethics or memory truth;
- preserved runner enforcement through the real `aidens-runner` runtime path;
- added runtime tests proving final-output memory personalization and repeated nudges cannot bypass agency receipts/gates.

## Files Changed

- `crates/aidens-agency-kit/src/lib.rs`
  - added v0.2 eval surface mapping for financial, medical, legal, reversibility, delegated high-impact, vulnerability, external/tool conflict, and memory influence surfaces;
  - added runner final-output personalization signal detection that produces `MemoryInfluenceTraceV1` through the existing agency report path;
  - added high-impact professional-review deferral for medical/legal recommendations;
  - added gates/blocked behaviors for all-in financial pressure, tool-origin conflict disclosure, delegated high-impact merge, emotional dependence, retention hooks, minor vulnerability pressure, and irreversible action without disclosure.
- `crates/aidens-runner/tests/phase_06_agency_v02.rs`
  - added runtime tests for memory-personalized output receipts/gating;
  - added runtime tests for repeated-nudge budget receipts/gating across repeated runner turns.
- `evals/p20_agency_eval_cases.jsonl`
  - expanded to 21 eval cases across 19 surfaces and 22 receipt kinds.
- `scripts/p20_2_validate_agency_cases.py`
  - raised default minimum eval case count to 20 for the expanded P21 agency gate.
- `target/p21/phase06/*`
  - proof logs and invariant logs.
- `handoffs/p21/PHASE_06_REPORT.md`
  - this report.

## Invariant Checks

Pre-change checks:

- `bash scripts/assert_stack_paths.sh . | tee target/p21/phase06/invariant_stack_paths.before.log` -> pass, no output.
- `bash scripts/assert_no_local_substitute_dependencies.sh | tee target/p21/phase06/invariant_no_local_substitute_dependencies.before.log` -> pass, `PASS: no local substitute dependency red flags detected.`
- `bash scripts/assert_compat_is_finite.sh . | tee target/p21/phase06/invariant_compat_is_finite.before.log` -> pass, no output.
- `bash scripts/assert_no_shadow_truth.sh . | tee target/p21/phase06/invariant_no_shadow_truth.before.log` -> pass, no output.
- `bash scripts/assert_no_scaffold_promoted.sh . | tee target/p21/phase06/invariant_no_scaffold_promoted.before.log` -> pass, `No scaffold promotion patterns found.`
- `bash scripts/p21_verify.sh | tee target/p21/phase06/p21_verify.before.log` -> pass.

Post-change checks:

- `bash scripts/assert_stack_paths.sh . | tee target/p21/phase06/invariant_stack_paths.after.log` -> pass, no output.
- `bash scripts/assert_no_local_substitute_dependencies.sh | tee target/p21/phase06/invariant_no_local_substitute_dependencies.after.log` -> pass, `PASS: no local substitute dependency red flags detected.`
- `bash scripts/assert_compat_is_finite.sh . | tee target/p21/phase06/invariant_compat_is_finite.after.log` -> pass, no output.
- `bash scripts/assert_no_shadow_truth.sh . | tee target/p21/phase06/invariant_no_shadow_truth.after.log` -> pass, no output.
- `bash scripts/assert_no_scaffold_promoted.sh . | tee target/p21/phase06/invariant_no_scaffold_promoted.after.log` -> pass, `No scaffold promotion patterns found.`
- `bash scripts/p21_verify.sh | tee target/p21/phase06/p21_verify.log` -> pass, including `Agency eval validation OK: 21 cases, 19 surfaces, 22 receipt kinds`.

Invariant notes:

- AiDENs still owns only the boundary agency policy/orchestration layer.
- No memory/evidence/kernel/repair/verification/federation/mechanism truth was introduced.
- Runner agency enforcement remains receipt-bearing through `AgencyPolicyReportV1`, `agency_receipt_ids`, stop rules, and durable `agency-policy-report-v1` records.
- No tests, fixtures, evals, or scanners were deleted. The agency eval fixture was expanded.

## Commands Run

Inspection/context commands:

- `sed`/`rg` reads over `docs/p21/P21_AGENCY_GOVERNANCE_V02.md`, `evals/p20_agency_eval_cases.jsonl`, `evals/p21_agency_eval_cases.jsonl`, `crates/aidens-agency-kit/src/lib.rs`, `crates/aidens-runner/src/lib.rs`, `crates/aidens-runner/tests/phase_08_agency_gate.rs`, `crates/aidens-cli/src/lib.rs`, and config examples.
- `git status --short` showed the wider parent repository has unrelated dirty/untracked state; Phase 06 work stayed within the files listed above.

Validation/proof commands:

- `python3 scripts/p20_2_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl --min-cases 20 | tee target/p21/phase06/agency_eval_validation.log` -> pass, `Agency eval validation OK: 21 cases, 19 surfaces, 22 receipt kinds`.
- `cargo fmt --all` -> pass, formatted.
- `cargo fmt --all --check | tee target/p21/phase06/fmt_check.log` -> pass.
- `cargo test -p aidens-agency-kit --all-targets --all-features | tee target/p21/phase06/agency_kit_tests.log` -> pass, 2 tests passed.
- `cargo test -p aidens-runner --test phase_06_agency_v02 --all-features | tee target/p21/phase06/runner_phase06_agency_v02_tests.log` -> pass, 2 tests passed.
- `cargo test -p aidens-runner --test phase_08_agency_gate --all-features | tee target/p21/phase06/runner_phase08_agency_gate_tests.log` -> pass, 2 tests passed.
- `cargo test -p aidens-runner --all-targets --all-features | tee target/p21/phase06/runner_all_tests.log` -> pass, 22 runner tests passed.
- `cargo test -p aidens-integration-tests --test phase_09_reference_hostile_tests agency_eval_cases_match_decision_semantics_and_required_receipts --all-features | tee target/p21/phase06/integration_agency_eval_tests.log` -> pass, 1 hostile/reference agency eval test passed.
- `cargo check -p aidens-agency-kit -p aidens-runner --all-targets --all-features | tee target/p21/phase06/check_touched_crates.log` -> pass.
- `cargo clippy -p aidens-agency-kit -p aidens-runner --all-targets --all-features -- -D warnings | tee target/p21/phase06/clippy_touched_crates.log` -> pass.

Superseded command:

- `cargo test -p aidens-runner --all-targets --all-features agency | tee target/p21/phase06/runner_agency_tests.log` compiled runner tests but matched 0 tests due the name filter. It was superseded by the explicit runner test-target commands above.

## Required Proof

- Expanded eval file validates: yes, `target/p21/phase06/agency_eval_validation.log`.
- Agency tests pass: yes, `target/p21/phase06/agency_kit_tests.log` and `target/p21/phase06/integration_agency_eval_tests.log`.
- Runner cannot bypass agency when enabled: yes.
  - high-impact final output gate: existing `phase_08_agency_gate` test target passes;
  - tool-output influence receipts: existing `phase_08_agency_gate` test target passes;
  - memory-personalized final output receipts/gate: new `phase_06_agency_v02` test passes;
  - repeated-nudge budget receipts/gate: new `phase_06_agency_v02` test passes.

## Outcome

Phase 06 passed. Stop here and wait for the Phase 07 operator injection.
