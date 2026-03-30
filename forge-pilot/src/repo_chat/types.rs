use crate::bootstrap::BootstrapRichness;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepoQuestionRoute {
    Navigation,
    Change,
    Structure,
    Ownership,
    DeepSemantic,
    #[default]
    GeneralNonRepo,
}

impl RepoQuestionRoute {
    /// Returns true when this route expects repository-grounded evidence.
    pub fn is_repo_question(self) -> bool {
        self != Self::GeneralNonRepo
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepoChatGroundingMode {
    CurrentStateImported,
    CurrentStateDegraded,
    ThinImportedState,
    ProviderFallbackEligible,
    #[default]
    Ungrounded,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepoChatEvidenceType {
    FilePath,
    Symbol,
    Chunk,
    Delta,
    Manifest,
    Deletion,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct RepoChatEvidence {
    pub manifest_id: String,
    pub path: String,
    pub evidence_type: RepoChatEvidenceType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_start: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_end: Option<usize>,
    pub score: f32,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RepoChatCitation {
    pub path: String,
    pub record_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct RepoChatAnswer {
    pub answer: String,
    pub grounded: bool,
    pub abstained: bool,
    #[serde(default)]
    pub citations: Vec<RepoChatCitation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caveat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_id: Option<String>,
    #[serde(default)]
    pub route: RepoQuestionRoute,
    #[serde(default)]
    pub grounding_mode: RepoChatGroundingMode,
    #[serde(default)]
    pub evidence: Vec<RepoChatEvidence>,
}

impl RepoChatAnswer {
    /// Chooses the grounding mode implied by route and bootstrap richness.
    pub fn thin_or_degraded(
        route: RepoQuestionRoute,
        richness: BootstrapRichness,
    ) -> RepoChatGroundingMode {
        match richness {
            BootstrapRichness::Thin => RepoChatGroundingMode::ThinImportedState,
            BootstrapRichness::Chunked if route == RepoQuestionRoute::DeepSemantic => {
                RepoChatGroundingMode::CurrentStateDegraded
            }
            _ => RepoChatGroundingMode::CurrentStateImported,
        }
    }
}
