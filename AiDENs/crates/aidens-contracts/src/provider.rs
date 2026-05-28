//! Provider route and backend display contracts.
//!
//! AiDENs records local provider readiness and routing evidence here; hosted provider authority remains external and unsupported unless separately proven.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderRouteKindV1 {
    Mock,
    NativeOpenAiResponses,
    NativeOpenAiChat,
    NativeAnthropic,
    NativeOllama,
    OllamaChat,
    OpenAiCompatible,
    ParserFallback,
    Disabled,
    Unavailable,
    Degraded,
}

impl fmt::Display for ProviderRouteKindV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Mock => "mock",
            Self::NativeOpenAiResponses => "native-openai-responses",
            Self::NativeOpenAiChat => "native-openai-chat",
            Self::NativeAnthropic => "native-anthropic",
            Self::NativeOllama => "native-ollama",
            Self::OllamaChat => "ollama-chat",
            Self::OpenAiCompatible => "openai-compatible",
            Self::ParserFallback => "parser-fallback",
            Self::Disabled => "disabled",
            Self::Unavailable => "unavailable",
            Self::Degraded => "degraded",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderRouteReportV1 {
    pub provider_kind: String,
    pub model: Option<String>,
    pub route: ProviderRouteKindV1,
    pub route_label: String,
    pub native_tool_loop: bool,
    #[serde(default)]
    pub degraded: bool,
    pub degraded_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ProviderRouteReportV1 {
    pub fn new(
        provider_kind: impl Into<String>,
        model: Option<String>,
        route: ProviderRouteKindV1,
        reason_codes: Vec<String>,
    ) -> Self {
        let native_tool_loop = matches!(
            route,
            ProviderRouteKindV1::NativeOpenAiResponses
                | ProviderRouteKindV1::NativeOpenAiChat
                | ProviderRouteKindV1::NativeAnthropic
                | ProviderRouteKindV1::NativeOllama
                | ProviderRouteKindV1::OpenAiCompatible
        );
        let degraded = matches!(
            route,
            ProviderRouteKindV1::ParserFallback
                | ProviderRouteKindV1::Unavailable
                | ProviderRouteKindV1::Degraded
        );
        let degraded_reason = degraded.then(|| reason_codes.join(", "));
        Self {
            provider_kind: provider_kind.into(),
            model,
            route_label: route.to_string(),
            route,
            native_tool_loop,
            degraded,
            degraded_reason,
            reason_codes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderBackendStatusV1 {
    Disabled,
    Executable,
    BoundaryUnavailable,
    Unsupported,
}

impl fmt::Display for ProviderBackendStatusV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Disabled => "disabled",
            Self::Executable => "executable",
            Self::BoundaryUnavailable => "boundary-unavailable",
            Self::Unsupported => "unsupported",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderBackendMatrixEntryV1 {
    pub provider_kind: String,
    pub status: ProviderBackendStatusV1,
    pub route_label: String,
    pub api_key_required: bool,
    pub chat_completion_executable: bool,
    pub native_tool_loop_executable: bool,
    pub streaming_executable: bool,
    pub structured_output_executable: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ProviderBackendMatrixEntryV1 {
    pub fn native_tool_loop_ready(&self) -> bool {
        self.status == ProviderBackendStatusV1::Executable && self.native_tool_loop_executable
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderBackendMatrixV1 {
    pub matrix_id: ArtifactId,
    pub generated_at: DateTime<Utc>,
    pub entries: Vec<ProviderBackendMatrixEntryV1>,
}

impl ProviderBackendMatrixV1 {
    pub fn new(entries: Vec<ProviderBackendMatrixEntryV1>) -> Self {
        Self {
            matrix_id: display_only_unstable_id("provider-backend-matrix"),
            generated_at: Utc::now(),
            entries,
        }
    }

    pub fn entry_for(&self, provider_kind: &str) -> Option<&ProviderBackendMatrixEntryV1> {
        let normalized = provider_kind.trim().to_ascii_lowercase();
        self.entries
            .iter()
            .find(|entry| entry.provider_kind == normalized)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderReadinessReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub provider_kind: String,
    pub model: Option<String>,
    pub configured: bool,
    pub executable: bool,
    pub native_tool_loop_executable: bool,
    pub route_label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderReadinessReportDraftV1 {
    pub provider_kind: String,
    pub model: Option<String>,
    pub configured: bool,
    pub executable: bool,
    pub native_tool_loop_executable: bool,
    pub route_label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ProviderReadinessReportV1 {
    pub fn new(draft: ProviderReadinessReportDraftV1) -> Self {
        Self {
            receipt_id: display_only_unstable_id("provider-readiness"),
            kind: ArtifactKindV1::ProviderReadiness,
            provider_kind: draft.provider_kind,
            model: draft.model,
            configured: draft.configured,
            executable: draft.executable,
            native_tool_loop_executable: draft.native_tool_loop_executable,
            route_label: draft.route_label,
            reason_codes: draft.reason_codes,
            checked_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderRouteReportV2 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub provider_kind: String,
    pub model: Option<String>,
    pub route: ProviderRouteKindV1,
    pub route_label: String,
    pub chat_completion_executable: bool,
    pub native_tool_loop: bool,
    #[serde(default)]
    pub degraded: bool,
    pub degraded_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderRouteReportDraftV2 {
    pub provider_kind: String,
    pub model: Option<String>,
    pub route: ProviderRouteKindV1,
    pub route_label: String,
    pub chat_completion_executable: bool,
    pub native_tool_loop: bool,
    pub degraded: bool,
    pub degraded_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ProviderRouteReportV2 {
    pub fn new(draft: ProviderRouteReportDraftV2) -> Self {
        Self {
            receipt_id: display_only_unstable_id("provider-route"),
            kind: ArtifactKindV1::ProviderRoute,
            provider_kind: draft.provider_kind,
            model: draft.model,
            route: draft.route,
            route_label: draft.route_label,
            chat_completion_executable: draft.chat_completion_executable,
            native_tool_loop: draft.native_tool_loop,
            degraded: draft.degraded,
            degraded_reason: draft.degraded_reason,
            reason_codes: draft.reason_codes,
            checked_at: Utc::now(),
        }
    }

    pub fn to_v1(&self) -> ProviderRouteReportV1 {
        ProviderRouteReportV1 {
            provider_kind: self.provider_kind.clone(),
            model: self.model.clone(),
            route: self.route,
            route_label: self.route_label.clone(),
            native_tool_loop: self.native_tool_loop,
            degraded: self.degraded,
            degraded_reason: self.degraded_reason.clone(),
            reason_codes: self.reason_codes.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderCertificationFixtureV1 {
    pub fixture_id: ArtifactId,
    pub provider_kind: String,
    pub scenario: String,
    pub input_config: BTreeMap<String, String>,
    pub expected_configured: bool,
    pub expected_executable: bool,
    pub expected_route_label: String,
    pub expected_native_tool_loop: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderCertificationFixtureDraftV1 {
    pub provider_kind: String,
    pub scenario: String,
    pub input_config: BTreeMap<String, String>,
    pub expected_configured: bool,
    pub expected_executable: bool,
    pub expected_route_label: String,
    pub expected_native_tool_loop: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_reason_codes: Vec<String>,
}

impl ProviderCertificationFixtureV1 {
    pub fn new(draft: ProviderCertificationFixtureDraftV1) -> Self {
        Self {
            fixture_id: display_only_unstable_id("provider-certification-fixture"),
            provider_kind: draft.provider_kind,
            scenario: draft.scenario,
            input_config: draft.input_config,
            expected_configured: draft.expected_configured,
            expected_executable: draft.expected_executable,
            expected_route_label: draft.expected_route_label,
            expected_native_tool_loop: draft.expected_native_tool_loop,
            expected_reason_codes: draft.expected_reason_codes,
        }
    }
}
