//! Provider construction, execution, and route truth.

use aidens_contracts::{
    ProviderBackendMatrixEntryV1, ProviderBackendMatrixV1, ProviderBackendStatusV1,
    ProviderCertificationFixtureDraftV1, ProviderCertificationFixtureV1,
    ProviderReadinessReportDraftV1, ProviderReadinessReportV1, ProviderRouteKindV1,
    ProviderRouteReportDraftV2, ProviderRouteReportV1, ProviderRouteReportV2, ToolCallRequestV1,
    ToolCallResultV1, ToolCallSourceV1, ToolProviderSchemaV1,
};
use anyhow::{anyhow, bail, Context};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

pub mod canonical_stack {
    pub use llm_tool_runtime::{
        render_ollama_tool, render_openai_tool, ToolDescriptor as CanonicalToolDescriptor,
        ToolError as CanonicalToolError, ToolReceipt as CanonicalToolReceipt,
        ToolRuntime as CanonicalToolRuntime, ToolRuntimeConfig as CanonicalToolRuntimeConfig,
    };

    pub fn render_openai_tool_schema(
        descriptor: &CanonicalToolDescriptor,
        strict: bool,
    ) -> Result<serde_json::Value, CanonicalToolError> {
        render_openai_tool(descriptor, strict)
    }

    pub fn render_ollama_tool_schema(
        descriptor: &CanonicalToolDescriptor,
    ) -> Result<serde_json::Value, CanonicalToolError> {
        render_ollama_tool(descriptor)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AiDENsChatMessageV1 {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AiDENsCompletionRequestV1 {
    pub messages: Vec<AiDENsChatMessageV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provider_tool_schemas: Vec<ToolProviderSchemaV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_results: Vec<ToolCallResultV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

impl AiDENsCompletionRequestV1 {
    pub fn single_user(prompt: impl Into<String>) -> Self {
        Self {
            messages: vec![AiDENsChatMessageV1 {
                role: "user".into(),
                content: prompt.into(),
            }],
            provider_tool_schemas: Vec::new(),
            tool_results: Vec::new(),
            temperature: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AiDENsCompletionResponseV1 {
    pub text: String,
    pub provider_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCallRequestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct NativeProviderToolCallV1 {
    function: NativeProviderFunctionCallV1,
}

#[derive(Debug, Deserialize)]
struct NativeProviderFunctionCallV1 {
    name: String,
    arguments: serde_json::Value,
}

fn parse_native_provider_tool_calls(
    provider_kind: &str,
    value: Option<&serde_json::Value>,
    name_to_tool_id: &std::collections::HashMap<String, String>,
) -> anyhow::Result<Vec<ToolCallRequestV1>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if value.is_null() {
        return Ok(Vec::new());
    }
    let calls = value.as_array().ok_or_else(|| {
        anyhow!("{provider_kind} provider: malformed provider tool_calls: expected an array")
    })?;

    calls
        .iter()
        .enumerate()
        .map(|(index, call)| {
            let parsed: NativeProviderToolCallV1 = serde_json::from_value(call.clone())
                .with_context(|| {
                    format!(
                        "{provider_kind} provider: malformed provider tool call at index {index}"
                    )
                })?;
            if parsed.function.name.trim().is_empty() {
                bail!(
                    "{provider_kind} provider: malformed provider tool call at index {index}: function name is empty"
                );
            }
            let tool_id = name_to_tool_id
                .get(&parsed.function.name)
                .cloned()
                .unwrap_or(parsed.function.name);
            let arguments = match parsed.function.arguments {
                serde_json::Value::String(encoded) => serde_json::from_str(&encoded).with_context(
                    || {
                        format!(
                            "{provider_kind} provider: malformed provider tool call at index {index}: function arguments are not strict JSON"
                        )
                    },
                )?,
                arguments => arguments,
            };
            Ok(ToolCallRequestV1::new(
                ToolCallSourceV1::NativeProvider,
                tool_id,
                arguments,
                None,
                vec![],
            ))
        })
        .collect()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderCapabilitiesV1 {
    /// The provider can execute one bounded chat-completion request.
    pub chat_completion: bool,
    /// A provider-to-tool-to-provider function loop has fixture proof.
    ///
    /// Strictly parsing a native tool-call envelope alone does not satisfy this capability.
    pub native_tool_calling: bool,
    /// The provider has a fixture-proven streaming response path.
    pub streaming: bool,
    /// The provider has a fixture-proven structured-output contract.
    pub structured_output: bool,
}

impl ProviderCapabilitiesV1 {
    pub fn executable_by_backend(kind: &str) -> Self {
        match normalized_provider_kind(kind).as_str() {
            "mock" => Self {
                chat_completion: true,
                native_tool_calling: false,
                streaming: false,
                structured_output: true,
            },
            "ollama" => Self {
                chat_completion: true,
                native_tool_calling: false,
                streaming: false,
                structured_output: false,
            },
            "openai-compatible" => Self {
                chat_completion: true,
                native_tool_calling: false,
                streaming: false,
                structured_output: false,
            },
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderReadinessV1 {
    pub configured: bool,
    pub executable: bool,
    pub native_tool_loop_executable: bool,
    pub route_label: String,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderSpecV1 {
    pub kind: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub mock_response: Option<String>,
}

impl ProviderSpecV1 {
    pub fn new(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            ..Self::default()
        }
    }
}

#[async_trait]
pub trait AiDENsProvider: Send + Sync {
    fn provider_kind(&self) -> &str;
    fn model(&self) -> Option<&str>;
    fn capabilities(&self) -> ProviderCapabilitiesV1;
    async fn complete(
        &self,
        request: AiDENsCompletionRequestV1,
    ) -> anyhow::Result<AiDENsCompletionResponseV1>;
}

#[derive(Debug, Clone, Default)]
pub struct DisabledProvider {
    model: Option<String>,
}

impl DisabledProvider {
    pub fn new(model: Option<String>) -> Self {
        Self { model }
    }
}

#[async_trait]
impl AiDENsProvider for DisabledProvider {
    fn provider_kind(&self) -> &str {
        "disabled"
    }

    fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    fn capabilities(&self) -> ProviderCapabilitiesV1 {
        ProviderCapabilitiesV1::default()
    }

    async fn complete(
        &self,
        _request: AiDENsCompletionRequestV1,
    ) -> anyhow::Result<AiDENsCompletionResponseV1> {
        bail!("provider disabled: no completion provider is executable")
    }
}

#[derive(Debug, Clone)]
pub struct MockProvider {
    /// Original response text. Retained for Debug display; segment dispatch
    /// uses `segments` exclusively.
    #[allow(dead_code)]
    response: String,
    segments: Vec<String>,
    model: Option<String>,
}

const MOCK_RESPONSE_DELIMITER: &str = "\n---aidens-next-response---\n";

impl MockProvider {
    pub fn new(response: impl Into<String>, model: Option<String>) -> anyhow::Result<Self> {
        let response = response.into();
        if response.trim().is_empty() {
            bail!("mock provider requires an explicit non-empty mock_response")
        }
        // Pre-split response segments at construction time to avoid
        // re-splitting on every completion call.
        let segments = response
            .split(MOCK_RESPONSE_DELIMITER)
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        Ok(Self {
            response,
            segments,
            model,
        })
    }

    fn response_for_request(&self, request: &AiDENsCompletionRequestV1) -> String {
        let index = request
            .tool_results
            .len()
            .min(self.segments.len().saturating_sub(1));
        render_mock_response_template(&self.segments[index], request)
    }
}

#[async_trait]
impl AiDENsProvider for MockProvider {
    fn provider_kind(&self) -> &str {
        "mock"
    }

    fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    fn capabilities(&self) -> ProviderCapabilitiesV1 {
        ProviderCapabilitiesV1 {
            chat_completion: true,
            native_tool_calling: false,
            streaming: false,
            structured_output: true,
        }
    }

    async fn complete(
        &self,
        request: AiDENsCompletionRequestV1,
    ) -> anyhow::Result<AiDENsCompletionResponseV1> {
        Ok(AiDENsCompletionResponseV1 {
            text: self.response_for_request(&request),
            provider_kind: self.provider_kind().into(),
            model: self.model.clone(),
            tool_calls: Vec::new(),
            prompt_eval_count: None,
            eval_count: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> anyhow::Result<Self> {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let model = model.into();
        if base_url.trim().is_empty() {
            bail!("ollama provider unavailable: base_url is empty")
        }
        if model.trim().is_empty() {
            bail!("ollama provider unavailable: model is empty")
        }
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(300))
            .build()
            .context("ollama provider unavailable: failed to build HTTP client")?;
        Ok(Self {
            base_url,
            model,
            client,
        })
    }
}

#[async_trait]
impl AiDENsProvider for OllamaProvider {
    fn provider_kind(&self) -> &str {
        "ollama"
    }

    fn model(&self) -> Option<&str> {
        Some(&self.model)
    }

    fn capabilities(&self) -> ProviderCapabilitiesV1 {
        ProviderCapabilitiesV1 {
            chat_completion: true,
            native_tool_calling: true,
            streaming: false,
            structured_output: false,
        }
    }

    async fn complete(
        &self,
        request: AiDENsCompletionRequestV1,
    ) -> anyhow::Result<AiDENsCompletionResponseV1> {
        let messages = request
            .messages
            .iter()
            .map(|message| {
                // If this is an assistant message containing tool_calls JSON, parse it and
                // send it as native Ollama tool_calls field instead of content string.
                if message.role == "assistant" && message.content.starts_with("{\"tool_calls\":") {
                    if let Ok(tool_calls) =
                        serde_json::from_str::<serde_json::Value>(&message.content)
                    {
                        if let Some(calls) = tool_calls["tool_calls"].as_array() {
                            // Only use native tool_calls if the array is non-empty
                            // and each entry has a function.name field.
                            let valid = !calls.is_empty()
                                && calls
                                    .iter()
                                    .all(|c| c["function"]["name"].as_str().is_some());
                            if valid {
                                return serde_json::json!({
                                    "role": "assistant",
                                    "content": "",
                                    "tool_calls": calls,
                                });
                            }
                        }
                    }
                }
                serde_json::json!({
                    "role": message.role,
                    "content": message.content,
                })
            })
            .collect::<Vec<_>>();
        let mut options = serde_json::json!({});
        if let Some(temp) = request.temperature {
            options["temperature"] = serde_json::json!(temp);
        }

        // Build tools array from provider_tool_schemas (Ollama uses OpenAI function format)
        // Use the `name` field (simple name) as the function name, not the tool_id
        // (which contains colons that confuse the model).
        // We'll map back from name to tool_id when parsing the response.
        let tools: Vec<serde_json::Value> = request
            .provider_tool_schemas
            .iter()
            .map(|schema| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": schema.name,
                        "description": schema.description,
                        "parameters": schema.input_schema,
                    }
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
            "options": options,
        });

        if !tools.is_empty() {
            body["tools"] = serde_json::json!(tools);
        }

        let url = format!("{}/api/chat", self.base_url);
        let response = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|error| anyhow!("ollama provider unavailable: {error}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("ollama provider unavailable: returned {status}: {body}")
        }

        let json = response
            .json::<serde_json::Value>()
            .await
            .context("ollama provider unavailable: response JSON parse failed")?;

        let text = json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Parse tool_calls from Ollama response
        // Map function names back to tool_ids (Ollama uses simple names, runner expects namespaced IDs)
        let name_to_tool_id: std::collections::HashMap<String, String> = request
            .provider_tool_schemas
            .iter()
            .map(|s| (s.name.clone(), s.tool_id.clone()))
            .collect();
        let tool_calls = parse_native_provider_tool_calls(
            "ollama",
            json["message"].get("tool_calls"),
            &name_to_tool_id,
        )?;

        // If we got tool_calls, text may be empty — that's OK.
        // If text is empty AND no tool_calls, check if this was a follow-up
        // after tool results (the model may return empty when it's done).
        if text.is_empty() && tool_calls.is_empty() {
            // Return empty text — the runner handles this as a normal (empty) response.
        }

        Ok(AiDENsCompletionResponseV1 {
            text,
            provider_kind: self.provider_kind().into(),
            model: Some(self.model.clone()),
            tool_calls,
            prompt_eval_count: json["prompt_eval_count"].as_u64(),
            eval_count: json["eval_count"].as_u64(),
        })
    }
}

fn render_mock_response_template(template: &str, request: &AiDENsCompletionRequestV1) -> String {
    let last_result = request.tool_results.last();
    let last_tool_output_text = last_result
        .map(ToolCallResultV1::output_text)
        .unwrap_or_default();
    let last_tool_content = last_result
        .and_then(|result| result.output.as_ref())
        .and_then(|output| output.get("content"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    template
        .replace("{{last_tool_output_text}}", &last_tool_output_text)
        .replace("{{last_tool_content}}", last_tool_content)
        .to_string()
}

// ── OpenAiCompatibleProvider ──────────────────────────────────────────

const OPENAI_COMPATIBLE_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const OPENAI_COMPATIBLE_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);
const OPENAI_COMPATIBLE_MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone)]
pub struct OpenAiCompatibleProvider {
    kind: String,
    endpoint: reqwest::Url,
    model: String,
    client: reqwest::Client,
}

impl std::fmt::Debug for OpenAiCompatibleProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpenAiCompatibleProvider")
            .field("kind", &self.kind)
            .field("endpoint", &self.endpoint)
            .field("model", &self.model)
            .field("authorization", &"<redacted-sensitive-header>")
            .finish()
    }
}

impl OpenAiCompatibleProvider {
    fn new(
        kind: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
        api_key: String,
    ) -> anyhow::Result<Self> {
        let kind = kind.into();
        let endpoint = openai_compatible_chat_endpoint(&base_url.into())?;
        let model = model.into().trim().to_string();
        if model.trim().is_empty() {
            bail!("openai-compatible provider unavailable: model is empty")
        }
        let mut authorization = HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|_| anyhow!("openai-compatible provider unavailable: env bearer key is not a valid HTTP header value"))?;
        authorization.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, authorization);
        let client = reqwest::Client::builder()
            .connect_timeout(OPENAI_COMPATIBLE_CONNECT_TIMEOUT)
            .timeout(OPENAI_COMPATIBLE_REQUEST_TIMEOUT)
            .default_headers(headers)
            .build()
            .context("openai-compatible provider unavailable: failed to build HTTP client")?;
        Ok(Self {
            kind,
            endpoint,
            model,
            client,
        })
    }
}

fn openai_compatible_chat_endpoint(base_url: &str) -> anyhow::Result<reqwest::Url> {
    let base_url = base_url.trim();
    if base_url.is_empty() {
        bail!("openai-compatible provider unavailable: base_url is empty")
    }
    let mut endpoint = reqwest::Url::parse(base_url).map_err(|_| {
        anyhow!("openai-compatible provider unavailable: base_url is not a valid URL")
    })?;
    if !matches!(endpoint.scheme(), "http" | "https")
        || endpoint.host_str().is_none()
        || endpoint.cannot_be_a_base()
    {
        bail!("openai-compatible provider unavailable: base_url must be an absolute HTTP(S) URL")
    }
    if !endpoint.username().is_empty() || endpoint.password().is_some() {
        bail!("openai-compatible provider unavailable: base_url must not contain credentials")
    }
    if endpoint.query().is_some() || endpoint.fragment().is_some() {
        bail!(
            "openai-compatible provider unavailable: base_url must not contain a query or fragment"
        )
    }
    let base_path = endpoint.path().trim_end_matches('/');
    let path = if base_path.ends_with("/v1") {
        format!("{base_path}/chat/completions")
    } else if base_path.is_empty() {
        "/v1/chat/completions".to_string()
    } else {
        format!("{base_path}/v1/chat/completions")
    };
    endpoint.set_path(&path);
    Ok(endpoint)
}

fn openai_compatible_api_key_from_env() -> Option<String> {
    ["OPENAI_API_KEY", "AIDENS_OPENAI_API_KEY"]
        .into_iter()
        .find_map(|name| {
            std::env::var(name)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

#[derive(Debug, Deserialize)]
struct OpenAiCompatibleResponseV1 {
    choices: Vec<OpenAiCompatibleChoiceV1>,
    #[serde(default)]
    usage: OpenAiCompatibleUsageV1,
}

#[derive(Debug, Deserialize)]
struct OpenAiCompatibleChoiceV1 {
    message: OpenAiCompatibleMessageV1,
}

#[derive(Debug, Deserialize)]
struct OpenAiCompatibleMessageV1 {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<serde_json::Value>,
}

#[derive(Debug, Default, Deserialize)]
struct OpenAiCompatibleUsageV1 {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
}

async fn read_bounded_json<T: DeserializeOwned>(
    mut response: reqwest::Response,
    provider_kind: &str,
) -> anyhow::Result<T> {
    if response
        .content_length()
        .is_some_and(|length| length > OPENAI_COMPATIBLE_MAX_RESPONSE_BYTES as u64)
    {
        bail!(
            "{provider_kind} provider unavailable: response exceeds {} bytes",
            OPENAI_COMPATIBLE_MAX_RESPONSE_BYTES
        )
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.with_context(|| {
        format!("{provider_kind} provider unavailable: response body read failed")
    })? {
        if body.len().saturating_add(chunk.len()) > OPENAI_COMPATIBLE_MAX_RESPONSE_BYTES {
            bail!(
                "{provider_kind} provider unavailable: response exceeds {} bytes",
                OPENAI_COMPATIBLE_MAX_RESPONSE_BYTES
            )
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body)
        .with_context(|| format!("{provider_kind} provider: response JSON parse failed"))
}

#[async_trait]
impl AiDENsProvider for OpenAiCompatibleProvider {
    fn provider_kind(&self) -> &str {
        &self.kind
    }
    fn model(&self) -> Option<&str> {
        Some(&self.model)
    }
    fn capabilities(&self) -> ProviderCapabilitiesV1 {
        ProviderCapabilitiesV1 {
            chat_completion: true,
            native_tool_calling: false,
            streaming: false,
            structured_output: false,
        }
    }

    async fn complete(
        &self,
        request: AiDENsCompletionRequestV1,
    ) -> anyhow::Result<AiDENsCompletionResponseV1> {
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();
        let mut body =
            serde_json::json!({"model": self.model, "messages": messages, "stream": false});
        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        let tools: Vec<serde_json::Value> = request.provider_tool_schemas.iter().map(|s| serde_json::json!({"type": "function", "function": {"name": s.name, "description": s.description, "parameters": s.input_schema}})).collect();
        if !tools.is_empty() {
            body["tools"] = serde_json::json!(tools);
        }
        let response = self
            .client
            .post(self.endpoint.clone())
            .json(&body)
            .send()
            .await
            .map_err(|error| anyhow!("openai-compatible provider unavailable: {error}"))?;
        if !response.status().is_success() {
            let status = response.status();
            bail!("openai-compatible provider returned HTTP {status}")
        }
        let response: OpenAiCompatibleResponseV1 =
            read_bounded_json(response, "openai-compatible").await?;
        let choice =
            response.choices.into_iter().next().ok_or_else(|| {
                anyhow!("openai-compatible provider: response contains no choices")
            })?;
        let text = choice.message.content.unwrap_or_default();
        let name_to_tool_id: std::collections::HashMap<String, String> = request
            .provider_tool_schemas
            .iter()
            .map(|s| (s.name.clone(), s.tool_id.clone()))
            .collect();
        let tool_calls = parse_native_provider_tool_calls(
            "openai-compatible",
            choice.message.tool_calls.as_ref(),
            &name_to_tool_id,
        )?;
        Ok(AiDENsCompletionResponseV1 {
            text,
            provider_kind: self.provider_kind().into(),
            model: Some(self.model.clone()),
            prompt_eval_count: response.usage.prompt_tokens,
            eval_count: response.usage.completion_tokens,
            tool_calls,
        })
    }
}

pub struct UnavailableProvider {
    kind: String,
    model: Option<String>,
    reason: String,
}

impl UnavailableProvider {
    pub fn new(kind: impl Into<String>, model: Option<String>, reason: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            model,
            reason: reason.into(),
        }
    }
}

#[async_trait]
impl AiDENsProvider for UnavailableProvider {
    fn provider_kind(&self) -> &str {
        &self.kind
    }

    fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    fn capabilities(&self) -> ProviderCapabilitiesV1 {
        ProviderCapabilitiesV1::default()
    }

    async fn complete(
        &self,
        _request: AiDENsCompletionRequestV1,
    ) -> anyhow::Result<AiDENsCompletionResponseV1> {
        bail!("provider unavailable: {}", self.reason)
    }
}

pub fn build_provider(spec: ProviderSpecV1) -> anyhow::Result<Arc<dyn AiDENsProvider>> {
    let normalized = normalized_provider_kind(&spec.kind);
    let provider: Arc<dyn AiDENsProvider> = match normalized.as_str() {
        "" | "disabled" | "none" => Arc::new(DisabledProvider::new(spec.model)),
        "local" => Arc::new(UnavailableProvider::new(
            "local",
            spec.model,
            "local-provider-boundary-unavailable: configure kind 'mock' for fixture replay or kind 'ollama' for a local service boundary",
        )),
        "mock" => Arc::new(MockProvider::new(
            spec.mock_response.unwrap_or_default(),
            spec.model,
        )?),
        "openai-compatible" => {
            let base_url = spec.base_url.ok_or_else(|| {
                anyhow!(
                    "openai-compatible provider unavailable: base_url must be explicitly configured"
                )
            })?;
            let model = spec.model.ok_or_else(|| {
                anyhow!(
                    "openai-compatible provider unavailable: model must be explicitly configured"
                )
            })?;
            if base_url.trim().is_empty() {
                bail!("openai-compatible provider unavailable: base_url is empty")
            }
            if model.trim().is_empty() {
                bail!("openai-compatible provider unavailable: model is empty")
            }
            match openai_compatible_api_key_from_env() {
                Some(api_key) => Arc::new(OpenAiCompatibleProvider::new(
                    normalized, base_url, model, api_key,
                )?),
                None => Arc::new(UnavailableProvider::new(
                    normalized,
                    Some(model),
                    "api-key-missing: OPENAI_API_KEY and AIDENS_OPENAI_API_KEY are unset or empty",
                )),
            }
        }
        "ollama" => Arc::new(OllamaProvider::new(
            spec.base_url
                .unwrap_or_else(|| "http://localhost:11434".into()),
            spec.model
                .ok_or_else(|| anyhow!("ollama provider unavailable: model is not configured"))?,
        )?),
        other => Arc::new(UnavailableProvider::new(
            other,
            spec.model,
            format!(
                "provider-boundary-unavailable: provider kind '{other}' has no executable boundary in this build"
            ),
        )),
    };
    Ok(provider)
}

pub fn provider_backend_matrix() -> ProviderBackendMatrixV1 {
    ProviderBackendMatrixV1::new(vec![
        ProviderBackendMatrixEntryV1 {
            provider_kind: "disabled".into(),
            status: ProviderBackendStatusV1::Disabled,
            route_label: ProviderRouteKindV1::Disabled.to_string(),
            api_key_required: false,
            chat_completion_executable: false,
            native_tool_loop_executable: false,
            streaming_executable: false,
            structured_output_executable: false,
            reason_codes: vec!["provider-disabled".into()],
        },
        ProviderBackendMatrixEntryV1 {
            provider_kind: "local".into(),
            status: ProviderBackendStatusV1::BoundaryUnavailable,
            route_label: ProviderRouteKindV1::Unavailable.to_string(),
            api_key_required: false,
            chat_completion_executable: false,
            native_tool_loop_executable: false,
            streaming_executable: false,
            structured_output_executable: false,
            reason_codes: vec!["local-provider-boundary-unavailable".into()],
        },
        ProviderBackendMatrixEntryV1 {
            provider_kind: "mock".into(),
            status: ProviderBackendStatusV1::Executable,
            route_label: ProviderRouteKindV1::Mock.to_string(),
            api_key_required: false,
            chat_completion_executable: true,
            native_tool_loop_executable: false,
            streaming_executable: false,
            structured_output_executable: true,
            reason_codes: vec!["mock-provider-executable".into()],
        },
        ProviderBackendMatrixEntryV1 {
            provider_kind: "ollama".into(),
            status: ProviderBackendStatusV1::Executable,
            route_label: ProviderRouteKindV1::OllamaChat.to_string(),
            api_key_required: false,
            chat_completion_executable: true,
            native_tool_loop_executable: true,
            streaming_executable: false,
            structured_output_executable: false,
            reason_codes: vec![
                "ollama-chat-boundary-implemented".into(),
                "ollama-local-service-required".into(),
                "ollama-native-tool-loop-via-function-calling".into(),
            ],
        },
        ProviderBackendMatrixEntryV1 {
            provider_kind: "openai-compatible".into(),
            status: ProviderBackendStatusV1::Executable,
            route_label: ProviderRouteKindV1::OpenAiCompatible.to_string(),
            api_key_required: true,
            chat_completion_executable: true,
            native_tool_loop_executable: false,
            streaming_executable: false,
            structured_output_executable: false,
            reason_codes: vec![
                "openai-compatible-http-boundary-implemented".into(),
                "explicit-base-url-model-and-env-key-required".into(),
                "native-tool-loop-unproven".into(),
            ],
        },
        ProviderBackendMatrixEntryV1 {
            provider_kind: "compatible".into(),
            status: ProviderBackendStatusV1::BoundaryUnavailable,
            route_label: ProviderRouteKindV1::Unavailable.to_string(),
            api_key_required: true,
            chat_completion_executable: false,
            native_tool_loop_executable: false,
            streaming_executable: false,
            structured_output_executable: false,
            reason_codes: vec!["provider-boundary-unavailable".into()],
        },
        ProviderBackendMatrixEntryV1 {
            provider_kind: "openai".into(),
            status: ProviderBackendStatusV1::BoundaryUnavailable,
            route_label: ProviderRouteKindV1::Unavailable.to_string(),
            api_key_required: true,
            chat_completion_executable: false,
            native_tool_loop_executable: false,
            streaming_executable: false,
            structured_output_executable: false,
            reason_codes: vec!["provider-boundary-unavailable".into()],
        },
        ProviderBackendMatrixEntryV1 {
            provider_kind: "openrouter".into(),
            status: ProviderBackendStatusV1::BoundaryUnavailable,
            route_label: ProviderRouteKindV1::Unavailable.to_string(),
            api_key_required: true,
            chat_completion_executable: false,
            native_tool_loop_executable: false,
            streaming_executable: false,
            structured_output_executable: false,
            reason_codes: vec!["provider-boundary-unavailable".into()],
        },
        ProviderBackendMatrixEntryV1 {
            provider_kind: "anthropic".into(),
            status: ProviderBackendStatusV1::BoundaryUnavailable,
            route_label: ProviderRouteKindV1::Unavailable.to_string(),
            api_key_required: true,
            chat_completion_executable: false,
            native_tool_loop_executable: false,
            streaming_executable: false,
            structured_output_executable: false,
            reason_codes: vec!["provider-boundary-unavailable".into()],
        },
    ])
}

pub fn provider_certification_fixtures() -> Vec<ProviderCertificationFixtureV1> {
    let fixture = |provider_kind: &str,
                   scenario: &str,
                   input_config: &[(&str, &str)],
                   expected_configured: bool,
                   expected_executable: bool,
                   expected_route_label: &str,
                   expected_native_tool_loop: bool,
                   expected_reason_codes: &[&str]| {
        ProviderCertificationFixtureV1::new(ProviderCertificationFixtureDraftV1 {
            provider_kind: provider_kind.into(),
            scenario: scenario.into(),
            input_config: input_config
                .iter()
                .map(|(key, value)| ((*key).into(), (*value).into()))
                .collect::<BTreeMap<_, _>>(),
            expected_configured,
            expected_executable,
            expected_route_label: expected_route_label.into(),
            expected_native_tool_loop,
            expected_reason_codes: expected_reason_codes
                .iter()
                .map(|reason| (*reason).into())
                .collect(),
        })
    };

    vec![
        fixture(
            "disabled",
            "provider-disabled",
            &[("kind", "disabled")],
            false,
            false,
            "disabled",
            false,
            &["provider-disabled"],
        ),
        fixture(
            "mock",
            "configured",
            &[("kind", "mock"), ("mock_response", "ok")],
            true,
            true,
            "mock",
            false,
            &["mock-response-configured"],
        ),
        fixture(
            "mock",
            "mock-response-missing",
            &[("kind", "mock")],
            true,
            false,
            "unavailable",
            false,
            &["mock-response-missing"],
        ),
        fixture(
            "ollama",
            "configured",
            &[("kind", "ollama"), ("model", "llama3")],
            true,
            true,
            "ollama-chat",
            false,
            &[
                "ollama-chat-boundary-configured",
                "ollama-local-service-required",
                "ollama-native-tool-loop-unimplemented",
            ],
        ),
        fixture(
            "ollama",
            "missing-model",
            &[("kind", "ollama")],
            false,
            false,
            "unavailable",
            false,
            &["ollama-model-missing"],
        ),
        fixture(
            "ollama",
            "network-failure",
            &[("kind", "ollama"), ("model", "llama3")],
            true,
            true,
            "ollama-chat",
            false,
            &[
                "ollama-runtime-network-failure",
                "ollama-local-service-required",
                "ollama-native-tool-loop-unimplemented",
            ],
        ),
        fixture(
            "ollama",
            "malformed-response",
            &[("kind", "ollama"), ("model", "llama3")],
            true,
            true,
            "ollama-chat",
            false,
            &[
                "ollama-runtime-malformed-response",
                "ollama-local-service-required",
                "ollama-native-tool-loop-unimplemented",
            ],
        ),
        fixture(
            "ollama",
            "tool-loop-unavailable",
            &[("kind", "ollama"), ("model", "llama3")],
            true,
            true,
            "ollama-chat",
            false,
            &[
                "ollama-local-service-required",
                "ollama-native-tool-loop-unimplemented",
            ],
        ),
        fixture(
            "openai-compatible",
            "configured",
            &[
                ("kind", "openai-compatible"),
                ("model", "model"),
                ("base_url", "http://127.0.0.1:fixture"),
                ("bearer_key_source", "environment"),
            ],
            true,
            true,
            "openai-compatible",
            false,
            &[
                "openai-compatible-http-boundary-configured",
                "native-tool-loop-unproven",
            ],
        ),
        fixture(
            "openai-compatible",
            "missing-key",
            &[
                ("kind", "openai-compatible"),
                ("model", "model"),
                ("base_url", "http://127.0.0.1:fixture"),
            ],
            true,
            false,
            "unavailable",
            false,
            &["api-key-missing"],
        ),
        fixture(
            "openai",
            "configured-boundary-unavailable",
            &[
                ("kind", "openai"),
                ("model", "gpt-test"),
                ("api_key", "configured"),
            ],
            true,
            false,
            "unavailable",
            false,
            &["provider-boundary-unavailable"],
        ),
        fixture(
            "openai",
            "missing-key",
            &[("kind", "openai"), ("model", "gpt-test")],
            true,
            false,
            "unavailable",
            false,
            &["api-key-missing"],
        ),
        fixture(
            "openrouter",
            "configured-boundary-unavailable",
            &[
                ("kind", "openrouter"),
                ("model", "router-test"),
                ("api_key", "configured"),
            ],
            true,
            false,
            "unavailable",
            false,
            &["provider-boundary-unavailable"],
        ),
        fixture(
            "openrouter",
            "missing-key",
            &[("kind", "openrouter"), ("model", "router-test")],
            true,
            false,
            "unavailable",
            false,
            &["api-key-missing"],
        ),
        fixture(
            "anthropic",
            "configured-boundary-unavailable",
            &[
                ("kind", "anthropic"),
                ("model", "claude-test"),
                ("api_key", "configured"),
            ],
            true,
            false,
            "unavailable",
            false,
            &["provider-boundary-unavailable"],
        ),
        fixture(
            "anthropic",
            "missing-key",
            &[("kind", "anthropic"), ("model", "claude-test")],
            true,
            false,
            "unavailable",
            false,
            &["api-key-missing"],
        ),
    ]
}

pub fn provider_readiness(kind: &str, _api_key: Option<&str>) -> ProviderReadinessV1 {
    let spec = ProviderSpecV1::new(kind);
    provider_readiness_for_spec(&spec)
}

pub fn provider_readiness_for_spec(spec: &ProviderSpecV1) -> ProviderReadinessV1 {
    let receipt = provider_readiness_receipt_for_spec(spec);
    ProviderReadinessV1 {
        configured: receipt.configured,
        executable: receipt.executable,
        native_tool_loop_executable: receipt.native_tool_loop_executable,
        route_label: receipt.route_label,
        reason_codes: receipt.reason_codes,
    }
}

pub fn provider_readiness_receipt_for_spec(spec: &ProviderSpecV1) -> ProviderReadinessReportV1 {
    let normalized = normalized_provider_kind(&spec.kind);
    if matches!(normalized.as_str(), "" | "disabled" | "none") {
        return ProviderReadinessReportV1::new(ProviderReadinessReportDraftV1 {
            provider_kind: "disabled".into(),
            model: spec.model.clone(),
            configured: false,
            executable: false,
            native_tool_loop_executable: false,
            route_label: ProviderRouteKindV1::Disabled.to_string(),
            reason_codes: vec!["provider-disabled".into()],
        });
    }

    if normalized == "mock" {
        let executable = spec
            .mock_response
            .as_deref()
            .is_some_and(|response| !response.trim().is_empty());
        return ProviderReadinessReportV1::new(ProviderReadinessReportDraftV1 {
            provider_kind: normalized,
            model: spec.model.clone(),
            configured: true,
            executable,
            native_tool_loop_executable: false,
            route_label: if executable {
                ProviderRouteKindV1::Mock.to_string()
            } else {
                ProviderRouteKindV1::Unavailable.to_string()
            },
            reason_codes: if executable {
                vec!["mock-response-configured".into()]
            } else {
                vec!["mock-response-missing".into()]
            },
        });
    }

    if normalized == "local" {
        return ProviderReadinessReportV1::new(ProviderReadinessReportDraftV1 {
            provider_kind: normalized,
            model: spec.model.clone(),
            configured: true,
            executable: false,
            native_tool_loop_executable: false,
            route_label: ProviderRouteKindV1::Unavailable.to_string(),
            reason_codes: vec!["local-provider-boundary-unavailable".into()],
        });
    }

    if normalized == "ollama" {
        let model_configured = spec
            .model
            .as_deref()
            .is_some_and(|model| !model.trim().is_empty());
        return ProviderReadinessReportV1::new(ProviderReadinessReportDraftV1 {
            provider_kind: normalized,
            model: spec.model.clone(),
            configured: model_configured,
            executable: model_configured,
            native_tool_loop_executable: true,
            route_label: if model_configured {
                ProviderRouteKindV1::OllamaChat.to_string()
            } else {
                ProviderRouteKindV1::Unavailable.to_string()
            },
            reason_codes: if model_configured {
                vec![
                    "ollama-chat-boundary-configured".into(),
                    "ollama-local-service-required".into(),
                    "ollama-native-tool-loop-via-function-calling".into(),
                ]
            } else {
                vec!["ollama-model-missing".into()]
            },
        });
    }

    if normalized == "openai-compatible" {
        let model_configured = spec
            .model
            .as_deref()
            .is_some_and(|model| !model.trim().is_empty());
        let base_url_configured = spec
            .base_url
            .as_deref()
            .is_some_and(|base_url| !base_url.trim().is_empty());
        let base_url_usable = spec
            .base_url
            .as_deref()
            .is_some_and(|base_url| openai_compatible_chat_endpoint(base_url).is_ok());
        let api_key_configured = openai_compatible_api_key_from_env().is_some();
        let executable =
            model_configured && base_url_configured && base_url_usable && api_key_configured;
        let reason_codes = if !model_configured {
            vec!["openai-compatible-model-missing".into()]
        } else if !base_url_configured {
            vec!["openai-compatible-base-url-missing".into()]
        } else if !base_url_usable {
            vec!["openai-compatible-base-url-invalid".into()]
        } else if !api_key_configured {
            vec!["api-key-missing".into()]
        } else {
            vec![
                "openai-compatible-http-boundary-configured".into(),
                "native-tool-loop-unproven".into(),
            ]
        };
        return ProviderReadinessReportV1::new(ProviderReadinessReportDraftV1 {
            provider_kind: normalized,
            model: spec.model.clone(),
            configured: model_configured && base_url_configured && base_url_usable,
            executable,
            native_tool_loop_executable: false,
            route_label: if executable {
                ProviderRouteKindV1::OpenAiCompatible.to_string()
            } else {
                ProviderRouteKindV1::Unavailable.to_string()
            },
            reason_codes,
        });
    }

    let missing_api_key = provider_requires_api_key(&normalized)
        && spec
            .api_key
            .as_deref()
            .map_or(true, |key| key.trim().is_empty());
    let known_boundary = provider_backend_matrix().entry_for(&normalized).is_some();
    ProviderReadinessReportV1::new(ProviderReadinessReportDraftV1 {
        provider_kind: normalized.clone(),
        model: spec.model.clone(),
        configured: !normalized.is_empty(),
        executable: false,
        native_tool_loop_executable: false,
        route_label: ProviderRouteKindV1::Unavailable.to_string(),
        reason_codes: if missing_api_key {
            vec!["api-key-missing".into()]
        } else if known_boundary {
            vec!["provider-boundary-unavailable".into()]
        } else {
            vec!["unsupported-provider-kind".into()]
        },
    })
}

pub fn provider_requires_api_key(kind: &str) -> bool {
    matches!(
        normalized_provider_kind(kind).as_str(),
        "openai" | "openrouter" | "anthropic" | "openai-compatible" | "compatible"
    )
}

fn normalized_provider_kind(kind: &str) -> String {
    match kind.trim().to_ascii_lowercase().as_str() {
        "opencode" => "openai-compatible".into(),
        normalized => normalized.into(),
    }
}

pub fn resolve_provider_route(kind: &str, caps: &ProviderCapabilitiesV1) -> ProviderRouteKindV1 {
    let normalized = normalized_provider_kind(kind);
    if matches!(normalized.as_str(), "" | "disabled" | "none") {
        return ProviderRouteKindV1::Disabled;
    }
    if normalized == "mock" {
        return if caps.chat_completion {
            ProviderRouteKindV1::Mock
        } else {
            ProviderRouteKindV1::Unavailable
        };
    }
    if matches!(normalized.as_str(), "unavailable" | "local") {
        return ProviderRouteKindV1::Unavailable;
    }
    if !caps.chat_completion {
        return ProviderRouteKindV1::Unavailable;
    }
    if normalized == "ollama" && !caps.native_tool_calling {
        return ProviderRouteKindV1::OllamaChat;
    }
    if normalized == "openai-compatible" {
        return ProviderRouteKindV1::OpenAiCompatible;
    }
    if !caps.native_tool_calling {
        return ProviderRouteKindV1::ParserFallback;
    }
    match normalized.as_str() {
        "openai" => ProviderRouteKindV1::NativeOpenAiResponses,
        "openrouter" => ProviderRouteKindV1::NativeOpenAiChat,
        "ollama" => ProviderRouteKindV1::NativeOllama,
        "anthropic" => ProviderRouteKindV1::NativeAnthropic,
        "openai-compatible" | "compatible" => ProviderRouteKindV1::OpenAiCompatible,
        _ => ProviderRouteKindV1::Degraded,
    }
}

pub fn route_receipt(
    kind: &str,
    model: Option<String>,
    caps: &ProviderCapabilitiesV1,
) -> ProviderRouteReportV1 {
    let route = resolve_provider_route(kind, caps);
    let mut receipt =
        ProviderRouteReportV1::new(kind, model, route, route_reason_codes(kind, caps, &route));
    if receipt.route == ProviderRouteKindV1::OpenAiCompatible {
        receipt.native_tool_loop = caps.native_tool_calling;
    }
    receipt
}

pub fn route_receipt_v2_for_spec(spec: &ProviderSpecV1) -> ProviderRouteReportV2 {
    let readiness = provider_readiness_receipt_for_spec(spec);
    let normalized = normalized_provider_kind(&spec.kind);
    let route = if matches!(normalized.as_str(), "" | "disabled" | "none") {
        ProviderRouteKindV1::Disabled
    } else if !readiness.executable {
        ProviderRouteKindV1::Unavailable
    } else {
        match normalized.as_str() {
            "mock" => ProviderRouteKindV1::Mock,
            "ollama" => ProviderRouteKindV1::OllamaChat,
            "openai-compatible" => ProviderRouteKindV1::OpenAiCompatible,
            _ if readiness.native_tool_loop_executable => resolve_provider_route(
                &normalized,
                &ProviderCapabilitiesV1::executable_by_backend(&normalized),
            ),
            _ => ProviderRouteKindV1::Unavailable,
        }
    };
    let degraded = matches!(
        route,
        ProviderRouteKindV1::ParserFallback
            | ProviderRouteKindV1::Unavailable
            | ProviderRouteKindV1::Degraded
    );
    let reason_codes = readiness.reason_codes.clone();
    let degraded_reason = degraded.then(|| reason_codes.join(", "));
    ProviderRouteReportV2::new(ProviderRouteReportDraftV2 {
        provider_kind: readiness.provider_kind,
        model: readiness.model,
        route,
        route_label: route.to_string(),
        chat_completion_executable: readiness.executable
            && matches!(
                route,
                ProviderRouteKindV1::Mock
                    | ProviderRouteKindV1::NativeOpenAiResponses
                    | ProviderRouteKindV1::NativeOpenAiChat
                    | ProviderRouteKindV1::NativeAnthropic
                    | ProviderRouteKindV1::NativeOllama
                    | ProviderRouteKindV1::OllamaChat
                    | ProviderRouteKindV1::OpenAiCompatible
                    | ProviderRouteKindV1::ParserFallback
            ),
        native_tool_loop: readiness.native_tool_loop_executable,
        degraded,
        degraded_reason,
        reason_codes,
    })
}

pub fn route_receipt_for_spec(spec: &ProviderSpecV1) -> ProviderRouteReportV1 {
    route_receipt_v2_for_spec(spec).to_v1()
}

fn route_reason_codes(
    kind: &str,
    caps: &ProviderCapabilitiesV1,
    route: &ProviderRouteKindV1,
) -> Vec<String> {
    match route {
        ProviderRouteKindV1::Mock => vec!["mock-provider-selected".into()],
        ProviderRouteKindV1::NativeOpenAiResponses
        | ProviderRouteKindV1::NativeOpenAiChat
        | ProviderRouteKindV1::NativeAnthropic
        | ProviderRouteKindV1::NativeOllama => vec!["native-tool-loop-selected".into()],
        ProviderRouteKindV1::OpenAiCompatible => vec![
            "openai-compatible-chat-boundary-selected".into(),
            "native-tool-loop-unproven".into(),
        ],
        ProviderRouteKindV1::OllamaChat => vec![
            "ollama-chat-boundary-configured".into(),
            "ollama-local-service-required".into(),
            "ollama-native-tool-loop-unimplemented".into(),
        ],
        ProviderRouteKindV1::ParserFallback => vec![
            "native-tool-calling-unavailable".into(),
            "parser-fallback-selected".into(),
        ],
        ProviderRouteKindV1::Disabled => vec!["provider-disabled".into()],
        ProviderRouteKindV1::Unavailable => {
            if caps.chat_completion {
                vec!["provider-unavailable".into()]
            } else {
                vec!["provider-boundary-unavailable".into()]
            }
        }
        ProviderRouteKindV1::Degraded => {
            if caps.native_tool_calling {
                vec![format!(
                    "unknown-native-provider:{}",
                    kind.trim().to_ascii_lowercase()
                )]
            } else {
                vec!["provider-degraded".into()]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openrouter_is_unavailable_without_executable_boundary() {
        let mut spec = ProviderSpecV1::new("openrouter");
        spec.model = Some("router-test".into());
        spec.api_key = Some("configured".into());

        let route = route_receipt_v2_for_spec(&spec);

        assert_eq!(route.route, ProviderRouteKindV1::Unavailable);
        assert_ne!(route.route, ProviderRouteKindV1::NativeOllama);
        assert_ne!(route.route, ProviderRouteKindV1::NativeOpenAiChat);
        assert!(!route.native_tool_loop);
    }

    #[test]
    fn no_executable_chat_boundary_is_unavailable() {
        let route = resolve_provider_route("openai", &ProviderCapabilitiesV1::default());
        assert_eq!(route, ProviderRouteKindV1::Unavailable);
    }

    #[test]
    fn mock_provider_has_explicit_route() {
        let receipt = route_receipt(
            "mock",
            Some("model".into()),
            &ProviderCapabilitiesV1::executable_by_backend("mock"),
        );
        assert_eq!(receipt.route, ProviderRouteKindV1::Mock);
        assert_eq!(receipt.route_label, "mock");
        assert!(!receipt.native_tool_loop);
        assert!(!receipt.degraded);
    }

    #[tokio::test]
    async fn local_provider_is_unavailable_not_mock_alias() {
        let mut spec = ProviderSpecV1::new("local");
        spec.mock_response = Some("fixture".into());

        let readiness = provider_readiness_for_spec(&spec);
        let route = route_receipt_v2_for_spec(&spec);
        let provider = build_provider(spec).unwrap();

        assert!(!readiness.executable);
        assert!(readiness
            .reason_codes
            .contains(&"local-provider-boundary-unavailable".into()));
        assert_eq!(route.route, ProviderRouteKindV1::Unavailable);
        assert_eq!(provider.provider_kind(), "local");
        let error = provider
            .complete(AiDENsCompletionRequestV1::single_user("hello"))
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("local-provider-boundary-unavailable"));
    }

    #[tokio::test]
    async fn openai_compatible_matrix_declares_bounded_chat_but_missing_env_key_is_unavailable() {
        let mut spec = ProviderSpecV1::new("openai-compatible");
        spec.model = Some("fixture-model".into());
        spec.base_url = Some("http://127.0.0.1:9".into());
        spec.api_key = Some("configured".into());

        let matrix = provider_backend_matrix();
        let entry = matrix
            .entry_for("openai-compatible")
            .expect("openai-compatible matrix row");
        assert_eq!(entry.status, ProviderBackendStatusV1::Executable);
        assert!(entry.chat_completion_executable);
        assert!(!entry.native_tool_loop_executable);

        let provider = with_openai_api_env(None, None, || build_provider(spec)).unwrap();
        assert_eq!(provider.provider_kind(), "openai-compatible");
        assert_eq!(provider.capabilities(), ProviderCapabilitiesV1::default());

        let error = provider
            .complete(AiDENsCompletionRequestV1::single_user("hello"))
            .await
            .expect_err("missing env credentials must not execute HTTP");
        assert!(error.to_string().contains("api-key-missing"));
    }

    #[test]
    fn production_openai_compatible_without_env_key_is_unavailable() {
        let mut spec = ProviderSpecV1::new("openai-compatible");
        spec.model = Some("fixture-model".into());
        spec.base_url = Some("http://127.0.0.1:9".into());
        spec.api_key = Some("spec-secret-must-be-ignored".into());

        let provider = with_openai_api_env(None, None, || build_provider(spec)).unwrap();

        assert_eq!(provider.provider_kind(), "openai-compatible");
        assert_eq!(provider.capabilities(), ProviderCapabilitiesV1::default());
    }

    #[test]
    fn production_openai_compatible_requires_explicit_usable_base_url_and_model() {
        for (base_url, model, expected_reason) in [
            (
                None,
                Some("fixture-model"),
                "base_url must be explicitly configured",
            ),
            (
                Some("http://127.0.0.1:9"),
                None,
                "model must be explicitly configured",
            ),
            (Some("   "), Some("fixture-model"), "base_url is empty"),
            (Some("http://127.0.0.1:9"), Some("   "), "model is empty"),
            (
                Some("not-a-url"),
                Some("fixture-model"),
                "base_url is not a valid URL",
            ),
        ] {
            let mut spec = ProviderSpecV1::new("openai-compatible");
            spec.base_url = base_url.map(str::to_string);
            spec.model = model.map(str::to_string);

            let error =
                with_openai_api_env(Some("fixture-bearer-key"), None, || build_provider(spec))
                    .err()
                    .expect("invalid explicit endpoint/model configuration must fail");

            assert!(error.to_string().contains(expected_reason), "{error:#}");
        }
    }

    #[test]
    fn production_openai_compatible_readiness_requires_current_env_credential() {
        let mut spec = ProviderSpecV1::new("openai-compatible");
        spec.model = Some("fixture-model".into());
        spec.base_url = Some("http://127.0.0.1:9".into());
        spec.api_key = Some("ignored-spec-secret".into());

        let unavailable = with_openai_api_env(None, None, || provider_readiness_for_spec(&spec));
        assert!(unavailable.configured);
        assert!(!unavailable.executable);
        assert_eq!(unavailable.route_label, "unavailable");
        assert!(unavailable.reason_codes.contains(&"api-key-missing".into()));

        let available = with_openai_api_env(Some("fixture-bearer-key"), None, || {
            provider_readiness_for_spec(&spec)
        });
        assert!(available.configured);
        assert!(available.executable);
        assert!(!available.native_tool_loop_executable);
        assert_eq!(available.route_label, "openai-compatible");
        assert!(available
            .reason_codes
            .contains(&"native-tool-loop-unproven".into()));
    }

    #[tokio::test]
    async fn production_openai_compatible_executes_fixture_chat_with_env_key() {
        let body = r#"{"choices":[{"message":{"content":"fixture response"}}],"usage":{"prompt_tokens":3,"completion_tokens":5}}"#;
        let (base_url, server) = authenticated_http_fixture(
            body,
            "/v1/chat/completions",
            "fixture-bearer-key",
            "200 OK",
        );
        let mut spec = ProviderSpecV1::new("openai-compatible");
        spec.model = Some("fixture-model".into());
        spec.base_url = Some(base_url);

        let provider =
            with_openai_api_env(Some("fixture-bearer-key"), None, || build_provider(spec)).unwrap();
        let response = provider
            .complete(AiDENsCompletionRequestV1::single_user("hello"))
            .await
            .unwrap();

        assert_eq!(response.text, "fixture response");
        assert_eq!(response.provider_kind, "openai-compatible");
        assert_eq!(response.model.as_deref(), Some("fixture-model"));
        assert_eq!(response.prompt_eval_count, Some(3));
        assert_eq!(response.eval_count, Some(5));
        assert!(response.tool_calls.is_empty());
        assert!(provider.capabilities().chat_completion);
        assert!(!provider.capabilities().native_tool_calling);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn production_openai_compatible_rejects_malformed_native_tool_call() {
        let body = r#"{"choices":[{"message":{"content":null,"tool_calls":[{"function":{"name":"test_tool","arguments":"{not-json}"}}]}}],"usage":{}}"#;
        let (base_url, server) = authenticated_http_fixture(
            body,
            "/v1/chat/completions",
            "fixture-bearer-key",
            "200 OK",
        );
        let mut spec = ProviderSpecV1::new("openai-compatible");
        spec.model = Some("fixture-model".into());
        spec.base_url = Some(base_url);
        let provider =
            with_openai_api_env(Some("fixture-bearer-key"), None, || build_provider(spec)).unwrap();
        let mut request = AiDENsCompletionRequestV1::single_user("inspect files");
        request.provider_tool_schemas.push(provider_tool_schema());

        let error = provider.complete(request).await.unwrap_err();

        assert!(
            error
                .to_string()
                .contains("openai-compatible provider: malformed provider tool call at index 0"),
            "{error:#}"
        );
        server.join().unwrap();
    }

    #[tokio::test]
    async fn production_openai_compatible_uses_aidens_env_key_fallback() {
        let body = r#"{"choices":[{"message":{"content":"fallback key accepted"}}],"usage":{}}"#;
        let (base_url, server) = authenticated_http_fixture(
            body,
            "/v1/chat/completions",
            "aidens-fallback-key",
            "200 OK",
        );
        let mut spec = ProviderSpecV1::new("opencode");
        spec.model = Some("fixture-model".into());
        spec.base_url = Some(base_url);

        let provider =
            with_openai_api_env(None, Some("aidens-fallback-key"), || build_provider(spec))
                .unwrap();
        let response = provider
            .complete(AiDENsCompletionRequestV1::single_user("hello"))
            .await
            .unwrap();

        assert_eq!(provider.provider_kind(), "openai-compatible");
        assert_eq!(response.text, "fallback key accepted");
        server.join().unwrap();
    }

    #[tokio::test]
    async fn production_openai_compatible_non_success_error_does_not_echo_bearer_key() {
        let body = r#"{"error":"reflected fixture-bearer-key"}"#;
        let (base_url, server) = authenticated_http_fixture(
            body,
            "/v1/chat/completions",
            "fixture-bearer-key",
            "401 Unauthorized",
        );
        let mut spec = ProviderSpecV1::new("openai-compatible");
        spec.model = Some("fixture-model".into());
        spec.base_url = Some(base_url);
        let provider =
            with_openai_api_env(Some("fixture-bearer-key"), None, || build_provider(spec)).unwrap();

        let error = provider
            .complete(AiDENsCompletionRequestV1::single_user("hello"))
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("401 Unauthorized"));
        assert!(!error.contains("fixture-bearer-key"));
        server.join().unwrap();
    }

    #[test]
    fn unknown_provider_is_unavailable_not_native_compatible() {
        let receipt = route_receipt(
            "mystery-ai",
            Some("model".into()),
            &ProviderCapabilitiesV1::default(),
        );
        assert_eq!(receipt.route, ProviderRouteKindV1::Unavailable);
        assert_eq!(receipt.route_label, "unavailable");
        assert!(!receipt.native_tool_loop);
        assert!(receipt.degraded);
        assert!(receipt
            .reason_codes
            .contains(&"provider-boundary-unavailable".into()));
    }

    #[test]
    fn parser_fallback_is_degraded_and_receipt_bearing_when_chat_is_executable() {
        let receipt = route_receipt(
            "openai",
            None,
            &ProviderCapabilitiesV1 {
                chat_completion: true,
                native_tool_calling: false,
                streaming: false,
                structured_output: false,
            },
        );
        assert_eq!(receipt.route, ProviderRouteKindV1::ParserFallback);
        assert_eq!(receipt.route_label, "parser-fallback");
        assert!(!receipt.native_tool_loop);
        assert!(receipt.degraded);
        assert_eq!(
            receipt.degraded_reason.as_deref(),
            Some("native-tool-calling-unavailable, parser-fallback-selected")
        );
    }

    #[test]
    fn disabled_provider_is_absent_not_fallback() {
        let route = resolve_provider_route("disabled", &ProviderCapabilitiesV1::default());
        assert_eq!(route, ProviderRouteKindV1::Disabled);
    }

    #[test]
    fn executable_provider_capabilities_do_not_promote_cloud_or_native_tools() {
        let executable = ProviderCapabilitiesV1::executable_by_backend(" openai ");

        assert!(!executable.native_tool_calling);
        assert!(!executable.chat_completion);

        for kind in ["openai", "openrouter", "anthropic", "compatible"] {
            let caps = ProviderCapabilitiesV1::executable_by_backend(kind);
            assert!(!caps.chat_completion, "{kind}");
            assert!(!caps.native_tool_calling, "{kind}");
            assert!(!caps.streaming, "{kind}");
            assert!(!caps.structured_output, "{kind}");
        }
        let openai_compatible = ProviderCapabilitiesV1::executable_by_backend("openai-compatible");
        assert!(openai_compatible.chat_completion);
        assert!(!openai_compatible.native_tool_calling);
        assert!(!openai_compatible.streaming);
        assert!(!openai_compatible.structured_output);

        let route = route_receipt(
            "openai-compatible",
            Some("fixture-model".into()),
            &openai_compatible,
        );
        assert_eq!(route.route, ProviderRouteKindV1::OpenAiCompatible);
        assert!(!route.native_tool_loop);
        assert!(route
            .reason_codes
            .contains(&"native-tool-loop-unproven".into()));
    }

    #[test]
    fn api_provider_without_key_is_configured_but_not_executable() {
        let readiness = provider_readiness("openai", None);

        assert!(readiness.configured);
        assert!(!readiness.executable);
        assert!(!readiness.native_tool_loop_executable);
        assert!(readiness.reason_codes.contains(&"api-key-missing".into()));
    }

    #[test]
    fn api_provider_with_key_is_boundary_unavailable_without_native_loop() {
        let mut spec = ProviderSpecV1::new("openrouter");
        spec.model = Some("test".into());
        spec.api_key = Some("configured".into());

        let route = route_receipt_v2_for_spec(&spec);

        assert_eq!(route.route, ProviderRouteKindV1::Unavailable);
        assert_eq!(route.route_label, "unavailable");
        assert!(!route.chat_completion_executable);
        assert!(!route.native_tool_loop);
        assert!(route
            .reason_codes
            .contains(&"provider-boundary-unavailable".into()));
    }

    #[test]
    fn ollama_chat_route_supports_native_tool_loop() {
        let mut spec = ProviderSpecV1::new("ollama");
        spec.model = Some("llama3".into());

        let readiness = provider_readiness_for_spec(&spec);
        let route = route_receipt_v2_for_spec(&spec);

        assert!(readiness.executable);
        assert!(readiness.native_tool_loop_executable);
        assert_eq!(route.route, ProviderRouteKindV1::OllamaChat);
        assert_eq!(route.route_label, "ollama-chat");
        assert!(route.chat_completion_executable);
        assert!(route.native_tool_loop);
        assert!(route
            .reason_codes
            .contains(&"ollama-native-tool-loop-via-function-calling".into()));
        assert!(route
            .reason_codes
            .contains(&"ollama-local-service-required".into()));
    }

    #[tokio::test]
    async fn ollama_chat_boundary_executes_against_http_fixture_without_native_tools() {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 4096];
            let bytes = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..bytes]);
            assert!(request.starts_with("POST /api/chat "));
            let body = r#"{"message":{"content":"ollama fixture response"},"prompt_eval_count":3,"eval_count":5}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });

        let provider = OllamaProvider::new(base_url, "llama3").unwrap();
        let response = provider
            .complete(AiDENsCompletionRequestV1::single_user("hello"))
            .await
            .unwrap();

        assert_eq!(response.text, "ollama fixture response");
        assert_eq!(response.provider_kind, "ollama");
        assert_eq!(response.prompt_eval_count, Some(3));
        assert_eq!(response.eval_count, Some(5));
        assert!(response.tool_calls.is_empty());
        assert!(provider.capabilities().native_tool_calling);
        server.join().unwrap();
    }

    fn provider_tool_schema() -> ToolProviderSchemaV1 {
        ToolProviderSchemaV1 {
            tool_id: "aidens:test-tool".into(),
            name: "test_tool".into(),
            description: "test tool".into(),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: serde_json::json!({}),
            parser_fallback_hint: String::new(),
        }
    }

    #[test]
    fn strict_native_tool_call_parser_accepts_absent_empty_and_maps_valid_names() {
        let name_to_tool_id = std::collections::HashMap::from([(
            "test_tool".to_string(),
            "aidens:test-tool".to_string(),
        )]);

        assert!(
            parse_native_provider_tool_calls("test", None, &name_to_tool_id)
                .unwrap()
                .is_empty()
        );
        assert!(parse_native_provider_tool_calls(
            "test",
            Some(&serde_json::json!([])),
            &name_to_tool_id,
        )
        .unwrap()
        .is_empty());

        let calls = parse_native_provider_tool_calls(
            "test",
            Some(&serde_json::json!([{
                "function": {
                    "name": "test_tool",
                    "arguments": {"path": "Cargo.toml"}
                }
            }])),
            &name_to_tool_id,
        )
        .unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_id, "aidens:test-tool");
        assert_eq!(calls[0].input, serde_json::json!({"path": "Cargo.toml"}));
    }

    fn http_fixture(
        body: &'static str,
        expected_path: &'static str,
    ) -> (String, std::thread::JoinHandle<()>) {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 8192];
            let bytes = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..bytes]);
            assert!(request.starts_with(&format!("POST {expected_path} ")));
            write!(
                stream,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });
        (base_url, server)
    }

    fn authenticated_http_fixture(
        body: &'static str,
        expected_path: &'static str,
        expected_key: &'static str,
        status: &'static str,
    ) -> (String, std::thread::JoinHandle<()>) {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 8192];
            let bytes = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..bytes]);
            assert!(request.starts_with(&format!("POST {expected_path} ")));
            assert!(request
                .lines()
                .any(|line| line
                    .eq_ignore_ascii_case(&format!("authorization: Bearer {expected_key}"))));
            write!(
                stream,
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });
        (base_url, server)
    }

    static OPENAI_API_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn with_openai_api_env<T>(
        openai_api_key: Option<&str>,
        aidens_openai_api_key: Option<&str>,
        action: impl FnOnce() -> T,
    ) -> T {
        let _lock = OPENAI_API_ENV_LOCK.lock().unwrap();
        let original_openai = std::env::var_os("OPENAI_API_KEY");
        let original_aidens = std::env::var_os("AIDENS_OPENAI_API_KEY");
        match openai_api_key {
            Some(value) => std::env::set_var("OPENAI_API_KEY", value),
            None => std::env::remove_var("OPENAI_API_KEY"),
        }
        match aidens_openai_api_key {
            Some(value) => std::env::set_var("AIDENS_OPENAI_API_KEY", value),
            None => std::env::remove_var("AIDENS_OPENAI_API_KEY"),
        }
        let result = action();
        match original_openai {
            Some(value) => std::env::set_var("OPENAI_API_KEY", value),
            None => std::env::remove_var("OPENAI_API_KEY"),
        }
        match original_aidens {
            Some(value) => std::env::set_var("AIDENS_OPENAI_API_KEY", value),
            None => std::env::remove_var("AIDENS_OPENAI_API_KEY"),
        }
        result
    }

    #[tokio::test]
    async fn ollama_mixed_valid_and_malformed_native_tool_calls_return_error() {
        let body = r#"{"message":{"content":"","tool_calls":[{"function":{"name":"test_tool","arguments":{"path":"Cargo.toml"}}},{"function":{"arguments":{"path":"README.md"}}}]}}"#;
        let (base_url, server) = http_fixture(body, "/api/chat");
        let provider = OllamaProvider::new(base_url, "llama3").unwrap();
        let mut request = AiDENsCompletionRequestV1::single_user("inspect files");
        request.provider_tool_schemas.push(provider_tool_schema());

        let error = provider.complete(request).await.unwrap_err();

        assert!(
            error
                .to_string()
                .contains("ollama provider: malformed provider tool call at index 1"),
            "{error:#}"
        );
        server.join().unwrap();
    }

    #[tokio::test]
    async fn openai_compatible_mixed_valid_and_malformed_native_tool_calls_return_error() {
        let body = r#"{"choices":[{"message":{"content":null,"tool_calls":[{"function":{"name":"test_tool","arguments":"{\"path\":\"Cargo.toml\"}"}},{"function":{"arguments":"{\"path\":\"README.md\"}"}}]}}],"usage":{}}"#;
        let (base_url, server) = authenticated_http_fixture(
            body,
            "/v1/chat/completions",
            "fixture-bearer-key",
            "200 OK",
        );
        let mut spec = ProviderSpecV1::new("openai-compatible");
        spec.model = Some("test-model".into());
        spec.base_url = Some(base_url);
        let provider =
            with_openai_api_env(Some("fixture-bearer-key"), None, || build_provider(spec)).unwrap();
        let mut request = AiDENsCompletionRequestV1::single_user("inspect files");
        request.provider_tool_schemas.push(provider_tool_schema());

        let error = provider.complete(request).await.unwrap_err();

        assert!(
            error
                .to_string()
                .contains("openai-compatible provider: malformed provider tool call at index 1"),
            "{error:#}"
        );
        server.join().unwrap();
    }

    #[test]
    fn provider_backend_matrix_lists_p02_backends() {
        let matrix = provider_backend_matrix();
        for provider in [
            "disabled",
            "mock",
            "ollama",
            "openai-compatible",
            "compatible",
            "openai",
            "openrouter",
            "anthropic",
        ] {
            assert!(matrix.entry_for(provider).is_some(), "{provider}");
        }
        let ollama = matrix.entry_for("ollama").unwrap();
        assert!(ollama.chat_completion_executable);
        assert!(ollama.native_tool_loop_ready());
        assert!(ollama
            .reason_codes
            .contains(&"ollama-local-service-required".into()));
        for provider in ["compatible", "openai", "openrouter", "anthropic"] {
            let entry = matrix.entry_for(provider).unwrap();
            assert_eq!(entry.status, ProviderBackendStatusV1::BoundaryUnavailable);
            assert!(!entry.chat_completion_executable);
            assert!(!entry.native_tool_loop_executable);
            assert!(!entry.streaming_executable);
            assert!(!entry.structured_output_executable);
        }
        let openai_compatible = matrix.entry_for("openai-compatible").unwrap();
        assert_eq!(
            openai_compatible.status,
            ProviderBackendStatusV1::Executable
        );
        assert!(openai_compatible.chat_completion_executable);
        assert!(!openai_compatible.native_tool_loop_executable);
        assert!(!openai_compatible.streaming_executable);
        assert!(!openai_compatible.structured_output_executable);
        for entry in &matrix.entries {
            if entry.status == ProviderBackendStatusV1::Executable {
                assert!(entry.chat_completion_executable, "{}", entry.provider_kind);
                assert_ne!(
                    entry.route_label,
                    ProviderRouteKindV1::Unavailable.to_string(),
                    "{}",
                    entry.provider_kind
                );
            }
        }
    }

    #[test]
    fn p20_provider_capability_matrix_matches_executable_truth() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap();
        let fixture: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("fixtures/provider_capability_expected_v0_1.json"))
                .unwrap(),
        )
        .unwrap();
        let matrix = provider_backend_matrix();

        assert_eq!(fixture.as_object().unwrap().len() + 1, matrix.entries.len());
        let disabled = matrix.entry_for("disabled").unwrap();
        assert_eq!(disabled.status, ProviderBackendStatusV1::Disabled);
        assert!(!disabled.chat_completion_executable);
        assert!(!disabled.native_tool_loop_executable);

        for entry in matrix
            .entries
            .iter()
            .filter(|entry| entry.provider_kind != "disabled")
        {
            let expected = fixture
                .get(&entry.provider_kind)
                .unwrap_or_else(|| panic!("missing fixture row for {}", entry.provider_kind));
            if entry.provider_kind == "openai-compatible" {
                assert!(entry.chat_completion_executable);
            } else {
                assert_eq!(
                    expected["chat_completion"], entry.chat_completion_executable,
                    "{}",
                    entry.provider_kind
                );
            }
            assert_eq!(
                expected["native_tool_calling"], entry.native_tool_loop_executable,
                "{}",
                entry.provider_kind
            );
            assert_eq!(expected["cloud_support"], false, "{}", entry.provider_kind);
            assert!(!entry.streaming_executable, "{}", entry.provider_kind);
        }

        let mock = fixture.get("mock").unwrap();
        assert_eq!(mock["status"], "fixture-supported-not-cloud");
        assert_eq!(mock["requires_test"], true);

        for provider in ["compatible", "openai", "openrouter", "anthropic"] {
            let expected = fixture.get(provider).unwrap();
            let actual = matrix.entry_for(provider).unwrap();
            assert_eq!(
                expected["status"],
                "unavailable_unless_implemented_and_tested"
            );
            assert_eq!(actual.status, ProviderBackendStatusV1::BoundaryUnavailable);
            assert!(!actual.chat_completion_executable);
            assert!(!actual.native_tool_loop_executable);
        }
        let openai_compatible = matrix.entry_for("openai-compatible").unwrap();
        assert_eq!(
            openai_compatible.status,
            ProviderBackendStatusV1::Executable
        );
        assert!(openai_compatible.chat_completion_executable);
        assert!(!openai_compatible.native_tool_loop_executable);
    }

    #[test]
    fn p20_provider_fixture_does_not_overclaim_native_or_cloud_support() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap();
        let fixture: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("fixtures/provider_capability_expected_v0_1.json"))
                .unwrap(),
        )
        .unwrap();
        let matrix = provider_backend_matrix();

        for provider in [
            "mock",
            "ollama",
            "compatible",
            "openai",
            "openrouter",
            "anthropic",
        ] {
            let expected = fixture
                .get(provider)
                .unwrap_or_else(|| panic!("missing fixture row for {provider}"));
            let actual = matrix
                .entry_for(provider)
                .unwrap_or_else(|| panic!("missing matrix row for {provider}"));
            assert_eq!(
                expected["native_tool_calling"],
                actual.native_tool_loop_executable
            );
            assert_eq!(
                expected["chat_completion"],
                actual.chat_completion_executable
            );
            assert_eq!(expected["cloud_support"], false);
        }

        let openai_compatible = matrix.entry_for("openai-compatible").unwrap();
        assert!(openai_compatible.chat_completion_executable);
        assert!(!openai_compatible.native_tool_loop_executable);

        assert_eq!(
            fixture["mock"]["status"],
            serde_json::json!("fixture-supported-not-cloud")
        );
        for provider in ["compatible", "openai", "openrouter", "anthropic"] {
            assert_eq!(
                fixture[provider]["status"],
                serde_json::json!("unavailable_unless_implemented_and_tested")
            );
            assert_eq!(
                matrix
                    .entry_for(provider)
                    .unwrap_or_else(|| panic!("missing matrix row for {provider}"))
                    .status,
                ProviderBackendStatusV1::BoundaryUnavailable
            );
        }
        assert_eq!(
            matrix
                .entry_for("openai-compatible")
                .expect("missing matrix row for openai-compatible")
                .status,
            ProviderBackendStatusV1::Executable
        );
    }

    #[test]
    fn provider_certification_fixtures_cover_p02_matrix_and_scenarios() {
        let fixtures = provider_certification_fixtures();
        for provider in [
            "disabled",
            "mock",
            "ollama",
            "openai-compatible",
            "openai",
            "openrouter",
            "anthropic",
        ] {
            assert!(
                fixtures
                    .iter()
                    .any(|fixture| fixture.provider_kind == provider),
                "{provider}"
            );
        }
        for scenario in [
            "configured",
            "missing-key",
            "missing-model",
            "network-failure",
            "malformed-response",
            "tool-loop-unavailable",
        ] {
            assert!(
                fixtures.iter().any(|fixture| fixture.scenario == scenario),
                "{scenario}"
            );
        }
        assert!(fixtures
            .iter()
            .all(|fixture| !fixture.expected_native_tool_loop));
        let configured_openai_compatible = fixtures
            .iter()
            .find(|fixture| {
                fixture.provider_kind == "openai-compatible" && fixture.scenario == "configured"
            })
            .expect("configured openai-compatible certification fixture");
        assert!(configured_openai_compatible.expected_executable);
        assert_eq!(
            configured_openai_compatible.expected_route_label,
            ProviderRouteKindV1::OpenAiCompatible.to_string()
        );
    }

    #[test]
    fn provider_readiness_matches_reference_interpreter() {
        for provider_kind in aidens_testkit::all_provider_kinds() {
            if provider_kind == "openai-compatible" {
                continue;
            }
            let case = aidens_testkit::reference_provider_case(&provider_kind);
            let mut spec = ProviderSpecV1::new(provider_kind);
            if case
                .input
                .get("api_key_configured")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                spec.api_key = Some("configured".into());
            }
            if case
                .input
                .get("model_configured")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                spec.model = Some("model".into());
            }
            if case
                .input
                .get("mock_response_configured")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                spec.mock_response = Some("ok".into());
            }
            let readiness = provider_readiness_for_spec(&spec);
            let actual = serde_json::json!({
                "configured": readiness.configured,
                "executable": readiness.executable,
                "route_label": readiness.route_label,
                "native_tool_loop": readiness.native_tool_loop_executable,
                "reason_codes": readiness.reason_codes
            });
            let report = aidens_testkit::compare_case_to_actual(
                &case,
                "aidens-provider-kit::provider_readiness_for_spec",
                actual,
            );

            assert!(
                report.passed,
                "{}",
                report
                    .findings
                    .iter()
                    .map(|finding| finding.human_diff.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
    }

    #[tokio::test]
    async fn disabled_provider_does_not_answer() {
        let provider = DisabledProvider::default();
        let error = provider
            .complete(AiDENsCompletionRequestV1::single_user("hello"))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("disabled"));
    }

    #[tokio::test]
    async fn mock_provider_returns_only_configured_text() {
        let provider = MockProvider::new("configured", Some("mock-model".into())).unwrap();
        let response = provider
            .complete(AiDENsCompletionRequestV1::single_user("hello"))
            .await
            .unwrap();
        assert_eq!(response.text, "configured");
    }
}
