# Provider Integration Spec

## Required provider trait

Implement a provider trait or equivalent in `aidens-provider-kit`.

```rust
#[async_trait::async_trait]
pub trait AiDENsProvider: Send + Sync {
    fn provider_kind(&self) -> &str;
    fn model(&self) -> Option<&str>;
    fn capabilities(&self) -> ProviderCapabilitiesV1;
    async fn complete(&self, request: AiDENsCompletionRequestV1) -> anyhow::Result<AiDENsCompletionResponseV1>;
}
```

## Required request/response

Add types or equivalents:

```rust
pub struct AiDENsCompletionRequestV1 {
    pub messages: Vec<AiDENsChatMessageV1>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

pub struct AiDENsCompletionResponseV1 {
    pub content: String,
    pub model: Option<String>,
    pub provider_kind: String,
    pub token_usage: Option<...>,
    pub transport_retry: Option<...>,
}
```

## Providers

### DisabledProvider

- `provider_kind = "disabled"`.
- Must return error on `complete`.
- Must never return answer text.

### MockProvider

- `provider_kind = "mock"`.
- Only enabled by explicit config.
- Returns configured response.
- Used for tests/smoke only.

### Ollama/LlmPipeline provider

Try to wire using Recall's source:

```text
~/Coding/Recall/recall-session/src/provider.rs
~/Coding/Recall/recall-session/src/provider_bridge.rs
~/Coding/Recall/deps/llm-pipeline/src/lib.rs
```

If dependency wiring is too large for this pass, keep the provider trait stable and record exact blockers. Do not fake a real provider.

## Route truth

Route labels must remain exact:

```text
native-openai-responses
native-openai-chat
native-anthropic
native-ollama
openai-compatible
parser-fallback
disabled
unavailable
degraded
mock
```

Provider truth must reflect actual execution, not provider-family capability assumptions.
