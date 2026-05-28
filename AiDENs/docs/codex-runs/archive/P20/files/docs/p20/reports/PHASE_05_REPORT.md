# P20 Phase 05 Report - Provider Capability Honesty

Phase: `05`
Scope: provider capability truth
Result: `PASS`

## Operator Injection

Proceed to Phase 05 only.

Focus: provider capability honesty.

A provider is supported only if executable and tested. Native tool calling is false unless a native tool-call loop is implemented and tested.

Fallback is not support. Mock is not cloud support. Text completion is not native tool use.

Create/update provider capability matrix and tests/doctor output.

## Files Changed

- `crates/aidens-provider-kit/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-testkit/src/lib.rs`
- `tests/fixtures/p02/provider_backend_matrix_v1.json`
- `tests/fixtures/p02/provider_route_receipt_v2.json`
- `docs/p20/PROVIDER_CAPABILITY_MATRIX.md`
- `docs/p20/PROVIDER_CAPABILITY_MATRIX.json`
- `docs/p20/P20_PROVIDER_CAPABILITY_POLICY.md`
- `docs/p20/P20_PROVIDER_CAPABILITY_MATRIX_TEMPLATE.md`
- `docs/p20/DOCS_CODE_TRUTH_REPORT.md`
- `docs/p20/OPERATOR_QUICKSTART.md`
- `docs/P20_PROVIDER_CAPABILITY_POLICY.md`
- `docs/P20_PROVIDER_CAPABILITY_MATRIX_TEMPLATE.md`
- `docs/PROVIDER_INTEGRATION_SPEC.md`
- `docs/p20/PROVIDER_INTEGRATION_SPEC.md`
- `docs/OPERATOR_QUICKSTART.md`
- `docs/CURRENT_AIDENS_AUDIT.md`
- `README.md`
- `STATUS.md`
- `scripts/p20_scan_aidens.py`
- `docs/p20/reports/PHASE_05_REPORT.md`

## Provider Truth Changes

- Removed the `AdvertisedProviderCapabilitiesV1` source surface that marked cloud providers as native-tool capable without an executable native loop.
- Kept executable provider capability truth in `ProviderCapabilitiesV1::executable_by_backend`.
- Added provider-kit tests proving OpenAI/OpenRouter/Anthropic/OpenAI-compatible/compatible have no executable chat, streaming, structured output, or native tool loop in this build.
- Added an Ollama local HTTP fixture test proving the chat boundary can execute without native tool calls.
- Added `ollama-local-service-required` to Ollama readiness/route/matrix reason codes.
- Added `provider_capability_matrix` to `aidens doctor` output.
- Stopped marking non-mock executable provider configurations as `healthy` in doctor output without a live health proof.
- Added docs matrix artifacts:
  - `docs/p20/PROVIDER_CAPABILITY_MATRIX.md`
  - `docs/p20/PROVIDER_CAPABILITY_MATRIX.json`

## Capability Matrix Summary

| Provider | Support label | Chat executable | Native tool loop executable | Cloud support |
|---|---|---:|---:|---:|
| `disabled` | `blocked/tested` | false | false | false |
| `mock` | `fixture-supported-not-cloud` | true | false | false |
| `ollama` | `partial-local-chat` | true | false | false |
| `openai-compatible` | `deferred/unavailable` | false | false | false |
| `compatible` | `deferred/unavailable` | false | false | false |
| `openai` | `deferred/unavailable` | false | false | false |
| `openrouter` | `deferred/unavailable` | false | false | false |
| `anthropic` | `deferred/unavailable` | false | false | false |

## Failures Found

- The provider kit had a theoretical `AdvertisedProviderCapabilitiesV1` type that returned `native_tool_calling=true` for cloud provider kinds. This was a provider capability overclaim because no native tool-call loop is implemented or tested.
- `aidens doctor` did not expose the full provider capability matrix, so operators could inspect only the configured provider route instead of the whole provider support boundary.
- README/STATUS and operator docs used generic `supported` wording for mock and disabled paths. That was too broad for Phase 05.
- The Phase 04 scanner treated the provider matrix Markdown table header as a provider capability claim. The scanner now skips Markdown table headers/separators while still scanning table rows.

## Fixes Applied

- Removed the theoretical advertised-capability type and replaced its test with executable-truth assertions.
- Added `p20_provider_capability_matrix_matches_executable_truth`.
- Added `ollama_chat_boundary_executes_against_http_fixture_without_native_tools`.
- Added `doctor_reports_provider_capability_matrix_without_cloud_or_native_overclaims`.
- Added `provider_capability_matrix_truths()` to the CLI doctor report.
- Changed docs labels to:
  - `fixture-supported-not-cloud` for mock;
  - `blocked/tested` for disabled behavior;
  - `partial-local-chat` for Ollama;
  - `deferred/unavailable` for cloud/API providers.
- Updated provider golden fixtures for the new Ollama local-service reason code.

## Command Evidence

| Command | Result | Evidence |
|---|---:|---|
| `python3 -m json.tool docs/p20/PROVIDER_CAPABILITY_MATRIX.json` | pass | terminal output: `ok` |
| `cargo fmt --all` | pass | no output |
| `cargo test -p aidens-provider-kit --all-targets` | pass | focused output; 17 tests passed |
| `cargo test -p aidens-cli provider --lib` | pass | focused output; 5 tests passed |
| `cargo test -p aidens-runner unavailable_api_provider_never_claims_native_tool_loop --lib` | pass | focused output; 1 test passed |
| `cargo run -q -p aidens-cli -- provider-check --config examples/aidens.mock.toml` | pass | `target/p20-phase05/logs/01_provider_check_mock.log` |
| `cargo run -q -p aidens-cli -- provider-check --config examples/aidens.openai-unavailable.toml` | pass | `target/p20-phase05/logs/02_provider_check_openai_unavailable.log` |
| `cargo run -q -p aidens-cli -- provider-check --config examples/aidens.ollama.toml` | pass | `target/p20-phase05/logs/03_provider_check_ollama.log` |
| `cargo run -q -p aidens-cli -- doctor --config examples/aidens.mock.toml` | pass | `target/p20-phase05/logs/04_doctor_mock.json` |
| `cargo check --workspace --all-targets --all-features` | pass | `target/p20-phase05/logs/05_cargo_check.log` |
| `cargo test --workspace --all-targets --all-features` | pass | `target/p20-phase05/logs/06_cargo_test.log` |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pass | `target/p20-phase05/logs/07_cargo_clippy.log` |
| `cargo fmt --all -- --check` | pass | `target/p20-phase05/logs/08_cargo_fmt_check.log` |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase05/scan-through-04 --require-phase-reports-through 4 --fail-on-blocking` | pass | `target/p20-phase05/scan-through-04/p20_scan.json`, `target/p20-phase05/scan-through-04/p20_scan.md` |
| `python3 -m py_compile scripts/p20_scan_aidens.py` | pass | `target/p20-phase05/logs/10_scan_py_compile.log` |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase05/scan-through-05 --require-phase-reports-through 5 --fail-on-blocking` | pass | `target/p20-phase05/scan-through-05/p20_scan.json`, `target/p20-phase05/scan-through-05/p20_scan.md` |
| `P20_REQUIRED_PHASE_REPORT_THROUGH=5 bash scripts/p20_verify.sh` | pass | `target/p20-phase05/logs/12_p20_verify_through_05.log`, `target/aidens-final-audit/` |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase05/scan-through-05-final --require-phase-reports-through 5 --fail-on-blocking` | pass | `target/p20-phase05/scan-through-05-final/p20_scan.json`, `target/p20-phase05/scan-through-05-final/p20_scan.md` |
| `cargo test -p aidens-provider-kit --all-targets` after removing hypothetical native test cases | pass | `target/p20-phase05/logs/14_provider_tests_after_native_hypothetical_removal.log` |
| `P20_REQUIRED_PHASE_REPORT_THROUGH=5 bash scripts/p20_verify.sh` final state | pass | `target/p20-phase05/logs/15_p20_verify_through_05_final.log`, `target/aidens-final-audit/` |

Provider-check evidence:

- Mock: `executable: true`, `route: mock`, `native_tool_loop: false`.
- OpenAI unavailable fixture: `executable: false`, `route: unavailable`, `native_tool_loop: false`.
- Ollama: `executable: true`, `route: ollama-chat`, `native_tool_loop: false`, reasons include `ollama-local-service-required` and `ollama-native-tool-loop-unimplemented`.

Doctor matrix evidence:

- `provider-matrix:mock`: `support_label=fixture-supported-not-cloud`, `native_tool_loop_executable=false`.
- `provider-matrix:ollama`: `support_label=partial-local-chat`, `native_tool_loop_executable=false`, `ollama-local-service-required`.
- `provider-matrix:openai`, `provider-matrix:openrouter`, `provider-matrix:anthropic`, `provider-matrix:openai-compatible`, and `provider-matrix:compatible`: `support_label=deferred/unavailable`, all executable flags false.

## Unresolved Blockers

None for Phase 05.

P20 is not final-complete. Phases 06-10 have not run.

## Phase Gate

Phase 05 gate: `PASS`

Stop here and wait for the Phase 06 operator injection.
