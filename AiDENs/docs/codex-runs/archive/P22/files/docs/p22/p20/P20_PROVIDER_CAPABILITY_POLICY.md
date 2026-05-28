# P20 Provider Capability Policy

## Safe v0.1 default

| Provider | v0.1 status unless proven otherwise |
|---|---|
| mock | fixture-supported only; not cloud support |
| Ollama | partial-local-chat only if executable chat path and tests pass |
| OpenAI | unavailable unless real implementation + tests exist |
| OpenRouter | unavailable unless real implementation + tests exist |
| Anthropic | unavailable unless real implementation + tests exist |
| OpenAI-compatible | unavailable unless real implementation + tests exist |

## Native tool support rule

Native tool calling is `false` unless an executable native tool-call loop exists and is tested.

## Fallback honesty rule

Fallback is not provider support. If a provider falls back to mock or text-only output, docs/receipts must say so.

## Required artifact

Create/update:

```text
docs/p20/PROVIDER_CAPABILITY_MATRIX.md
docs/p20/PROVIDER_CAPABILITY_MATRIX.json
```
