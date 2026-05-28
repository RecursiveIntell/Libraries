# Provider Capability Matrix Template

| Provider | Config key | Chat executable | Native tool loop executable | Structured output executable | Streaming executable | Status | Required proof | Notes |
|---|---|---:|---:|---:|---:|---|---|---|
| disabled | `disabled` | false | false | false | false | blocked/tested | provider-unavailable test | returns blocked/degraded receipt |
| mock | `mock` | true | false | true | false | fixture-supported-not-cloud | runner fixture test | parser fallback may simulate tool calls |
| ollama | `ollama` | true if implemented and tested | false unless tested | false unless tested | false unless tested | partial-local-chat | integration or mocked HTTP fixture | local model route |
| openai-compatible | `openai-compatible` | false unless implemented | false | false unless implemented | false unless implemented | deferred/unavailable | HTTP tests if enabled | do not claim native tools |
| openai | `openai` | false unless implemented | false unless native tool tests pass | false unless implemented | false unless implemented | deferred/unavailable | HTTP tests if enabled | no fake API surface |
| openrouter | `openrouter` | false unless implemented | false | false unless implemented | false unless implemented | deferred/unavailable | HTTP tests if enabled | no fake API surface |
| anthropic | `anthropic` | false unless implemented | false unless native tool tests pass | false unless implemented | false unless implemented | deferred/unavailable | HTTP tests if enabled | no fake API surface |

## Rule

A provider may advertise a capability only if that exact capability is executable and tested. Otherwise it may expose a configuration placeholder only as `unavailable`, `deferred`, or `provider-boundary-unavailable`.
