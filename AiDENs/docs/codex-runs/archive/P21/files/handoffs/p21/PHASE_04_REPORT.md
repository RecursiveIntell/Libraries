# P21 Phase 04 Report — Profile And Plan Usability

Date: 2026-05-01

## Scope

Phase 04 focused only on profile/plan usability and truthfulness.

- `aidens-plan-kit` now owns bounded execution-plan assembly only.
- Profile surfaces report supported/partial/deferred status explicitly.
- `aidens plan validate` and `aidens plan compile` accept normal AiDENs configs and the existing test-agent fixture config by deriving an executable mock AiDENs config.
- Unknown `profile_id` values now fail explicitly instead of silently falling back to another profile.

No memory, evidence, kernel, repair, verification, federation, or mechanism truth was added to AiDENs.

## Files Changed

- `Cargo.lock`
- `README.md`
- `STATUS.md`
- `crates/aidens-app-kit/Cargo.toml`
- `crates/aidens-app-kit/src/lib.rs`
- `crates/aidens-cli/Cargo.toml`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-plan-kit/Cargo.toml`
- `crates/aidens-plan-kit/src/lib.rs`
- `docs/CURRENT_AIDENS_AUDIT.md`
- `docs/p20/DOCS_CODE_TRUTH_REPORT.md`
- `scripts/assert_no_scaffold_promoted.sh`
- `scripts/p20_generate_audit_bundle.sh`
- `scripts/p20_scan_aidens.py`
- `handoffs/p21/PHASE_04_REPORT.md`
- Proof artifacts under `target/p21/phase04/`

## Operator Proof

Logs are in `target/p21/phase04/`.

| Command | Result | Log |
|---|---:|---|
| `cargo run -p aidens-cli -- profile list` | PASS; reports `chat-only` and `coding-agent` supported, `memory-agent` partial/proof-only, `autonomous-daemon` partial/safe-mode, `research-workbench` deferred/example-only | `profile_list.log` |
| `cargo run -p aidens-cli -- profile explain coding-agent` | PASS; shows supported status and permit-gated side-effect risks | `profile_explain_coding_agent.log` |
| `cargo run -p aidens-cli -- profile explain research-workbench` | PASS; shows deferred/example-only and not complete | `profile_explain_research_workbench.log` |
| `cargo run -p aidens-cli -- plan validate --config fixtures/test-agent/basic-agent.toml` | PASS; validates derived mock config from test-agent fixture source | `plan_validate_basic_agent.log` |
| `cargo run -p aidens-cli -- plan compile --config fixtures/test-agent/basic-agent.toml --out target/p21/phase04/basic-agent.plan.json` | PASS; compiled plan emitted | `plan_compile_basic_agent.log`, `basic-agent.plan.json` |
| `jq '.plan.profile_id, .provider_route.route_label, .config_apply_receipt.reason_codes, (.parity_report.mismatches // [])' target/p21/phase04/basic-agent.plan.json` | PASS; `coding-agent`, `mock`, `plan-kit:execution-plan-assembly-only`, no parity mismatches | `compiled_plan_inspect.log` |
| `cargo run -p aidens-cli -- plan validate --config examples/aidens.mock.toml` | PASS | `plan_validate_examples_mock.log` |
| `cargo run -p aidens-cli -- plan compile --config examples/aidens.mock.toml --out target/p21/phase04/examples-mock.plan.json` | PASS | `plan_compile_examples_mock.log` |

Baseline logs before repair are retained:

- `profile_list.before.log`
- `profile_explain_coding_agent.before.log`
- `profile_explain_research_workbench.before.log`
- `plan_validate_basic_agent.before.log` failed on missing `app_id` because the command did not understand test-agent source configs.
- `plan_compile_basic_agent.before.log` failed on missing `app_id` for the same reason.

One intermediate test command used invalid Cargo filter syntax and failed before execution; the corrected tests passed:

- Failed syntax log: `cargo_test_cli_phase04.log`
- Corrected logs: `cargo_test_cli_profile_status.log`, `cargo_test_cli_plan_commands.log`

## Build And Test Proof

| Command | Result | Log |
|---|---:|---|
| `cargo fmt --all --check` | PASS | `cargo_fmt_check.log` |
| `cargo check -p aidens-plan-kit -p aidens-app-kit -p aidens-cli --all-targets --all-features` | PASS | `cargo_check_touched.log` |
| `cargo test -p aidens-plan-kit -p aidens-app-kit -p aidens-cli --all-targets --all-features` | PASS; includes plan assembly, profile status, unknown-profile rejection, and plan command tests | `cargo_test_touched.log` |
| `cargo clippy -p aidens-plan-kit -p aidens-app-kit -p aidens-cli --all-targets --all-features -- -D warnings` | PASS | `cargo_clippy_touched.log` |
| `cargo check --workspace --all-targets --all-features` | PASS | `cargo_check_workspace.log` |

## Invariant Checks

| Check | Result | Log |
|---|---:|---|
| `bash scripts/assert_stack_paths.sh .` | PASS | `invariant_stack_paths.after.log` |
| `bash scripts/assert_no_local_substitute_dependencies.sh` | PASS | `invariant_no_local_substitute_dependencies.after.log` |
| `bash scripts/assert_compat_is_finite.sh .` | PASS | `invariant_compat_is_finite.after.log` |
| `bash scripts/assert_no_shadow_truth.sh .` | PASS | `invariant_no_shadow_truth.after.log` |
| `bash scripts/assert_no_scaffold_promoted.sh .` | PASS | `invariant_no_scaffold_promoted.after.log` |
| `bash scripts/assert_no_fake_completion.sh .` | PASS | `invariant_no_fake_completion.after.log` |
| `bash scripts/p21_verify.sh` | PASS | `p21_verify.after.log` |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p21/phase04/p20-scan --require-phase-reports-through 0 --fail-on-blocking` | PASS; 0 blocking findings, 21 warnings retained as inventory | `p20_scan.after.log`, `p20-scan/p20_scan.md` |

## Invariant Notes

- `aidens-plan-kit` depends only on `aidens-contracts` and `thiserror`; it does not import canonical stack crates or own canonical truth.
- `aidens-app-kit` and `aidens-cli` call `aidens-plan-kit` for final app-layer plan assembly.
- Test-agent plan source support derives an executable mock config from the existing fixture; it does not duplicate or bypass the real runner path.
- Scaffold profile crates remain scaffold-only/deferred in `STATUS.md`, doctor scaffold reports, and scanner policy.
- `aidens-plan-kit` is no longer marked scaffold-only because it now has tested, bounded behavior; docs and scanners were updated to keep that truth explicit.
- Unknown config `profile_id` values now fail with `unknown AiDENs profile` instead of silently falling back.

## Deferred / Unsupported

- Memory-agent remains partial/proof-only; canonical memory crates own memory truth.
- Autonomous daemon remains partial/safe-mode.
- Research workbench remains deferred/example-only.
- Cloud provider execution and native provider tool loops remain unavailable unless later phases add executable proof.

## Phase Boundary

Phase 04 is complete. Stop here and wait for the operator's next global and phase-specific injection before starting Phase 05.
