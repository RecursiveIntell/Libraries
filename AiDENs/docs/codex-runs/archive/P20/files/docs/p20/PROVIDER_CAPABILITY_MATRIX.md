# P20 Provider Capability Matrix

Record date: `2026-04-30`

Rule: a provider capability is supported only when that exact capability is executable and tested. Native tool calling is `false` until AiDENs has an implemented and tested native tool-call loop for that provider.

Fallback is not support. Mock is fixture support, not cloud support. Text completion is not native tool use.

| Provider | Config key | Status | Support label | Route | Chat executable | Native tool loop executable | Streaming executable | Structured output executable | Proof | Limits |
|---|---|---|---|---|---:|---:|---:|---:|---|---|
| Disabled | `disabled` | disabled | `blocked/tested` | `disabled` | false | false | false | false | provider and runner disabled-provider tests | intentionally does not answer |
| Mock | `mock` | executable | `fixture-supported-not-cloud` | `mock` | true | false | false | true | provider, runner, CLI, and smoke fixture tests | explicit fixture provider only; not cloud support |
| Ollama | `ollama` | executable boundary | `partial-local-chat` | `ollama-chat` | true | false | false | false | provider route/readiness tests and local HTTP fixture test | requires local Ollama-compatible service at runtime; text chat only |
| OpenAI-compatible | `openai-compatible` | boundary unavailable | `deferred/unavailable` | `unavailable` | false | false | false | false | provider readiness tests assert unavailable | no HTTP boundary, no native tool loop |
| Compatible alias | `compatible` | boundary unavailable | `deferred/unavailable` | `unavailable` | false | false | false | false | provider readiness tests assert unavailable | alias only; no HTTP boundary, no native tool loop |
| OpenAI | `openai` | boundary unavailable | `deferred/unavailable` | `unavailable` | false | false | false | false | provider readiness tests assert unavailable | no HTTP boundary, no native tool loop |
| OpenRouter | `openrouter` | boundary unavailable | `deferred/unavailable` | `unavailable` | false | false | false | false | provider readiness tests assert unavailable | no HTTP boundary, no native tool loop |
| Anthropic | `anthropic` | boundary unavailable | `deferred/unavailable` | `unavailable` | false | false | false | false | provider readiness tests assert unavailable | no HTTP boundary, no native tool loop |

## Doctor Output Contract

`aidens doctor` includes a `provider_capability_matrix` section. Each row is emitted as a `RuntimeCapabilityTruthV1` with:

- `provider-matrix:<kind>` capability id;
- executable/availability states derived from `aidens-provider-kit::provider_backend_matrix`;
- `native_tool_loop_executable=false` for every provider in v0.1;
- `support_label=fixture-supported-not-cloud` for mock;
- `support_label=partial-local-chat` for Ollama;
- `support_label=deferred/unavailable` for cloud/API providers.

## Machine-Readable Artifact

The matching JSON artifact is `docs/p20/PROVIDER_CAPABILITY_MATRIX.json`.
