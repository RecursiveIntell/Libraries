# P21 Phase 05 Report — Provider/Tool Capability Truth

Date: 2026-05-01

## Scope

Phase 05 focused only on executable provider/tool capability truth.

- `provider-check` now emits machine-readable JSON and exposes `configured`, `executable`, `chat_completion`, `native_tool_loop`, `structured_output`, `streaming`, `degraded`, `backend_status`, `support_label`, and `reason_codes`.
- `tools inspect` now exposes top-level `requires_permit`, `provider_schema_tool_ids`, and per-tool `tool_capabilities` alongside declared/registered/executable/exposed/hidden/blocked lists.
- Unsupported cloud providers remain unavailable/deferred even when configured with an API key.
- No cloud provider execution or native provider tool-loop support was added.

## Files Changed

- `crates/aidens-cli/src/lib.rs`
- `scripts/check_examples.sh`
- `handoffs/p21/PHASE_05_REPORT.md`
- Proof artifacts under `target/p21/phase05/`

## Baseline Findings

Baseline logs are retained:

- `provider_check_mock.before.log`
- `provider_check_openai_unavailable.before.log`
- `provider_check_ollama.before.log`
- `tools_inspect_mock.before.log`

Observed gaps before repair:

- `provider-check` was text output rather than machine-readable JSON.
- `provider-check` did not expose `structured_output`.
- `tools inspect` had permit/schema truth inside nested exposure data, but not as top-level operator fields.

## Operator Proof

Logs are in `target/p21/phase05/`.

| Command | Result | Log |
|---|---:|---|
| `cargo run -p aidens-cli -- provider-check --config examples/aidens.mock.toml` | PASS; JSON shows mock executable, `native_tool_loop=false`, `structured_output=true`, support label `fixture-supported-not-cloud` | `provider_check_mock.log` |
| `cargo run -p aidens-cli -- provider-check --config examples/aidens.openai-unavailable.toml` | PASS; JSON shows OpenAI unavailable, not executable, no native loop, no structured output | `provider_check_openai_unavailable.log` |
| `cargo run -p aidens-cli -- provider-check --config examples/aidens.ollama.toml` | PASS; JSON shows Ollama partial local chat, no native loop, no structured output | `provider_check_ollama.log` |
| `cargo run -p aidens-cli -- tools inspect --config examples/aidens.mock.toml` | PASS; JSON shows declared/registered/executable/exposed/blocked/permit/schema truth | `tools_inspect_mock.log` |
| `cargo run -q -p aidens-cli -- provider-check --config examples/aidens.mock.toml \| jq ...` | PASS; parsed fields prove machine-readable provider truth | `provider_check_mock_jq.log` |
| `cargo run -q -p aidens-cli -- provider-check --config examples/aidens.openai-unavailable.toml \| jq ...` | PASS; parsed unsupported provider truth | `provider_check_openai_unavailable_jq.log` |
| `cargo run -q -p aidens-cli -- tools inspect --config examples/aidens.mock.toml \| jq ...` | PASS; parsed top-level tool truth | `tools_inspect_mock_jq.log` |

## Test Proof

| Command | Result | Log |
|---|---:|---|
| `cargo test -p aidens-cli provider_check_reports_missing_api_key_without_claiming_executable` | PASS | `cargo_test_cli_provider_missing_key.log` |
| `cargo test -p aidens-cli provider_check_reports_configured_cloud_providers_as_unavailable` | PASS; would fail if OpenAI/OpenRouter/Anthropic/OpenAI-compatible were promoted to executable | `cargo_test_cli_configured_cloud_unavailable.log` |
| `cargo test -p aidens-cli provider_route_does_not_claim_native_when_backend_is_unavailable` | PASS | `cargo_test_cli_no_native_unavailable.log` |
| `cargo test -p aidens-cli provider_check_reports_ollama_chat_without_native_tool_loop` | PASS | `cargo_test_cli_ollama_no_native.log` |
| `cargo test -p aidens-cli inspect_tools_reports_registered_vs_executable` | PASS | `cargo_test_cli_tools_inspect_truth.log` |
| `cargo test -p aidens-provider-kit --all-targets --all-features` | PASS | `cargo_test_provider_kit.log` |
| `cargo test -p aidens-tool-kit --all-targets --all-features` | PASS | `cargo_test_tool_kit.log` |
| `cargo test -p aidens-cli --all-targets --all-features` | PASS | `cargo_test_cli.log` |
| `cargo test --workspace --all-targets --all-features` | PASS | `cargo_test_workspace.log` |
| `bash scripts/check_examples.sh` | PASS; example smoke updated to assert JSON provider-check truth | `check_examples.log` |

## Build Proof

| Command | Result | Log |
|---|---:|---|
| `cargo fmt --all --check` | PASS | `cargo_fmt_check.log` |
| `cargo check -p aidens-cli --all-targets --all-features` | PASS | `cargo_check_cli.after.log` |
| `cargo check --workspace --all-targets --all-features` | PASS | `cargo_check_workspace.log` |
| `cargo clippy -p aidens-provider-kit -p aidens-tool-kit -p aidens-cli --all-targets --all-features -- -D warnings` | PASS | `cargo_clippy_touched.log` |

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
| `python3 scripts/p20_scan_aidens.py --root . --out target/p21/phase05/p20-scan --require-phase-reports-through 0 --fail-on-blocking` | PASS; 0 blocking findings, 21 warnings retained as inventory | `p20_scan.after.log`, `p20-scan/p20_scan.md` |

## Capability Truth Notes

- `mock` remains the only fixture-supported executable provider with structured output.
- `ollama` remains partial local chat only; `native_tool_loop=false` and `structured_output=false`.
- `openai`, `openrouter`, `anthropic`, and `openai-compatible` remain `deferred/unavailable`; configured API keys do not make them executable.
- Tool inspection shows safe read/search/stat/propose tools exposed, patch apply/run checks blocked by permit, and shell/network/memory/schedule declarations hidden because they are not registered in the safe coding profile.
- Permit-required tools are surfaced explicitly and approval requests remain receipt-bearing through the existing tool exposure object.

## Phase Boundary

Phase 05 is complete. Stop here and wait for the operator's next global and phase-specific injection before starting Phase 06.
