//! Runtime view, widening, projection, and degradation disclosure DTOs.
//!
//! These records disclose local views without becoming a shadow memory/runtime truth system.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeViewModeV1 {
    Semantic,
    Temporal,
    Entity,
    Causal,
    Control,
    Execution,
    CombinedWithDisclosure,
}

impl fmt::Display for RuntimeViewModeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Semantic => "semantic",
            Self::Temporal => "temporal",
            Self::Entity => "entity",
            Self::Causal => "causal",
            Self::Control => "control",
            Self::Execution => "execution",
            Self::CombinedWithDisclosure => "combined-with-disclosure",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeTimeScopeModeV1 {
    Timeless,
    AsOf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeTimeScopeV1 {
    pub mode: RuntimeTimeScopeModeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<DateTime<Utc>>,
}

impl RuntimeTimeScopeV1 {
    pub fn timeless() -> Self {
        Self {
            mode: RuntimeTimeScopeModeV1::Timeless,
            valid_at: None,
            recorded_at: None,
        }
    }

    pub fn as_of(valid_at: DateTime<Utc>, recorded_at: DateTime<Utc>) -> Self {
        Self {
            mode: RuntimeTimeScopeModeV1::AsOf,
            valid_at: Some(valid_at),
            recorded_at: Some(recorded_at),
        }
    }

    pub fn is_time_scoped(&self) -> bool {
        self.mode == RuntimeTimeScopeModeV1::AsOf
            && self.valid_at.is_some()
            && self.recorded_at.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum IdentityExpansionPolicyV1 {
    ExactOnly,
    AliasExpansionAllowed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum QueryWideningKindV1 {
    AliasExpansion,
    TimeWindowExpansion,
    ScopeExpansion,
    FallbackIndex,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RetrievalPolicyV1 {
    pub policy_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub view_mode: RuntimeViewModeV1,
    pub time_scope: RuntimeTimeScopeV1,
    pub identity_expansion: IdentityExpansionPolicyV1,
    pub allow_query_widening: bool,
    pub allow_timeless_fallback: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub created_at: DateTime<Utc>,
}

impl RetrievalPolicyV1 {
    pub fn time_scoped(
        view_mode: RuntimeViewModeV1,
        valid_at: DateTime<Utc>,
        recorded_at: DateTime<Utc>,
    ) -> Self {
        Self {
            policy_id: display_only_unstable_id("retrieval-policy"),
            kind: ArtifactKindV1::RetrievalPolicy,
            view_mode,
            time_scope: RuntimeTimeScopeV1::as_of(valid_at, recorded_at),
            identity_expansion: IdentityExpansionPolicyV1::ExactOnly,
            allow_query_widening: false,
            allow_timeless_fallback: false,
            scopes: Vec::new(),
            reason_codes: vec!["time-scoped-query-no-silent-timeless-fallback".into()],
            canonical_backpointers: canonical_owner_backpointer(
                "knowledge-runtime",
                "QueryTrace",
                "canonical-runtime-query-owner",
            ),
            created_at: Utc::now(),
        }
    }

    pub fn timeless(view_mode: RuntimeViewModeV1) -> Self {
        Self {
            policy_id: display_only_unstable_id("retrieval-policy"),
            kind: ArtifactKindV1::RetrievalPolicy,
            view_mode,
            time_scope: RuntimeTimeScopeV1::timeless(),
            identity_expansion: IdentityExpansionPolicyV1::ExactOnly,
            allow_query_widening: false,
            allow_timeless_fallback: false,
            scopes: Vec::new(),
            reason_codes: vec!["timeless-query-explicit".into()],
            canonical_backpointers: canonical_owner_backpointer(
                "knowledge-runtime",
                "QueryTrace",
                "canonical-runtime-query-owner",
            ),
            created_at: Utc::now(),
        }
    }

    pub fn with_alias_expansion(mut self) -> Self {
        self.identity_expansion = IdentityExpansionPolicyV1::AliasExpansionAllowed;
        self.allow_query_widening = true;
        self.reason_codes
            .push("alias-expansion-requires-receipt".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn with_timeless_fallback(mut self) -> Self {
        self.allow_timeless_fallback = true;
        self.reason_codes
            .push("timeless-fallback-requires-degradation-event".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self.scopes.sort();
        self.scopes.dedup();
        self
    }

    pub fn forbids_silent_timeless_fallback(&self) -> bool {
        self.time_scope.is_time_scoped()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeViewRequestV1 {
    pub request_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub view_mode: RuntimeViewModeV1,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predicate: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    pub retrieval_policy: RetrievalPolicyV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub requested_at: DateTime<Utc>,
}

impl RuntimeViewRequestV1 {
    pub fn new(query: impl Into<String>, retrieval_policy: RetrievalPolicyV1) -> Self {
        let view_mode = retrieval_policy.view_mode.clone();
        Self {
            request_id: display_only_unstable_id("runtime-view-request"),
            kind: ArtifactKindV1::RuntimeViewRequest,
            view_mode,
            query: query.into(),
            subject: None,
            predicate: None,
            aliases: Vec::new(),
            retrieval_policy,
            reason_codes: vec!["runtime-view-requested".into()],
            canonical_backpointers: canonical_owner_backpointer(
                "knowledge-runtime",
                "QueryTrace",
                "canonical-runtime-view-owner",
            ),
            requested_at: Utc::now(),
        }
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn predicate(mut self, predicate: impl Into<String>) -> Self {
        self.predicate = Some(predicate.into());
        self
    }

    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self.aliases.sort();
        self.aliases.dedup();
        self
    }

    pub fn is_time_scoped(&self) -> bool {
        self.retrieval_policy.time_scope.is_time_scoped()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct QueryWideningReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub request_id: ArtifactId,
    pub policy_id: ArtifactId,
    pub widening_kind: QueryWideningKindV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub original_terms: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub widened_terms: Vec<String>,
    pub allowed_by_policy: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub recorded_at: DateTime<Utc>,
}

impl QueryWideningReportV1 {
    pub fn alias_expansion(
        request: &RuntimeViewRequestV1,
        original_terms: Vec<String>,
        widened_terms: Vec<String>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("query-widening"),
            kind: ArtifactKindV1::QueryWidening,
            request_id: request.request_id.clone(),
            policy_id: request.retrieval_policy.policy_id.clone(),
            widening_kind: QueryWideningKindV1::AliasExpansion,
            original_terms,
            widened_terms,
            allowed_by_policy: request.retrieval_policy.allow_query_widening
                && request.retrieval_policy.identity_expansion
                    == IdentityExpansionPolicyV1::AliasExpansionAllowed,
            reason_codes: vec!["alias-expansion-disclosed".into()],
            canonical_backpointers: canonical_owner_backpointer(
                "knowledge-runtime",
                "WideningDisclosure",
                "canonical-widening-disclosure-owner",
            ),
            recorded_at: Utc::now(),
        }
    }

    pub fn is_alias_expansion(&self) -> bool {
        self.widening_kind == QueryWideningKindV1::AliasExpansion
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DegradationEventV1 {
    pub event_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub request_id: ArtifactId,
    pub view_mode: RuntimeViewModeV1,
    pub degraded: bool,
    pub fallback_attempted: bool,
    pub fallback_allowed: bool,
    pub reason_code: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_claim_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_degradation_record_id: Option<StackDegradationRecordId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub recorded_at: DateTime<Utc>,
}

impl DegradationEventV1 {
    pub fn new(
        request: &RuntimeViewRequestV1,
        reason_code: impl Into<String>,
        fallback_attempted: bool,
        fallback_allowed: bool,
        affected_claim_ids: Vec<ArtifactId>,
    ) -> Self {
        Self {
            event_id: display_only_unstable_id("degradation-event"),
            kind: ArtifactKindV1::DegradationEvent,
            request_id: request.request_id.clone(),
            view_mode: request.view_mode.clone(),
            degraded: true,
            fallback_attempted,
            fallback_allowed,
            reason_code: reason_code.into(),
            affected_claim_ids,
            canonical_degradation_record_id: None,
            canonical_backpointers: canonical_owner_backpointer(
                "knowledge-runtime",
                "WideningDisclosure",
                "canonical-degradation-disclosure-owner",
            ),
            recorded_at: Utc::now(),
        }
    }

    pub fn proves_no_silent_timeless_fallback(&self) -> bool {
        !self.fallback_attempted || !self.fallback_allowed || self.degraded
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProjectionDigestV1 {
    pub projection_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub view_mode: RuntimeViewModeV1,
    pub policy_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_episode_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_claim_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_evidence_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_projection_id: Option<StackProjectionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_import_batch_id: Option<StackImportBatchId>,
    pub digest: DisplayDigestV1,
    pub rebuilt_from_authoritative_memory: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub created_at: DateTime<Utc>,
}

impl ProjectionDigestV1 {
    pub fn new(
        view_mode: RuntimeViewModeV1,
        policy_id: ArtifactId,
        source_episode_ids: Vec<ArtifactId>,
        source_claim_ids: Vec<ArtifactId>,
        source_evidence_ids: Vec<ArtifactId>,
        digest_payload: serde_json::Value,
    ) -> Self {
        Self {
            projection_id: display_only_unstable_id("projection-digest"),
            kind: ArtifactKindV1::ProjectionDigest,
            view_mode,
            policy_id,
            source_episode_ids,
            source_claim_ids,
            source_evidence_ids,
            canonical_projection_id: None,
            canonical_import_batch_id: None,
            digest: DisplayDigestV1::for_json_value(&digest_payload),
            rebuilt_from_authoritative_memory: true,
            reason_codes: vec!["projection-rebuild-from-authoritative-memory".into()],
            canonical_backpointers: vec![
                CanonicalBackpointerV1::owner_type(
                    "semantic-memory",
                    "ImportReceipt",
                    "canonical-memory-import-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "forge-memory-bridge",
                    "ProjectionImportBatchV3",
                    "canonical-projection-transform-owner",
                ),
            ],
            created_at: Utc::now(),
        }
    }

    pub fn equivalent_digest(&self, other: &Self) -> bool {
        self.digest == other.digest && self.policy_id == other.policy_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ViewDisclosureReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub request_id: ArtifactId,
    pub view_mode: RuntimeViewModeV1,
    pub policy_id: ArtifactId,
    pub projection_digest: ProjectionDigestV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_claim_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query_widening_receipt_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degradation_event_ids: Vec<ArtifactId>,
    pub authoritative_source: String,
    pub separates_execution_from_domain_truth: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub recorded_at: DateTime<Utc>,
}

impl ViewDisclosureReportV1 {
    pub fn new(
        request: &RuntimeViewRequestV1,
        projection_digest: ProjectionDigestV1,
        matched_claim_ids: Vec<ArtifactId>,
        query_widening_receipt_ids: Vec<ArtifactId>,
        degradation_event_ids: Vec<ArtifactId>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("view-disclosure"),
            kind: ArtifactKindV1::ViewDisclosure,
            request_id: request.request_id.clone(),
            view_mode: request.view_mode.clone(),
            policy_id: request.retrieval_policy.policy_id.clone(),
            projection_digest,
            matched_claim_ids,
            query_widening_receipt_ids,
            degradation_event_ids,
            authoritative_source: "append-only-memory-and-receipts".into(),
            separates_execution_from_domain_truth: true,
            reason_codes: vec!["view-disclosure-emitted-before-results".into()],
            canonical_backpointers: canonical_owner_backpointer(
                "knowledge-runtime",
                "QueryTrace",
                "canonical-runtime-disclosure-owner",
            ),
            recorded_at: Utc::now(),
        }
    }

    pub fn discloses_policy_events(&self) -> bool {
        self.separates_execution_from_domain_truth
            && self
                .query_widening_receipt_ids
                .iter()
                .all(|id| !id.0.is_empty())
            && self.degradation_event_ids.iter().all(|id| !id.0.is_empty())
    }

    pub fn has_visible_widening_or_degradation(&self) -> bool {
        !self.query_widening_receipt_ids.is_empty() || !self.degradation_event_ids.is_empty()
    }
}
