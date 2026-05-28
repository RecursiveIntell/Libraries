# P27 Phase Report

## Phase

- Phase ID: 09
- Phase title: Provider path E2E hardening
- Date: 2026-05-05T02:21:01Z

## Scope

- Intended work: prove the supported-local mock provider Plan->Act->Verify path runs end-to-end without cloud credentials, and classify optional local Ollama smoke evidence honestly.
- Issue IDs in scope: `P27-009`.
- Explicit non-goals: no hosted provider keys, no cloud-provider verifier dependency, no native Ollama tool-loop claim, no broad provider runtime expansion, no canonical provider/tool/verification ownership change.

## Files inspected

- `prompts/phases/P27_PHASE_09_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_09_BEFORE_PHASE_10.md`
- `P27_PHASE_PLAN.md`
- `P27_MASTER_ISSUE_MATRIX.md`
- `STATUS.md`
- `scripts/p27_verify.sh`
- `examples/aidens.ollama.toml`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-tool-kit/src/lib.rs`
- `crates/aidens-receipts/src/lib.rs`
- `crates/aidens-integration-tests/tests/phase_08_run_bundle_store.rs`

## Files changed

- `STATUS.md`
- `scripts/p27_verify.sh`
- `scripts/p27_provider_path_smoke.py`
- `crates/aidens-integration-tests/tests/phase_09_provider_path_e2e.rs`
- `handoffs/p27/PHASE_09_REPORT.md`

## Changes made

- Added `scripts/p27_provider_path_smoke.py`.
- The smoke helper runs `aidens-cli agent new`, rewrites the generated task to a read-only README task, runs `aidens-cli agent run`, inspects the persisted receipt store, and fails closed unless the mock provider path produces:
  - `PlanActVerifyLoopV1OutputDisplay.outcome = Success`
  - `AiDENsRunBundleV3`
  - `support.support_tier = supported-local`
  - `provider_route = mock`
  - verified event-log digest
  - run-bundle store `semantic_status = exact_check`
  - non-degraded failure taxonomy
- Added optional local Ollama handling to the smoke helper. It probes `http://localhost:11434/api/tags` and runs `provider-check --config examples/aidens.ollama.toml` only when requested; unavailable Ollama is recorded as an environment prerequisite instead of failing the verifier unless `--require-ollama` is used.
- Wired the provider-path smoke into `scripts/p27_verify.sh` only inside the cargo-capable verifier branch. `P27_SKIP_CARGO=1` continues to skip cargo-backed checks.
- Added an integration test proving the mock provider Plan->Act->Verify path stores an exact `AiDENsRunBundleV3` receipt and can inspect it after reopening the receipt-store root.
- Updated `STATUS.md` to close `P27-009` with evidence-backed limits.

## Commands run

| Command | Result | Log |
|---|---|---|
| `python3 -m py_compile scripts/p27_provider_path_smoke.py` | pass | `target/p27/audit/phase09_py_compile_final.log` |
| `bash -n scripts/p27_verify.sh` | pass | `target/p27/audit/phase09_bash_n_p27_verify.log` |
| `python3 scripts/p27_provider_path_smoke.py . --receipt-out target/p27/audit/phase09_provider_path_smoke_receipt.json` | pass | `target/p27/audit/phase09_provider_path_smoke.log` |
| `python3 scripts/p27_provider_path_smoke.py . --allow-optional-ollama --receipt-out target/p27/audit/phase09_provider_path_smoke_with_ollama_receipt.json` | pass; local Ollama provider-check available and classified as `partial` with `native_tool_loop=false` | `target/p27/audit/phase09_provider_path_smoke_with_ollama.log` |
| `cargo fmt --check` before formatting | failed with formatting diffs only | `target/p27/audit/phase09_cargo_fmt_check_initial.log` |
| `cargo fmt` | pass | `target/p27/audit/phase09_cargo_fmt_after_test_fix.log` |
| `cargo fmt --check` final | pass | `target/p27/audit/phase09_cargo_fmt_check_final.log` |
| `cargo test -p aidens-integration-tests phase_09_mock_plan_act_verify_e2e` initial | failed due to an over-specific test assertion on receipt display shape | `target/p27/audit/phase09_cargo_test_integration_provider_e2e.log` |
| `cargo test -p aidens-integration-tests phase_09_mock_plan_act_verify_e2e` final | pass | `target/p27/audit/phase09_cargo_test_integration_provider_e2e_final.log` |
| `cargo check -p aidens-cli -p aidens-runner -p aidens-provider-kit -p aidens-integration-tests` | pass | `target/p27/audit/phase09_cargo_check_provider_path.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass | `target/p27/audit/phase09_verify_current_skip_cargo.log` |
| `python3 scripts/assert_support_claims.py .` | pass | `target/p27/audit/phase09_assert_support_claims.log` |
| `python3 scripts/assert_p27_current_run_truth.py .` | pass | `target/p27/audit/phase09_assert_p27_current_run_truth.log` |
| `python3 scripts/assert_p27_agents_md_current.py .` | pass | `target/p27/audit/phase09_assert_p27_agents_md_current.log` |

## Evidence emitted

- `target/p27/audit/phase09_provider_path_smoke_receipt.json`
- `target/p27/audit/phase09_provider_path_smoke.log`
- `target/p27/audit/phase09_provider_path_smoke_with_ollama_receipt.json`
- `target/p27/audit/phase09_provider_path_smoke_with_ollama.log`
- `target/p27/audit/phase09_provider_path_smoke_work/inspect-from-store.json`
- `target/p27/audit/phase09_provider_path_smoke_work/run/run-bundle.json`
- `target/p27/audit/phase09_provider_path_smoke_work/run/run-bundle-store-record.json`
- `target/p27/audit/phase09_provider_path_smoke_work/run/plan-act-verify-output.json`
- `target/p27/audit/phase09_provider_path_smoke_work/run/event-log.ndjson`
- `target/p27/audit/phase09_cargo_test_integration_provider_e2e_final.log`
- `target/p27/audit/phase09_cargo_check_provider_path.log`
- `target/p27/audit/phase09_verify_current_skip_cargo.log`
- `target/p27/audit/phase09_assert_support_claims.log`
- `target/p27/audit/phase09_assert_p27_current_run_truth.log`

## 11A semantic impact

- Exact/approx labels touched: added `semantic_status` to the new provider-path smoke receipt.
- Degradation labels touched: the required mock smoke records `exact_check`; optional Ollama records `exact_check` only when the local service and provider-check pass, otherwise it records `degraded_exact_check` as an environment prerequisite.
- Support labels touched: no `SUPPORT_PROFILE.md` claim was widened. `STATUS.md` now records `P27-009` closed for the mock E2E path, with Ollama explicitly optional.
- Proof/check hooks added: CLI smoke and integration test both inspect a persisted `AiDENsRunBundleV3` receipt store after process exit.

## Support profile impact

- No support-tier claim changed in `SUPPORT_PROFILE.md`.
- `STATUS.md` changed to close `P27-009` with the limited claim: mock supported-local Plan->Act->Verify E2E is executable; Ollama remains optional local smoke evidence.
- Local Ollama was available in this environment. The receipt records it as `support_tier=partial`, `native_tool_loop=false`, and not required for verifier success.

## Canonical-owner impact

- No canonical-owner boundary changed.
- Provider smoke receipts are AiDENs-local operator evidence only.
- Canonical provider/tool/verification ownership remains delegated to the sibling owner crates already declared by the existing runtime surfaces, including `llm-tool-runtime` and the `verification-*` crates.

## Issues closed

- `P27-009`: mock provider Plan->Act->Verify path is now tested end-to-end through the CLI and persisted receipt inspection. Optional Ollama smoke is available without becoming a verifier prerequisite.

## New issues / risks

- Ollama remains `partial` and local-environment dependent.
- Native Ollama tool-loop support remains unimplemented and explicitly not claimed.
- The provider-path smoke uses `cargo run -p aidens-cli`; full verifier runs with `P27_SKIP_CARGO=1` intentionally skip it.

## Decision

Rationale: The supported-local mock provider path is executable end-to-end without cloud credentials, durable evidence is emitted and inspectable, optional Ollama is classified without widening support claims, and current verifier truth checks still pass.

Decision: continue
