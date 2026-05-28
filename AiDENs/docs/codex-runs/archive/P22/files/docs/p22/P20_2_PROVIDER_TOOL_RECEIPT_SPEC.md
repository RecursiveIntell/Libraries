# Provider, Tool, Permit, and Receipt Spec

## Provider truth

For v0.1:

| Provider | Status |
|---|---|
| mock | supported for deterministic tests |
| Ollama | partial chat-only if implemented locally |
| OpenAI | unavailable unless native route is implemented and tested |
| Anthropic | unavailable unless native route is implemented and tested |
| OpenRouter | unavailable unless native route is implemented and tested |
| OpenAI-compatible | unavailable unless explicitly implemented and tested |

## Tool loop proof

The test agent must use a deterministic mock provider/tool path. Codex may not claim cloud-native tool calling unless it adds executable fixtures and tests.

## Receipt law

Every provider route, tool exposure, permit decision, tool invocation, boundary repair, budget exhaustion, agency decision, and final output must be receipt-bearing or explicitly marked out of scope.
