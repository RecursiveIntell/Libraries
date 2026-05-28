//! AiDENs boundary policy for agency and influence governance.
//!
//! This crate owns only AiDENs-facing influence classification and gating.
//! It does not define memory, evidence, verification, repair, provider, or
//! tool-runtime truth.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const AGENCY_POLICY_REPORT_V1_SCHEMA: &str = "AgencyPolicyReportV1";
pub const AGENCY_POLICY_CLASSIFIER_V1: &str = "aidens-heuristic-boundary-classifier-v1";
pub const DEFAULT_NUDGE_BUDGET: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgencyPolicyClassifierKindV1 {
    HeuristicBoundaryClassifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InfluenceClassV1 {
    Informational,
    Advice,
    ChoiceArchitecture,
    HighImpactAdvice,
    MemoryPersonalized,
    RepeatedSteering,
    ToolMediatedInfluence,
    DelegatedInfluence,
    Manipulation,
    RelationalBoundary,
    ReceiptPrivacy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgencySurfaceV1 {
    PreGeneration,
    FinalOutput,
    MemoryPersonalization,
    HighImpactRecommendation,
    RepeatedNudge,
    ToolOutput,
    DelegatedAggregation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgencyPolicyOutcomeV1 {
    Allow,
    AllowWithDisclosure,
    RequireAlternatives,
    RequireUserConfirmation,
    DeferToProfessionalOrExternalSource,
    Block,
    Quarantine,
}

impl AgencyPolicyOutcomeV1 {
    pub fn as_policy_label(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::AllowWithDisclosure => "allow_with_disclosure",
            Self::RequireAlternatives => "require_alternatives",
            Self::RequireUserConfirmation => "require_user_confirmation",
            Self::DeferToProfessionalOrExternalSource => "defer_to_professional_or_external_source",
            Self::Block => "block",
            Self::Quarantine => "quarantine",
        }
    }

    pub fn allows_direct_output(self) -> bool {
        matches!(self, Self::Allow | Self::AllowWithDisclosure)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecisionDomainV1 {
    #[default]
    General,
    Employment,
    Finance,
    Legal,
    Medical,
    Housing,
    Relationship,
    Education,
    Safety,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalizationFeatureUseV1 {
    pub feature_id: String,
    pub source: String,
    pub sensitive: bool,
    pub vulnerability_related: bool,
    pub ephemeral_only: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl PersonalizationFeatureUseV1 {
    pub fn sensitive_signal(feature: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            feature_id: feature.into(),
            source: source.into(),
            sensitive: true,
            vulnerability_related: true,
            ephemeral_only: true,
            reason_codes: vec!["sensitive-or-vulnerability-signal".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalizationUsePolicyV1 {
    pub policy_id: String,
    pub allow_sensitive_signal_for_persuasion: bool,
    pub require_memory_influence_trace: bool,
    pub require_redaction_for_receipts: bool,
}

impl Default for PersonalizationUsePolicyV1 {
    fn default() -> Self {
        Self {
            policy_id: "personalization-use-policy:v1".into(),
            allow_sensitive_signal_for_persuasion: false,
            require_memory_influence_trace: true,
            require_redaction_for_receipts: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersuasionBudgetV1 {
    pub budget_id: String,
    pub max_repeated_nudges_without_confirmation: u32,
    pub semantic_paraphrases_count_against_budget: bool,
}

impl Default for PersuasionBudgetV1 {
    fn default() -> Self {
        Self {
            budget_id: "persuasion-budget:v1".into(),
            max_repeated_nudges_without_confirmation: DEFAULT_NUDGE_BUDGET,
            semantic_paraphrases_count_against_budget: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NudgeLedgerV1 {
    counts_by_action_key: BTreeMap<String, u32>,
}

impl NudgeLedgerV1 {
    pub fn prior_count(&self, action_key: &str) -> u32 {
        self.counts_by_action_key
            .get(action_key)
            .copied()
            .unwrap_or_default()
    }

    pub fn observe(&mut self, action_key: impl Into<String>) -> (u32, u32) {
        let action_key = action_key.into();
        let prior = self.prior_count(&action_key);
        let current = prior.saturating_add(1);
        self.counts_by_action_key.insert(action_key, current);
        (prior, current)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NudgeCounterV1 {
    pub receipt_id: String,
    pub action_key: String,
    pub prior_count: u32,
    pub current_count: u32,
    pub max_without_confirmation: u32,
    pub over_budget: bool,
    pub counted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalInfluenceSourceV1 {
    pub receipt_id: String,
    pub source_id: String,
    pub source_kind: String,
    pub content_summary: String,
    pub urgency_or_scarcity_claim: bool,
    pub trusted: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl ExternalInfluenceSourceV1 {
    pub fn tool_output(source_id: impl Into<String>, content: impl Into<String>) -> Self {
        let content_summary = content.into();
        let lower = content_summary.to_ascii_lowercase();
        let urgency_or_scarcity_claim = contains_any(
            &lower,
            &[
                "urgent",
                "limited time",
                "scarcity",
                "only today",
                "act now",
                "deadline",
            ],
        );
        Self {
            receipt_id: receipt_id("external-influence-source"),
            source_id: source_id.into(),
            source_kind: "tool-output".into(),
            content_summary,
            urgency_or_scarcity_claim,
            trusted: false,
            reason_codes: if urgency_or_scarcity_claim {
                vec!["tool-output-urgency-or-scarcity".into()]
            } else {
                Vec::new()
            },
            recorded_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlternativeOptionV1 {
    pub option_id: String,
    pub label: String,
    pub viable: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlternativeSetV1 {
    pub receipt_id: String,
    pub alternatives: Vec<AlternativeOptionV1>,
    pub viable_count: u32,
    pub decorative_alternatives_detected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeoffMatrixV1 {
    pub receipt_id: String,
    pub dimensions: Vec<String>,
    pub option_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecommendationTraceV1 {
    pub trace_id: String,
    pub action_key: String,
    pub single_path: bool,
    pub high_impact: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdviceEnvelopeV1 {
    pub envelope_id: String,
    pub decision_domain: DecisionDomainV1,
    pub recommendation_present: bool,
    pub high_impact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionSupportEnvelopeV1 {
    pub envelope_id: String,
    pub alternative_set_receipt_id: Option<String>,
    pub tradeoff_matrix_receipt_id: Option<String>,
    pub requires_viable_alternatives: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighImpactGateV1 {
    pub gate_id: String,
    pub decision_domain: DecisionDomainV1,
    pub triggered: bool,
    pub requires_alternatives: bool,
    pub requires_user_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfluenceReceiptV1 {
    pub receipt_id: String,
    pub classes: Vec<InfluenceClassV1>,
    pub risk_surface: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdviceReceiptV1 {
    pub receipt_id: String,
    pub advice_envelope_id: String,
    pub recommendation_trace_id: String,
    pub decision_domain: DecisionDomainV1,
    pub high_impact: bool,
    pub single_path_recommendation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighImpactRecommendationReceiptV1 {
    pub receipt_id: String,
    pub high_impact_gate_id: String,
    pub decision_domain: DecisionDomainV1,
    pub required_outcome: AgencyPolicyOutcomeV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryInfluenceTraceV1 {
    pub receipt_id: String,
    pub features: Vec<PersonalizationFeatureUseV1>,
    pub used_for_recommendation: bool,
    pub sensitive_signal_used: bool,
    pub redacted_feature_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensitiveSignalRetentionPolicyV1 {
    pub receipt_id: String,
    pub raw_sensitive_signal_retained: bool,
    pub redacted_receipt_required: bool,
    pub ephemeral_context_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepeatedSteeringReceiptV1 {
    pub receipt_id: String,
    pub nudge_counter_receipt_id: String,
    pub action_key: String,
    pub outcome: AgencyPolicyOutcomeV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolOutputPersuasionRiskV1 {
    pub receipt_id: String,
    pub source_receipt_ids: Vec<String>,
    pub urgency_or_scarcity_detected: bool,
    pub untrusted_source_count: u32,
    pub outcome: AgencyPolicyOutcomeV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegatedInfluencePolicyV1 {
    pub receipt_id: String,
    pub aggregate_delegated_outputs: bool,
    pub max_unconfirmed_repeated_recommendations: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfluenceAggregationReceiptV1 {
    pub receipt_id: String,
    pub delegated_output_count: u32,
    pub repeated_recommendation_count: u32,
    pub outcome: AgencyPolicyOutcomeV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EphemeralContextReceiptV1 {
    pub receipt_id: String,
    pub context_class: String,
    pub retained_after_turn: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedInfluenceReceiptV1 {
    pub receipt_id: String,
    pub source_receipt_id: Option<String>,
    pub redaction_reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgencyIncidentRecordV1 {
    pub receipt_id: String,
    pub incident_class: String,
    pub outcome: AgencyPolicyOutcomeV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_behavior: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgencyPolicyDecisionV1 {
    pub decision_id: String,
    pub outcome: AgencyPolicyOutcomeV1,
    pub output_allowed: bool,
    pub disclosure_required: bool,
    pub user_confirmation_required: bool,
    pub alternatives_required: bool,
    pub blocked: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgencyPolicyInputV1 {
    pub surface: AgencySurfaceV1,
    pub prompt: String,
    pub user_goal: Option<String>,
    pub candidate_output: Option<String>,
    pub assistant_behavior: Option<String>,
    pub risk_surface: Option<String>,
    pub decision_domain: DecisionDomainV1,
    pub high_impact: bool,
    pub recommendation_present: bool,
    pub single_path_recommendation: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<String>,
    pub prior_nudges: u32,
    pub nudge_action_key: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memory_features: Vec<PersonalizationFeatureUseV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_sources: Vec<ExternalInfluenceSourceV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delegated_outputs: Vec<String>,
    pub requested_manipulation: bool,
    pub exit_or_stop_request: bool,
    pub receipt_privacy_required: bool,
}

impl AgencyPolicyInputV1 {
    pub fn for_runner_final_output(
        prompt: impl Into<String>,
        candidate_output: impl Into<String>,
        tool_results: &[String],
    ) -> Self {
        let prompt = prompt.into();
        let candidate_output = candidate_output.into();
        let combined = format!("{prompt}\n{candidate_output}");
        let tool_sources = tool_results
            .iter()
            .enumerate()
            .map(|(index, text)| {
                ExternalInfluenceSourceV1::tool_output(format!("runner-tool-result-{index}"), text)
            })
            .collect::<Vec<_>>();
        let memory_features = infer_runner_personalization_features(&combined);
        Self {
            surface: AgencySurfaceV1::FinalOutput,
            user_goal: Some(prompt.clone()),
            prompt,
            candidate_output: Some(candidate_output.clone()),
            assistant_behavior: Some(candidate_output),
            risk_surface: None,
            decision_domain: classify_domain(&combined),
            high_impact: looks_high_impact(&combined),
            recommendation_present: looks_like_recommendation(&combined),
            single_path_recommendation: looks_single_path(&combined),
            alternatives: extract_inline_alternatives(&combined),
            prior_nudges: 0,
            nudge_action_key: Some(semantic_action_key(&combined)),
            memory_features,
            tool_sources,
            delegated_outputs: Vec::new(),
            requested_manipulation: looks_requested_manipulation(&combined),
            exit_or_stop_request: looks_exit_or_stop(&combined),
            receipt_privacy_required: false,
        }
    }

    pub fn for_runner_tool_output(
        prompt: impl Into<String>,
        tool_output: impl Into<String>,
    ) -> Self {
        let prompt = prompt.into();
        let tool_output = tool_output.into();
        let combined = format!("{prompt}\n{tool_output}");
        Self {
            surface: AgencySurfaceV1::ToolOutput,
            user_goal: Some(prompt.clone()),
            prompt,
            candidate_output: None,
            assistant_behavior: None,
            risk_surface: Some("tool_mediated_influence".into()),
            decision_domain: classify_domain(&combined),
            high_impact: looks_high_impact(&combined),
            recommendation_present: looks_like_recommendation(&combined),
            single_path_recommendation: false,
            alternatives: Vec::new(),
            prior_nudges: 0,
            nudge_action_key: Some(semantic_action_key(&combined)),
            memory_features: Vec::new(),
            tool_sources: vec![ExternalInfluenceSourceV1::tool_output(
                "runner-tool-output",
                tool_output,
            )],
            delegated_outputs: Vec::new(),
            requested_manipulation: false,
            exit_or_stop_request: false,
            receipt_privacy_required: false,
        }
    }

    pub fn from_eval_case(id: &str, risk_surface: &str, input: &Value) -> Result<Self, String> {
        let user_goal = string_field(input, "user_goal")
            .or_else(|| string_field(input, "user"))
            .or_else(|| string_field(input, "recommendation"))
            .or_else(|| string_field(input, "new_nudge"))
            .or_else(|| string_field(input, "tool_output"))
            .or_else(|| string_field(input, "user_bad_plan"))
            .or_else(|| Some(id.to_string()));
        let assistant_behavior = string_field(input, "assistant_behavior");
        let recommendation = string_field(input, "recommendation");
        let new_nudge = string_field(input, "new_nudge");
        let tool_output = string_field(input, "tool_output");
        let sensitive_memory = string_field(input, "sensitive_memory");
        let user_bad_plan = string_field(input, "user_bad_plan");
        let user_plan = string_field(input, "user_plan");
        let user_context = string_field(input, "user_context");
        let source = string_field(input, "source");
        let target_action = string_field(input, "target_action");
        let reversibility_disclosure = string_field(input, "reversibility_disclosure");
        let delegated_outputs = string_array_field(input, "subagent_outputs");
        let mut combined_parts = [
            user_goal.as_deref().unwrap_or_default(),
            assistant_behavior.as_deref().unwrap_or_default(),
            recommendation.as_deref().unwrap_or_default(),
            new_nudge.as_deref().unwrap_or_default(),
            tool_output.as_deref().unwrap_or_default(),
            sensitive_memory.as_deref().unwrap_or_default(),
            user_bad_plan.as_deref().unwrap_or_default(),
            user_plan.as_deref().unwrap_or_default(),
            user_context.as_deref().unwrap_or_default(),
            source.as_deref().unwrap_or_default(),
            target_action.as_deref().unwrap_or_default(),
            reversibility_disclosure.as_deref().unwrap_or_default(),
        ]
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
        combined_parts.extend(delegated_outputs.iter().cloned());
        let combined = combined_parts.join("\n");
        let alternatives = string_array_field(input, "alternatives");
        let memory_features = string_array_field(input, "memory_used")
            .into_iter()
            .chain(sensitive_memory.iter().cloned())
            .map(|item| {
                PersonalizationFeatureUseV1::sensitive_signal(
                    redacted_feature_id(&item),
                    "eval-case",
                )
            })
            .collect::<Vec<_>>();
        let tool_sources = tool_output
            .iter()
            .cloned()
            .map(|text| ExternalInfluenceSourceV1::tool_output(format!("eval:{id}:tool"), text))
            .collect::<Vec<_>>();
        let receipt_privacy_required = bool_field(input, "receipt_needed")
            || sensitive_memory.is_some()
            || id.contains("sensitive_signal");
        let high_impact_surface = matches!(
            risk_surface,
            "high_impact_advice"
                | "high_impact_recommendation"
                | "financial_high_impact"
                | "medical_high_impact"
                | "legal_high_impact"
                | "decision_reversibility"
                | "subagent_merge_high_impact"
                | "vulnerability_context"
        );
        Ok(Self {
            surface: surface_for_risk(risk_surface),
            prompt: user_goal.clone().unwrap_or_default(),
            user_goal,
            candidate_output: recommendation
                .clone()
                .or_else(|| assistant_behavior.clone()),
            assistant_behavior,
            risk_surface: Some(risk_surface.to_string()),
            decision_domain: classify_domain(&combined),
            high_impact: high_impact_surface || looks_high_impact(&combined),
            recommendation_present: recommendation.is_some()
                || risk_surface == "agency_degradation"
                || risk_surface == "alternative_set_integrity"
                || risk_surface.ends_with("_high_impact")
                || matches!(
                    risk_surface,
                    "decision_reversibility" | "vulnerability_context"
                )
                || looks_like_recommendation(&combined),
            single_path_recommendation: looks_single_path(&combined) || high_impact_surface,
            alternatives,
            prior_nudges: number_field(input, "prior_nudges").unwrap_or_default(),
            nudge_action_key: Some(
                target_action
                    .as_deref()
                    .map(semantic_action_key)
                    .unwrap_or_else(|| semantic_action_key(&combined)),
            ),
            memory_features,
            tool_sources,
            delegated_outputs,
            requested_manipulation: looks_requested_manipulation(&combined)
                || risk_surface == "user_requested_manipulation",
            exit_or_stop_request: looks_exit_or_stop(&combined)
                || matches!(risk_surface, "relational_boundary" | "exit_respect"),
            receipt_privacy_required,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgencyReceiptBundleV1 {
    pub influence_receipt: InfluenceReceiptV1,
    pub decision: AgencyPolicyDecisionV1,
    pub recommendation_trace: Option<RecommendationTraceV1>,
    pub advice_envelope: Option<AdviceEnvelopeV1>,
    pub advice_receipt: Option<AdviceReceiptV1>,
    pub decision_support_envelope: Option<DecisionSupportEnvelopeV1>,
    pub high_impact_gate: Option<HighImpactGateV1>,
    pub high_impact_receipt: Option<HighImpactRecommendationReceiptV1>,
    pub memory_trace: Option<MemoryInfluenceTraceV1>,
    pub personalization_policy: Option<PersonalizationUsePolicyV1>,
    pub sensitive_signal_policy: Option<SensitiveSignalRetentionPolicyV1>,
    pub nudge_counter: Option<NudgeCounterV1>,
    pub repeated_steering_receipt: Option<RepeatedSteeringReceiptV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_sources: Vec<ExternalInfluenceSourceV1>,
    pub tool_output_persuasion_risk: Option<ToolOutputPersuasionRiskV1>,
    pub delegated_influence_policy: Option<DelegatedInfluencePolicyV1>,
    pub influence_aggregation_receipt: Option<InfluenceAggregationReceiptV1>,
    pub ephemeral_context_receipt: Option<EphemeralContextReceiptV1>,
    pub redacted_influence_receipt: Option<RedactedInfluenceReceiptV1>,
    pub agency_incident_record: Option<AgencyIncidentRecordV1>,
    pub alternative_set: Option<AlternativeSetV1>,
    pub tradeoff_matrix: Option<TradeoffMatrixV1>,
}

impl AgencyReceiptBundleV1 {
    pub fn receipt_ids(&self) -> Vec<String> {
        let mut ids = vec![
            self.influence_receipt.receipt_id.clone(),
            self.decision.decision_id.clone(),
        ];
        if let Some(receipt) = &self.advice_receipt {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.high_impact_receipt {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.memory_trace {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.sensitive_signal_policy {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.nudge_counter {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.repeated_steering_receipt {
            ids.push(receipt.receipt_id.clone());
        }
        ids.extend(
            self.external_sources
                .iter()
                .map(|source| source.receipt_id.clone()),
        );
        if let Some(receipt) = &self.tool_output_persuasion_risk {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.delegated_influence_policy {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.influence_aggregation_receipt {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.ephemeral_context_receipt {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.redacted_influence_receipt {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.agency_incident_record {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.alternative_set {
            ids.push(receipt.receipt_id.clone());
        }
        if let Some(receipt) = &self.tradeoff_matrix {
            ids.push(receipt.receipt_id.clone());
        }
        ids
    }

    pub fn receipt_schema_names(&self) -> BTreeSet<String> {
        let mut names = BTreeSet::from([
            "InfluenceReceiptV1".to_string(),
            "AgencyPolicyDecisionV1".to_string(),
        ]);
        if self.recommendation_trace.is_some() {
            names.insert("RecommendationTraceV1".into());
        }
        if self.advice_envelope.is_some() {
            names.insert("AdviceEnvelopeV1".into());
        }
        if self.advice_receipt.is_some() {
            names.insert("AdviceReceiptV1".into());
        }
        if self.decision_support_envelope.is_some() {
            names.insert("DecisionSupportEnvelopeV1".into());
        }
        if self.high_impact_gate.is_some() {
            names.insert("HighImpactGateV1".into());
        }
        if self.high_impact_receipt.is_some() {
            names.insert("HighImpactRecommendationReceiptV1".into());
        }
        if self.memory_trace.is_some() {
            names.insert("MemoryInfluenceTraceV1".into());
        }
        if self.personalization_policy.is_some() {
            names.insert("PersonalizationUsePolicyV1".into());
        }
        if self.sensitive_signal_policy.is_some() {
            names.insert("SensitiveSignalRetentionPolicyV1".into());
        }
        if self.nudge_counter.is_some() {
            names.insert("NudgeCounterV1".into());
        }
        if self.repeated_steering_receipt.is_some() {
            names.insert("RepeatedSteeringReceiptV1".into());
        }
        if !self.external_sources.is_empty() {
            names.insert("ExternalInfluenceSourceV1".into());
        }
        if self.tool_output_persuasion_risk.is_some() {
            names.insert("ToolOutputPersuasionRiskV1".into());
        }
        if self.delegated_influence_policy.is_some() {
            names.insert("DelegatedInfluencePolicyV1".into());
        }
        if self.influence_aggregation_receipt.is_some() {
            names.insert("InfluenceAggregationReceiptV1".into());
        }
        if self.ephemeral_context_receipt.is_some() {
            names.insert("EphemeralContextReceiptV1".into());
        }
        if self.redacted_influence_receipt.is_some() {
            names.insert("RedactedInfluenceReceiptV1".into());
        }
        if self.agency_incident_record.is_some() {
            names.insert("AgencyIncidentRecordV1".into());
        }
        if self.alternative_set.is_some() {
            names.insert("AlternativeSetV1".into());
        }
        if self.tradeoff_matrix.is_some() {
            names.insert("TradeoffMatrixV1".into());
        }
        names
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgencyPolicyReportV1 {
    pub schema_version: String,
    pub classifier_id: String,
    pub classifier_kind: AgencyPolicyClassifierKindV1,
    pub report_id: String,
    pub surface: AgencySurfaceV1,
    pub risk_surface: String,
    pub classes: Vec<InfluenceClassV1>,
    pub decision: AgencyPolicyDecisionV1,
    pub outcome: AgencyPolicyOutcomeV1,
    pub receipts: AgencyReceiptBundleV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_behavior: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disclosure_notes: Vec<String>,
    pub evaluated_at: DateTime<Utc>,
}

impl AgencyPolicyReportV1 {
    pub fn receipt_ids(&self) -> Vec<String> {
        self.receipts.receipt_ids()
    }

    pub fn receipt_schema_names(&self) -> BTreeSet<String> {
        self.receipts.receipt_schema_names()
    }

    pub fn reason_codes(&self) -> &[String] {
        &self.decision.reason_codes
    }

    pub fn disclosure_text(&self) -> String {
        if self.disclosure_notes.is_empty() {
            "Agency disclosure: influence risk was classified before output.".into()
        } else {
            format!("Agency disclosure: {}", self.disclosure_notes.join("; "))
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgencyPolicyEngineV1 {
    pub personalization_policy: PersonalizationUsePolicyV1,
    pub persuasion_budget: PersuasionBudgetV1,
}

impl AgencyPolicyEngineV1 {
    pub fn evaluate(
        &self,
        input: &AgencyPolicyInputV1,
        ledger: &mut NudgeLedgerV1,
    ) -> AgencyPolicyReportV1 {
        let mut classes = BTreeSet::<InfluenceClassV1>::new();
        let mut reason_codes = BTreeSet::<String>::new();
        let mut blocked_behavior = BTreeSet::<String>::new();
        let mut disclosure_notes = Vec::<String>::new();
        let mut outcome = AgencyPolicyOutcomeV1::Allow;
        let lower = policy_text(input);

        if input.recommendation_present {
            classes.insert(InfluenceClassV1::Advice);
            reason_codes.insert("advice-or-recommendation-detected".into());
        } else {
            classes.insert(InfluenceClassV1::Informational);
        }

        let alternatives = build_alternative_set(&input.alternatives);
        if alternatives
            .as_ref()
            .map(|set| set.decorative_alternatives_detected)
            .unwrap_or(false)
        {
            classes.insert(InfluenceClassV1::ChoiceArchitecture);
            reason_codes.insert("decorative-alternatives-detected".into());
            blocked_behavior.insert("decorative_alternatives".into());
            outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::RequireAlternatives);
        }

        if input.high_impact && input.recommendation_present {
            classes.insert(InfluenceClassV1::HighImpactAdvice);
            reason_codes.insert("high-impact-recommendation-gate".into());
            if input.single_path_recommendation
                || alternatives
                    .as_ref()
                    .map(|set| set.viable_count < 2)
                    .unwrap_or(true)
            {
                reason_codes.insert("high-impact-single-path-recommendation".into());
                blocked_behavior.insert("single_path_recommendation".into());
                outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::RequireAlternatives);
            }
            match input.decision_domain {
                DecisionDomainV1::Medical => {
                    reason_codes.insert("medical-professional-review-required".into());
                    blocked_behavior.insert("medical_directive_without_professional".into());
                    outcome = stronger_outcome(
                        outcome,
                        AgencyPolicyOutcomeV1::DeferToProfessionalOrExternalSource,
                    );
                }
                DecisionDomainV1::Legal => {
                    reason_codes.insert("legal-professional-review-required".into());
                    blocked_behavior.insert("legal_directive".into());
                    outcome = stronger_outcome(
                        outcome,
                        AgencyPolicyOutcomeV1::DeferToProfessionalOrExternalSource,
                    );
                }
                DecisionDomainV1::Finance
                    if contains_any(&lower, &["all savings", "all-in", "all in"]) =>
                {
                    reason_codes.insert("all-in-financial-recommendation".into());
                    blocked_behavior.insert("all_in_financial_advice".into());
                    outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::RequireAlternatives);
                }
                _ => {}
            }
        }

        if contains_any(
            &lower,
            &["urgent", "today", "right now", "act now", "limited time"],
        ) {
            reason_codes.insert("urgency-or-time-pressure-detected".into());
            if input.high_impact || !input.tool_sources.is_empty() {
                blocked_behavior.insert(if input.tool_sources.is_empty() {
                    "false_urgency".into()
                } else {
                    "unverified_urgency".into()
                });
                outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::AllowWithDisclosure);
                disclosure_notes
                    .push("urgency/scarcity claims are treated as influence risk".into());
            }
        }

        let action_key = input
            .nudge_action_key
            .clone()
            .unwrap_or_else(|| semantic_action_key(&lower));
        let ledger_prior = ledger.prior_count(&action_key);
        let looks_nudge = input.prior_nudges > 0
            || ledger_prior > 0
            || input.surface == AgencySurfaceV1::RepeatedNudge
            || lower.contains("nudge")
            || (input.recommendation_present && input.single_path_recommendation);
        let nudge_counter = looks_nudge.then(|| {
            let prior = input.prior_nudges.max(ledger_prior);
            let current = prior.saturating_add(1);
            if prior == ledger_prior {
                ledger.observe(action_key.clone());
            }
            NudgeCounterV1 {
                receipt_id: receipt_id("nudge-counter"),
                action_key: action_key.clone(),
                prior_count: prior,
                current_count: current,
                max_without_confirmation: self
                    .persuasion_budget
                    .max_repeated_nudges_without_confirmation,
                over_budget: current
                    > self
                        .persuasion_budget
                        .max_repeated_nudges_without_confirmation,
                counted_at: Utc::now(),
            }
        });
        if let Some(counter) = &nudge_counter {
            classes.insert(InfluenceClassV1::RepeatedSteering);
            reason_codes.insert("nudge-counter-updated".into());
            if counter.over_budget {
                reason_codes.insert("repeated-nudge-budget-exceeded".into());
                blocked_behavior.insert("counter_bypass_by_paraphrase".into());
                outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::RequireUserConfirmation);
            }
        }

        let sensitive_signal_used = input
            .memory_features
            .iter()
            .any(|feature| feature.sensitive || feature.vulnerability_related);
        if !input.memory_features.is_empty() {
            classes.insert(InfluenceClassV1::MemoryPersonalized);
            reason_codes.insert("memory-influence-trace-required".into());
            if sensitive_signal_used && input.recommendation_present {
                reason_codes.insert("sensitive-memory-used-for-recommendation".into());
                blocked_behavior.insert("exploit_vulnerability".into());
                outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::Block);
            }
        }

        let tool_urgency = input
            .tool_sources
            .iter()
            .any(|source| source.urgency_or_scarcity_claim);
        let tool_conflict = input.tool_sources.iter().any(|source| {
            contains_any(
                &source.content_summary.to_ascii_lowercase(),
                &["paid for placement", "sponsored", "advertiser", "vendor"],
            )
        });
        if !input.tool_sources.is_empty() {
            classes.insert(InfluenceClassV1::ToolMediatedInfluence);
            reason_codes.insert("tool-output-influence-classified".into());
            if tool_urgency {
                blocked_behavior.insert("unverified_urgency".into());
                outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::AllowWithDisclosure);
                disclosure_notes.push(
                    "tool-origin urgency/scarcity was not accepted as verified urgency".into(),
                );
            }
            if tool_conflict {
                blocked_behavior.insert("hidden_conflict".into());
                blocked_behavior.insert("hidden_external_influence".into());
                reason_codes.insert("tool-output-conflict-disclosure-required".into());
                outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::AllowWithDisclosure);
                disclosure_notes.push(
                    "tool-origin source or conflict was treated as influence evidence".into(),
                );
            }
        }

        if !input.delegated_outputs.is_empty() {
            classes.insert(InfluenceClassV1::DelegatedInfluence);
            reason_codes.insert("delegated-influence-aggregated".into());
            let delegated_text = input.delegated_outputs.join("\n").to_ascii_lowercase();
            if looks_high_impact(&delegated_text) {
                reason_codes.insert("delegated-high-impact-merge".into());
                blocked_behavior.insert("unreviewed_high_impact_merge".into());
            }
            if input.delegated_outputs.len() >= 3 {
                blocked_behavior.insert("aggregate_pressure_untracked".into());
                outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::RequireUserConfirmation);
            }
        }

        if input.exit_or_stop_request
            && contains_any(
                &lower,
                &["guilt", "fomo", "farewell", "do not leave", "don't leave"],
            )
        {
            classes.insert(InfluenceClassV1::RelationalBoundary);
            blocked_behavior.insert("exit_resistance".into());
            blocked_behavior.insert("guilt_hook".into());
            reason_codes.insert("exit-resistance-or-guilt-hook".into());
            outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::Block);
        }

        if contains_any(
            &lower,
            &[
                "you do not need anyone else",
                "you don't need anyone else",
                "i am always the one who understands you",
                "only one who understands you",
                "depend on me",
                "you need me",
            ],
        ) {
            classes.insert(InfluenceClassV1::RelationalBoundary);
            blocked_behavior.insert("dependency_loop".into());
            blocked_behavior.insert("exclusivity_cue".into());
            reason_codes.insert("emotional-dependence-risk".into());
            outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::Block);
        }

        if contains_any(
            &lower,
            &["lose your progress", "regret leaving", "regret it"],
        ) && (input.exit_or_stop_request
            || contains_any(&lower, &["stop using", "leave this app"]))
        {
            classes.insert(InfluenceClassV1::RelationalBoundary);
            blocked_behavior.insert("retention_hook".into());
            blocked_behavior.insert("exit_resistance".into());
            reason_codes.insert("retention-manipulation-risk".into());
            outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::Block);
        }

        if input.requested_manipulation {
            classes.insert(InfluenceClassV1::Manipulation);
            blocked_behavior.insert("manipulative_copy_generation".into());
            reason_codes.insert("user-requested-manipulation".into());
            outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::Block);
        }

        if contains_any(
            &lower,
            &[
                "minor",
                "hide this from your parents",
                "do not tell your parents",
            ],
        ) {
            classes.insert(InfluenceClassV1::Manipulation);
            blocked_behavior.insert("exploit_minor_vulnerability".into());
            reason_codes.insert("minor-vulnerability-exploitation-risk".into());
            outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::Block);
        }

        if input.recommendation_present
            && contains_any(
                &lower,
                &["permanently", "irreversible", "delete the archive"],
            )
            && !contains_any(&lower, &["backup", "restore point", "reversible"])
        {
            classes.insert(InfluenceClassV1::Advice);
            blocked_behavior.insert("irreversible_action_without_disclosure".into());
            reason_codes.insert("reversibility-disclosure-required".into());
            outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::RequireAlternatives);
        }

        if contains_any(
            &lower,
            &[
                "over-validates",
                "ignore all external advice",
                "risky thing",
                "you are definitely right",
            ],
        ) {
            classes.insert(InfluenceClassV1::Advice);
            blocked_behavior.insert("sycophantic_overvalidation".into());
            reason_codes.insert("sycophancy-overvalidation-risk".into());
            outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::RequireAlternatives);
        }

        if input.receipt_privacy_required {
            classes.insert(InfluenceClassV1::ReceiptPrivacy);
            blocked_behavior.insert("unredacted_sensitive_receipt".into());
            reason_codes.insert("sensitive-receipt-redaction-required".into());
            outcome = stronger_outcome(outcome, AgencyPolicyOutcomeV1::AllowWithDisclosure);
            disclosure_notes.push("sensitive influence evidence is redacted in receipts".into());
        }

        let classes = classes.into_iter().collect::<Vec<_>>();
        let reason_codes = reason_codes.into_iter().collect::<Vec<_>>();
        let blocked_behavior = blocked_behavior.into_iter().collect::<Vec<_>>();
        let report_material = format!(
            "input={input:?};classes={classes:?};outcome={outcome:?};reason_codes={reason_codes:?};blocked_behavior={blocked_behavior:?};nudge_counter={:?}",
            nudge_counter
                .as_ref()
                .map(|counter| (&counter.action_key, counter.prior_count, counter.current_count))
        );
        let decision = AgencyPolicyDecisionV1 {
            decision_id: receipt_id("agency-policy-decision"),
            outcome,
            output_allowed: outcome.allows_direct_output(),
            disclosure_required: outcome == AgencyPolicyOutcomeV1::AllowWithDisclosure,
            user_confirmation_required: outcome == AgencyPolicyOutcomeV1::RequireUserConfirmation,
            alternatives_required: outcome == AgencyPolicyOutcomeV1::RequireAlternatives,
            blocked: matches!(
                outcome,
                AgencyPolicyOutcomeV1::Block | AgencyPolicyOutcomeV1::Quarantine
            ),
            reason_codes: reason_codes.clone(),
            decided_at: Utc::now(),
        };

        let risk_surface = input
            .risk_surface
            .clone()
            .unwrap_or_else(|| risk_surface_for(&classes).into());
        let recommendation_trace = (input.recommendation_present
            || input.single_path_recommendation)
            .then(|| RecommendationTraceV1 {
                trace_id: receipt_id("recommendation-trace"),
                action_key: action_key.clone(),
                single_path: input.single_path_recommendation,
                high_impact: input.high_impact,
                reason_codes: reason_codes.clone(),
            });
        let advice_envelope =
            (input.recommendation_present || input.high_impact).then(|| AdviceEnvelopeV1 {
                envelope_id: receipt_id("advice-envelope"),
                decision_domain: input.decision_domain,
                recommendation_present: input.recommendation_present,
                high_impact: input.high_impact,
            });
        let high_impact_gate = input.high_impact.then(|| HighImpactGateV1 {
            gate_id: receipt_id("high-impact-gate"),
            decision_domain: input.decision_domain,
            triggered: input.high_impact,
            requires_alternatives: outcome == AgencyPolicyOutcomeV1::RequireAlternatives,
            requires_user_confirmation: outcome == AgencyPolicyOutcomeV1::RequireUserConfirmation,
        });
        let alternative_set = if outcome == AgencyPolicyOutcomeV1::RequireAlternatives {
            Some(alternatives.unwrap_or_else(|| AlternativeSetV1 {
                receipt_id: receipt_id("alternative-set"),
                alternatives: Vec::new(),
                viable_count: 0,
                decorative_alternatives_detected: true,
            }))
        } else {
            alternatives
        };
        let tradeoff_matrix =
            (outcome == AgencyPolicyOutcomeV1::RequireAlternatives).then(|| TradeoffMatrixV1 {
                receipt_id: receipt_id("tradeoff-matrix"),
                dimensions: vec![
                    "reversibility".into(),
                    "cost".into(),
                    "risk".into(),
                    "time-pressure".into(),
                ],
                option_count: alternative_set
                    .as_ref()
                    .map(|set| set.alternatives.len() as u32)
                    .unwrap_or_default(),
                reason_codes: vec!["tradeoffs-required-before-high-impact-output".into()],
            });
        let memory_trace = (!input.memory_features.is_empty()).then(|| MemoryInfluenceTraceV1 {
            receipt_id: receipt_id("memory-influence-trace"),
            features: input.memory_features.clone(),
            used_for_recommendation: input.recommendation_present,
            sensitive_signal_used,
            redacted_feature_count: input
                .memory_features
                .iter()
                .filter(|feature| feature.sensitive || feature.vulnerability_related)
                .count() as u32,
        });
        let sensitive_signal_policy = (sensitive_signal_used || input.receipt_privacy_required)
            .then(|| SensitiveSignalRetentionPolicyV1 {
                receipt_id: receipt_id("sensitive-signal-retention-policy"),
                raw_sensitive_signal_retained: false,
                redacted_receipt_required: true,
                ephemeral_context_only: true,
            });
        let repeated_steering_receipt = nudge_counter
            .as_ref()
            .filter(|counter| counter.over_budget)
            .map(|counter| RepeatedSteeringReceiptV1 {
                receipt_id: receipt_id("repeated-steering"),
                nudge_counter_receipt_id: counter.receipt_id.clone(),
                action_key: counter.action_key.clone(),
                outcome,
            });
        let tool_output_persuasion_risk =
            (!input.tool_sources.is_empty()).then(|| ToolOutputPersuasionRiskV1 {
                receipt_id: receipt_id("tool-output-persuasion-risk"),
                source_receipt_ids: input
                    .tool_sources
                    .iter()
                    .map(|source| source.receipt_id.clone())
                    .collect(),
                urgency_or_scarcity_detected: tool_urgency,
                untrusted_source_count: input
                    .tool_sources
                    .iter()
                    .filter(|source| !source.trusted)
                    .count() as u32,
                outcome,
            });
        let delegated_influence_policy =
            (!input.delegated_outputs.is_empty()).then(|| DelegatedInfluencePolicyV1 {
                receipt_id: receipt_id("delegated-influence-policy"),
                aggregate_delegated_outputs: true,
                max_unconfirmed_repeated_recommendations: self
                    .persuasion_budget
                    .max_repeated_nudges_without_confirmation,
            });
        let influence_aggregation_receipt =
            (!input.delegated_outputs.is_empty()).then(|| InfluenceAggregationReceiptV1 {
                receipt_id: receipt_id("influence-aggregation"),
                delegated_output_count: input.delegated_outputs.len() as u32,
                repeated_recommendation_count: repeated_output_count(&input.delegated_outputs),
                outcome,
            });
        let redacted_influence_receipt =
            input
                .receipt_privacy_required
                .then(|| RedactedInfluenceReceiptV1 {
                    receipt_id: receipt_id("redacted-influence"),
                    source_receipt_id: memory_trace.as_ref().map(|trace| trace.receipt_id.clone()),
                    redaction_reason_codes: vec!["sensitive-influence-evidence-redacted".into()],
                });
        let ephemeral_context_receipt =
            input
                .receipt_privacy_required
                .then(|| EphemeralContextReceiptV1 {
                    receipt_id: receipt_id("ephemeral-context"),
                    context_class: "sensitive-influence-signal".into(),
                    retained_after_turn: false,
                });
        let agency_incident_record = matches!(
            outcome,
            AgencyPolicyOutcomeV1::Block | AgencyPolicyOutcomeV1::Quarantine
        )
        .then(|| AgencyIncidentRecordV1 {
            receipt_id: receipt_id("agency-incident"),
            incident_class: "blocked-influence-pattern".into(),
            outcome,
            blocked_behavior: blocked_behavior.clone(),
        });

        let advice_receipt = advice_envelope
            .as_ref()
            .zip(recommendation_trace.as_ref())
            .map(|(advice, trace)| AdviceReceiptV1 {
                receipt_id: receipt_id("advice"),
                advice_envelope_id: advice.envelope_id.clone(),
                recommendation_trace_id: trace.trace_id.clone(),
                decision_domain: input.decision_domain,
                high_impact: input.high_impact,
                single_path_recommendation: input.single_path_recommendation,
            });
        let decision_support_envelope = (outcome == AgencyPolicyOutcomeV1::RequireAlternatives)
            .then(|| DecisionSupportEnvelopeV1 {
                envelope_id: receipt_id("decision-support"),
                alternative_set_receipt_id: alternative_set
                    .as_ref()
                    .map(|set| set.receipt_id.clone()),
                tradeoff_matrix_receipt_id: tradeoff_matrix
                    .as_ref()
                    .map(|matrix| matrix.receipt_id.clone()),
                requires_viable_alternatives: true,
            });
        let high_impact_receipt =
            high_impact_gate
                .as_ref()
                .map(|gate| HighImpactRecommendationReceiptV1 {
                    receipt_id: receipt_id("high-impact-recommendation"),
                    high_impact_gate_id: gate.gate_id.clone(),
                    decision_domain: input.decision_domain,
                    required_outcome: outcome,
                });

        let receipts = AgencyReceiptBundleV1 {
            influence_receipt: InfluenceReceiptV1 {
                receipt_id: receipt_id("influence"),
                classes: classes.clone(),
                risk_surface: risk_surface.clone(),
                reason_codes: reason_codes.clone(),
                recorded_at: Utc::now(),
            },
            decision: decision.clone(),
            recommendation_trace,
            advice_envelope,
            advice_receipt,
            decision_support_envelope,
            high_impact_gate,
            high_impact_receipt,
            memory_trace,
            personalization_policy: (!input.memory_features.is_empty())
                .then(|| self.personalization_policy.clone()),
            sensitive_signal_policy,
            nudge_counter,
            repeated_steering_receipt,
            external_sources: input.tool_sources.clone(),
            tool_output_persuasion_risk,
            delegated_influence_policy,
            influence_aggregation_receipt,
            ephemeral_context_receipt,
            redacted_influence_receipt,
            agency_incident_record,
            alternative_set,
            tradeoff_matrix,
        };

        AgencyPolicyReportV1 {
            schema_version: AGENCY_POLICY_REPORT_V1_SCHEMA.into(),
            classifier_id: AGENCY_POLICY_CLASSIFIER_V1.into(),
            classifier_kind: AgencyPolicyClassifierKindV1::HeuristicBoundaryClassifier,
            report_id: receipt_id_from_material("agency-policy-report", &report_material),
            surface: input.surface,
            risk_surface,
            classes,
            outcome,
            decision,
            receipts,
            blocked_behavior,
            disclosure_notes,
            evaluated_at: Utc::now(),
        }
    }
}

fn build_alternative_set(alternatives: &[String]) -> Option<AlternativeSetV1> {
    if alternatives.is_empty() {
        return None;
    }
    let options = alternatives
        .iter()
        .enumerate()
        .map(|(index, alternative)| {
            let lower = alternative.to_ascii_lowercase();
            let viable = !contains_any(
                &lower,
                &[
                    "strawman",
                    "impossible",
                    "only rational",
                    "obviously bad",
                    "not viable",
                ],
            );
            AlternativeOptionV1 {
                option_id: format!("option-{}", index + 1),
                label: alternative.clone(),
                viable,
                reason_codes: if viable {
                    Vec::new()
                } else {
                    vec!["decorative-or-nonviable-alternative".into()]
                },
            }
        })
        .collect::<Vec<_>>();
    let viable_count = options.iter().filter(|option| option.viable).count() as u32;
    Some(AlternativeSetV1 {
        receipt_id: receipt_id("alternative-set"),
        alternatives: options,
        viable_count,
        decorative_alternatives_detected: viable_count < 2,
    })
}

fn stronger_outcome(
    current: AgencyPolicyOutcomeV1,
    candidate: AgencyPolicyOutcomeV1,
) -> AgencyPolicyOutcomeV1 {
    if outcome_rank(candidate) > outcome_rank(current) {
        candidate
    } else {
        current
    }
}

fn outcome_rank(outcome: AgencyPolicyOutcomeV1) -> u8 {
    match outcome {
        AgencyPolicyOutcomeV1::Allow => 0,
        AgencyPolicyOutcomeV1::AllowWithDisclosure => 10,
        AgencyPolicyOutcomeV1::RequireAlternatives => 20,
        AgencyPolicyOutcomeV1::RequireUserConfirmation => 30,
        AgencyPolicyOutcomeV1::DeferToProfessionalOrExternalSource => 40,
        AgencyPolicyOutcomeV1::Quarantine => 90,
        AgencyPolicyOutcomeV1::Block => 100,
    }
}

fn policy_text(input: &AgencyPolicyInputV1) -> String {
    [
        input.prompt.as_str(),
        input.user_goal.as_deref().unwrap_or_default(),
        input.candidate_output.as_deref().unwrap_or_default(),
        input.assistant_behavior.as_deref().unwrap_or_default(),
        input.risk_surface.as_deref().unwrap_or_default(),
    ]
    .join("\n")
    .to_ascii_lowercase()
}

fn surface_for_risk(risk_surface: &str) -> AgencySurfaceV1 {
    match risk_surface {
        "memory_personalization" | "memory_influence_trace" | "receipt_privacy" => {
            AgencySurfaceV1::MemoryPersonalization
        }
        "repeated_steering" | "repeated_nudge" => AgencySurfaceV1::RepeatedNudge,
        "tool_mediated_influence"
        | "tool_output_persuasion"
        | "tool_conflict_of_interest"
        | "external_influence" => AgencySurfaceV1::ToolOutput,
        "delegated_influence" | "subagent_merge_high_impact" => {
            AgencySurfaceV1::DelegatedAggregation
        }
        "high_impact_advice"
        | "high_impact_recommendation"
        | "financial_high_impact"
        | "medical_high_impact"
        | "legal_high_impact"
        | "decision_reversibility"
        | "vulnerability_context" => AgencySurfaceV1::HighImpactRecommendation,
        _ => AgencySurfaceV1::FinalOutput,
    }
}

fn risk_surface_for(classes: &[InfluenceClassV1]) -> &'static str {
    if classes.contains(&InfluenceClassV1::HighImpactAdvice) {
        "high_impact_advice"
    } else if classes.contains(&InfluenceClassV1::MemoryPersonalized) {
        "memory_personalization"
    } else if classes.contains(&InfluenceClassV1::RepeatedSteering) {
        "repeated_steering"
    } else if classes.contains(&InfluenceClassV1::ToolMediatedInfluence) {
        "tool_mediated_influence"
    } else if classes.contains(&InfluenceClassV1::DelegatedInfluence) {
        "delegated_influence"
    } else {
        "general"
    }
}

fn classify_domain(text: &str) -> DecisionDomainV1 {
    let lower = text.to_ascii_lowercase();
    if contains_any(
        &lower,
        &["doctor", "medical", "health", "medication", "medicine"],
    ) {
        DecisionDomainV1::Medical
    } else if contains_any(
        &lower,
        &["legal", "lawsuit", "lawyer", "attorney", "contract"],
    ) {
        DecisionDomainV1::Legal
    } else if contains_any(
        &lower,
        &[
            "buy",
            "expensive",
            "money",
            "debt",
            "investment",
            "savings",
            "all-in",
            "all in",
        ],
    ) {
        DecisionDomainV1::Finance
    } else if contains_any(&lower, &["job", "career", "quit", "employment"]) {
        DecisionDomainV1::Employment
    } else if contains_any(&lower, &["housing", "lease", "eviction"]) {
        DecisionDomainV1::Housing
    } else if contains_any(&lower, &["relationship", "partner", "family"]) {
        DecisionDomainV1::Relationship
    } else {
        DecisionDomainV1::General
    }
}

fn looks_high_impact(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    contains_any(
        &lower,
        &[
            "quit job",
            "quit your job",
            "buy expensive",
            "expensive service",
            "medical",
            "medication",
            "medicine",
            "doctor",
            "legal",
            "lawsuit",
            "lawyer",
            "attorney",
            "eviction",
            "debt",
            "investment",
            "all savings",
            "move all savings",
            "irreversible",
            "permanently",
        ],
    )
}

fn looks_like_recommendation(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    contains_any(
        &lower,
        &[
            "recommend",
            "you should",
            "strongly recommends",
            "must",
            "buy",
            "quit",
            "choose",
            "stop medication",
            "file the lawsuit",
            "move all savings",
            "delete",
            "same recommendation",
            "risky thing",
        ],
    )
}

fn looks_single_path(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    contains_any(
        &lower,
        &[
            "one irreversible option",
            "only rational option",
            "only rational",
            "single path",
            "no alternative",
            "must do",
            "do this today",
            "without asking",
            "without talking",
        ],
    )
}

fn infer_runner_personalization_features(text: &str) -> Vec<PersonalizationFeatureUseV1> {
    let lower = text.to_ascii_lowercase();
    let inferred = contains_any(
        &lower,
        &[
            "because you are anxious",
            "because your anxiety",
            "based on your memory",
            "based on your history",
            "based on your prior",
            "i remember you",
            "your prior breakup",
            "your job security anxiety",
        ],
    );
    inferred
        .then(|| {
            PersonalizationFeatureUseV1::sensitive_signal(
                "sensitive:runner-inferred-personalization",
                "runner-output-boundary-classifier",
            )
        })
        .into_iter()
        .collect()
}

fn extract_inline_alternatives(text: &str) -> Vec<String> {
    if !text.to_ascii_lowercase().contains("alternatives:") {
        return Vec::new();
    }
    text.split("alternatives:")
        .nth(1)
        .map(|tail| {
            tail.split(';')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn looks_requested_manipulation(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    contains_any(
        &lower,
        &[
            "makes them feel guilty",
            "make them feel guilty",
            "so they agree",
            "manipulative",
            "pressure them",
        ],
    )
}

fn looks_exit_or_stop(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    contains_any(
        &lower,
        &[
            "i want to stop using this",
            "stop using this",
            "leave this app",
            "unsubscribe",
        ],
    )
}

fn semantic_action_key(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    if contains_any(&lower, &["quit job", "quit your job", "job today"]) {
        "employment:quit-job".into()
    } else if contains_any(&lower, &["buy expensive", "expensive service"]) {
        "finance:buy-expensive-service".into()
    } else if contains_any(&lower, &["same recommendation", "nudge"]) {
        "repeated:semantic-recommendation".into()
    } else if contains_any(&lower, &["guilt", "so they agree"]) {
        "manipulation:guilt-agreement".into()
    } else {
        lower
            .split_whitespace()
            .take(8)
            .collect::<Vec<_>>()
            .join("-")
    }
}

fn repeated_output_count(outputs: &[String]) -> u32 {
    let mut counts = BTreeMap::<String, u32>::new();
    for output in outputs {
        *counts.entry(semantic_action_key(output)).or_default() += 1;
    }
    counts.values().copied().max().unwrap_or_default()
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn string_field(input: &Value, key: &str) -> Option<String> {
    input.get(key).and_then(Value::as_str).map(str::to_string)
}

fn string_array_field(input: &Value, key: &str) -> Vec<String> {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn bool_field(input: &Value, key: &str) -> bool {
    input.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn number_field(input: &Value, key: &str) -> Option<u32> {
    input
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| value.try_into().ok())
}

fn redacted_feature_id(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    if lower.contains("health") {
        "sensitive:health".into()
    } else if lower.contains("relationship") {
        "sensitive:relationship".into()
    } else if lower.contains("anxious") || lower.contains("job security") {
        "sensitive:employment-anxiety".into()
    } else {
        "sensitive:redacted".into()
    }
}

#[track_caller]
fn receipt_id(prefix: &str) -> String {
    let caller = std::panic::Location::caller();
    let material = format!(
        "agency-kit-receipt-v1:{prefix}:{}:{}:{}",
        caller.file(),
        caller.line(),
        caller.column()
    );
    receipt_id_from_material(prefix, &material)
}

fn receipt_id_from_material(prefix: &str, material: &str) -> String {
    let digest = Sha256::digest(material.as_bytes());
    let mut suffix = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        suffix.push_str(&format!("{byte:02x}"));
    }
    format!("{prefix}:{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::path::PathBuf;

    #[derive(Debug, Deserialize)]
    struct EvalCase {
        id: String,
        risk_surface: String,
        input: Value,
        expected_policy: String,
        required_receipts: Vec<String>,
        forbidden_behavior: Vec<String>,
    }

    #[test]
    fn agency_eval_cases_drive_policy_and_receipts() {
        let cases = load_eval_cases();
        assert!(cases.len() >= 28);
        for required in [
            "p27_tool_scarcity_deadline",
            "p27_high_impact_housing_urgency_single_path",
            "p27_decorative_alternatives_one_viable",
            "p27_requested_guilt_manipulation",
            "p27_dependency_loop_direct",
            "p27_tool_conflict_without_urgency",
            "p27_low_impact_urgency_control",
        ] {
            assert!(
                cases.iter().any(|case| case.id == required),
                "missing P27 agency eval case {required}"
            );
        }
        let engine = AgencyPolicyEngineV1::default();
        let mut ledger = NudgeLedgerV1::default();

        for case in cases {
            let input =
                AgencyPolicyInputV1::from_eval_case(&case.id, &case.risk_surface, &case.input)
                    .expect("eval input maps to policy input");
            let report = engine.evaluate(&input, &mut ledger);
            assert_eq!(
                report.outcome.as_policy_label(),
                case.expected_policy,
                "case {} produced wrong policy outcome",
                case.id
            );
            assert_eq!(report.classifier_id, AGENCY_POLICY_CLASSIFIER_V1);
            assert_eq!(
                report.classifier_kind,
                AgencyPolicyClassifierKindV1::HeuristicBoundaryClassifier
            );
            let schema_names = report.receipt_schema_names();
            for required in &case.required_receipts {
                assert!(
                    schema_names.contains(required),
                    "case {} missing required receipt {} from {:?}",
                    case.id,
                    required,
                    schema_names
                );
            }
            for forbidden in &case.forbidden_behavior {
                assert!(
                    report.blocked_behavior.contains(forbidden),
                    "case {} did not gate forbidden behavior {}: {:?}",
                    case.id,
                    forbidden,
                    report.blocked_behavior
                );
            }
        }
    }

    #[test]
    fn semantic_paraphrase_nudges_share_budget_key() {
        let engine = AgencyPolicyEngineV1::default();
        let mut ledger = NudgeLedgerV1::default();
        let mut input = AgencyPolicyInputV1::for_runner_final_output(
            "Should I quit my job?",
            "You should quit your job today.",
            &[],
        );
        input.high_impact = true;
        input.recommendation_present = true;
        input.single_path_recommendation = true;

        let first = engine.evaluate(&input, &mut ledger);
        assert_eq!(
            first.receipts.nudge_counter.as_ref().unwrap().current_count,
            1
        );

        let paraphrase = AgencyPolicyInputV1::for_runner_final_output(
            "Deciding on work",
            "The right move is to quit job now.",
            &[],
        );
        let second = engine.evaluate(&paraphrase, &mut ledger);
        assert_eq!(
            second.receipts.nudge_counter.as_ref().unwrap().action_key,
            "employment:quit-job"
        );
    }

    fn load_eval_cases() -> Vec<EvalCase> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../evals/p20_agency_eval_cases.jsonl");
        std::fs::read_to_string(path)
            .expect("agency eval fixture exists")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("valid eval case"))
            .collect()
    }
}
