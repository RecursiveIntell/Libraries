# Codex Prompt — P02 Provider runtime truth and executable backend matrix

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P02_PROVIDER_RUNTIME_TRUTH_AND_BACKEND_MATRIX.md`.

Implement P02 only. Do not start later passes.

## Goal

Make provider readiness and route labels reflect actual executable backends, not advertised theoretical capabilities.

## Primary crates

- `aidens-provider-kit`
- `aidens-contracts`
- `aidens-cli`
- `aidens-runner`

## Required artifacts

- `ProviderBackendMatrixV1`
- `ProviderReadinessReceiptV1`
- `ProviderRouteReceiptV2`
- `ProviderCertificationFixtureV1`

## Acceptance gates

- provider-check never reports native_tool_loop=true for an unavailable or unsupported backend.
- OpenAI/OpenRouter/Anthropic routes either execute real requests behind feature-gated clients or return provider-boundary-unavailable with native_tool_loop=false.
- Ollama route truth distinguishes ordinary chat from native tool loop if tool calling is not implemented.

## Forbidden shortcuts

- Do not use provider kind string alone to infer executable native capability.
- Do not label parser fallback as native.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
