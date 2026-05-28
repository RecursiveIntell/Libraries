# P02 Provider Runtime Truth And Backend Matrix Handoff

## Scope

- Implemented P02 only.
- Later turn-executor/tool-loop work remains in P03.
- OpenAI-compatible, OpenAI, OpenRouter, and Anthropic HTTP boundaries were not added; they are explicitly reported as `provider-boundary-unavailable` with `native_tool_loop=false`.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-provider-kit/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`
- `tests/fixtures/p02/provider_backend_matrix_v1.json`
- `tests/fixtures/p02/provider_readiness_receipt_v1.json`
- `tests/fixtures/p02/provider_route_receipt_v2.json`
- `tests/fixtures/p02/provider_certification_fixture_v1.json`
- `ARTIFACT_SCHEMA_REGISTRY.md`
- `STATUS.md`

## Artifacts

- Added `ProviderBackendMatrixV1`, `ProviderReadinessReceiptV1`, `ProviderRouteReceiptV2`, and `ProviderCertificationFixtureV1`.
- Added `ProviderRouteKindV1::OllamaChat` so ordinary Ollama chat is not labeled `native-ollama`.
- Added provider certification fixture generation covering disabled, mock, Ollama, OpenAI-compatible, OpenAI, OpenRouter, and Anthropic plus configured, missing-key, missing-model, network-failure, malformed-response, and tool-loop-unavailable scenarios.

## Tests Added

- Contract constructor and golden fixture tests for all P02 artifact families.
- Provider-kit tests for advertised-vs-executable capability separation, backend matrix coverage, OpenRouter/OpenAI unavailable routes, Ollama chat-only route truth, and certification fixture coverage.
- CLI regression tests for provider-check missing-key, unavailable OpenRouter, and Ollama chat-only native flag output.
- Runner regression test proving a configured OpenAI provider without an implemented backend records `unavailable` and `native_tool_loop=false`.

## Commands Run

```bash
cargo check -p aidens-contracts -p aidens-provider-kit -p aidens-cli -p aidens-runner
cargo test -p aidens-contracts
cargo test -p aidens-provider-kit
cargo test -p aidens-cli
cargo test -p aidens-runner
cargo fmt --all
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
```

All commands passed.

## Blockers

- None for P02 acceptance.
- Real OpenAI-compatible/OpenAI/OpenRouter/Anthropic HTTP clients remain unimplemented by design in this pass and are surfaced as unavailable rather than native.

## Next-Pass Readiness

- P03 can start from explicit provider route truth: mock is executable, Ollama is ordinary chat only, API providers are boundary-unavailable, and no unsupported route exposes a native tool loop.
