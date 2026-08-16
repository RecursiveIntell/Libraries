//! Versioned recursive provenance for repeated context compaction.
//!
//! V1 remains the single-generation wire format. V2 adds one immutable parent
//! edge, deterministic lineage identities, transitive original-source refs,
//! and append-only persistence without moving lifecycle authority into a host
//! adapter or derived index.

use crate::{
    compact_context, context_expand, finalize_compacted_response, hash_text, hash_text_sha256,
    receipt_index, verified_exact_fallback_item, verify_exact_fallback_integrity,
    verify_response_integrity, CheckpointStrategy, CompactRequest, CompactResponse,
    ContextAllocationPlanV1, ContextCompactionReceiptV1, ContextExpandResult, ContextGovernorError,
    ContextStepV1, ExactRecoveryStateV1, ExactStoredItemV1, FileContextStore,
    FileContextStoreSaveResultV1, Message, PlanStateV1, RecoveryDurabilityV1, StructuralFloorV1,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const V1_SCHEMA: &str = "ContextCompactionReceiptV1";
const V2_SCHEMA: &str = "ContextCompactionReceiptV2";

/// Immutable reference to the one receipt whose compacted projection formed
/// the exact prefix of a child compaction input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParentReceiptRefV2 {
    pub receipt_schema: String,
    pub receipt_id: String,
    pub generation: u32,
    pub receipt_identity_blake3: String,
    pub receipt_identity_sha256: String,
    #[serde(default)]
    pub lineage_blake3: String,
    #[serde(default)]
    pub lineage_sha256: String,
    pub compacted_transcript_blake3: String,
    pub compacted_transcript_sha256: String,
}

/// Transitive identity for one exact original input message. The bytes live
/// only in the originating V2 receipt, or in a verified V1 exact-store item
/// when an explicit legacy bridge was requested.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct OriginalSourceRefV2 {
    pub source_id: String,
    pub origin_receipt_schema: String,
    pub origin_receipt_id: String,
    pub origin_generation: u32,
    pub origin_message_index: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub content_blake3: String,
    pub content_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_sha256: Option<String>,
}

/// Exact source bytes introduced at this generation. Descendants retain only
/// the corresponding `OriginalSourceRefV2` and traverse back here to expand.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceEvidenceItemV2 {
    pub schema: String,
    pub source_id: String,
    pub origin_receipt_id: String,
    pub origin_generation: u32,
    pub origin_message_index: usize,
    pub message: Message,
    pub content_blake3: String,
    pub content_sha256: String,
    pub message_blake3: String,
    pub message_sha256: String,
}

/// V2 keeps the proven V1 local-compaction fields while adding recursive
/// provenance. V1 itself is not extended or reinterpreted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextCompactionReceiptV2 {
    pub schema: String,
    pub receipt_id: String,
    pub session_id: String,
    pub parent_session_id: Option<String>,
    pub created_utc: DateTime<Utc>,
    pub engine: String,
    pub engine_version: String,
    pub original_message_count: usize,
    pub compacted_message_count: usize,
    pub original_approx_tokens: usize,
    pub compacted_approx_tokens: usize,
    pub token_savings_estimate: isize,
    pub token_counter: crate::TokenCounterKind,
    pub original_transcript_blake3: String,
    pub compacted_transcript_blake3: String,
    pub original_transcript_sha256: String,
    pub compacted_transcript_sha256: String,
    pub allocation_plan_id: String,
    pub semantic_memory_fact_ids: Vec<String>,
    pub semantic_memory_document_ids: Vec<String>,
    pub exact_fallback_refs: Vec<crate::ExactFallbackRefV1>,
    pub summary_loss_report: crate::SummaryLossReportV1,
    pub warnings: Vec<String>,
    pub recovery_durability: RecoveryDurabilityV1,
    pub generation: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_receipt: Option<ParentReceiptRefV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_receipt_id: Option<String>,
    pub covered_original_sources: Vec<OriginalSourceRefV2>,
    pub local_source_ids: Vec<String>,
    pub lineage_blake3: String,
    pub lineage_sha256: String,
    pub receipt_identity_blake3: String,
    pub receipt_identity_sha256: String,
    /// Full SHA-256 key identity. The secret key is never serialized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing_key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactResponseV2 {
    pub receipt: ContextCompactionReceiptV2,
    pub allocation_plan: ContextAllocationPlanV1,
    pub compacted_messages: Vec<Message>,
    pub exact_store: Vec<ExactStoredItemV1>,
    #[serde(default)]
    pub context_steps: Vec<ContextStepV1>,
    #[serde(default)]
    pub plan_state: PlanStateV1,
    #[serde(default)]
    pub structural_floor: StructuralFloorV1,
    pub source_evidence: Vec<SourceEvidenceItemV2>,
    /// Detached HMAC over only immutable provenance and exact-source evidence.
    /// This remains verifiable when the rebuildable compacted projection is
    /// damaged, allowing authenticated exact recovery without trusting it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_hmac: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hmac: Option<String>,
}

/// Read-time union for one authoritative receipt store. The discriminant is
/// the receipt's explicit schema, never a permissive best-effort parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionedCompactResponse {
    V1(Box<CompactResponse>),
    V2(Box<CompactResponseV2>),
}

/// Authenticated, durable but deliberately non-authoritative receipt staged
/// while the host commits its corresponding transcript projection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingReceiptInfoV2 {
    pub schema: String,
    pub receipt_id: String,
    pub session_id: String,
    pub generation: u32,
    pub created_utc: DateTime<Utc>,
    pub pending_path: PathBuf,
    pub expected_compacted_message_count: usize,
    pub expected_compacted_transcript_blake3: String,
    pub expected_compacted_transcript_sha256: String,
    /// The authenticated expected projection lets a recovering host narrowly
    /// rehydrate fields its durable conversation store does not round-trip.
    pub expected_compacted_messages: Vec<Message>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReceiptActivationRequestV2 {
    pub receipt_id: String,
    pub committed_messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReceiptActivationResultV2 {
    pub schema: String,
    pub receipt_id: String,
    pub path: PathBuf,
    pub activated: bool,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReceiptDiscardResultV2 {
    pub schema: String,
    pub receipt_id: String,
    pub discarded: bool,
}

impl Serialize for VersionedCompactResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::V1(response) => response.serialize(serializer),
            Self::V2(response) => response.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for VersionedCompactResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let schema = value
            .get("receipt")
            .and_then(|receipt| receipt.get("schema"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| serde::de::Error::custom("receipt.schema is required"))?;
        match schema {
            V1_SCHEMA => serde_json::from_value(value)
                .map(Box::new)
                .map(Self::V1)
                .map_err(serde::de::Error::custom),
            V2_SCHEMA => serde_json::from_value(value)
                .map(Box::new)
                .map(Self::V2)
                .map_err(serde::de::Error::custom),
            other => Err(serde::de::Error::custom(format!(
                "unsupported context compaction receipt schema: {other}"
            ))),
        }
    }
}

impl VersionedCompactResponse {
    pub fn receipt_id(&self) -> &str {
        match self {
            Self::V1(response) => &response.receipt.receipt_id,
            Self::V2(response) => &response.receipt.receipt_id,
        }
    }

    pub fn session_id(&self) -> &str {
        match self {
            Self::V1(response) => &response.receipt.session_id,
            Self::V2(response) => &response.receipt.session_id,
        }
    }

    pub fn compacted_messages(&self) -> &[Message] {
        match self {
            Self::V1(response) => &response.compacted_messages,
            Self::V2(response) => &response.compacted_messages,
        }
    }

    pub fn created_utc(&self) -> DateTime<Utc> {
        match self {
            Self::V1(response) => response.receipt.created_utc,
            Self::V2(response) => response.receipt.created_utc,
        }
    }

    pub fn as_v1_projection(&self) -> CompactResponse {
        match self {
            Self::V1(response) => response.as_ref().clone(),
            Self::V2(response) => response.as_v1_projection(),
        }
    }
}

impl CompactResponseV2 {
    /// Compatibility projection for V1 compaction helpers and rebuildable
    /// search indexes. It is never persisted in place of the V2 receipt.
    pub fn as_v1_projection(&self) -> CompactResponse {
        CompactResponse {
            receipt: self.receipt.as_v1_projection(),
            allocation_plan: self.allocation_plan.clone(),
            compacted_messages: self.compacted_messages.clone(),
            exact_store: self.exact_store.clone(),
            context_steps: self.context_steps.clone(),
            plan_state: self.plan_state.clone(),
            structural_floor: self.structural_floor.clone(),
            hmac: None,
        }
    }
}

impl ContextCompactionReceiptV2 {
    fn from_v1(receipt: &ContextCompactionReceiptV1) -> Self {
        Self {
            schema: V2_SCHEMA.to_string(),
            receipt_id: receipt.receipt_id.clone(),
            session_id: receipt.session_id.clone(),
            parent_session_id: receipt.parent_session_id.clone(),
            created_utc: receipt.created_utc,
            engine: receipt.engine.clone(),
            engine_version: receipt.engine_version.clone(),
            original_message_count: receipt.original_message_count,
            compacted_message_count: receipt.compacted_message_count,
            original_approx_tokens: receipt.original_approx_tokens,
            compacted_approx_tokens: receipt.compacted_approx_tokens,
            token_savings_estimate: receipt.token_savings_estimate,
            token_counter: receipt.token_counter.clone(),
            original_transcript_blake3: receipt.original_transcript_blake3.clone(),
            compacted_transcript_blake3: receipt.compacted_transcript_blake3.clone(),
            original_transcript_sha256: receipt.original_transcript_sha256.clone(),
            compacted_transcript_sha256: receipt.compacted_transcript_sha256.clone(),
            allocation_plan_id: receipt.allocation_plan_id.clone(),
            semantic_memory_fact_ids: receipt.semantic_memory_fact_ids.clone(),
            semantic_memory_document_ids: receipt.semantic_memory_document_ids.clone(),
            exact_fallback_refs: receipt.exact_fallback_refs.clone(),
            summary_loss_report: receipt.summary_loss_report.clone(),
            warnings: receipt.warnings.clone(),
            recovery_durability: receipt.recovery_durability.clone(),
            generation: 1,
            parent_receipt: None,
            supersedes_receipt_id: None,
            covered_original_sources: Vec::new(),
            local_source_ids: Vec::new(),
            lineage_blake3: String::new(),
            lineage_sha256: String::new(),
            receipt_identity_blake3: String::new(),
            receipt_identity_sha256: String::new(),
            signing_key_id: None,
        }
    }

    fn as_v1_projection(&self) -> ContextCompactionReceiptV1 {
        ContextCompactionReceiptV1 {
            schema: V1_SCHEMA.to_string(),
            receipt_id: self.receipt_id.clone(),
            session_id: self.session_id.clone(),
            parent_session_id: self.parent_session_id.clone(),
            created_utc: self.created_utc,
            engine: self.engine.clone(),
            engine_version: self.engine_version.clone(),
            original_message_count: self.original_message_count,
            compacted_message_count: self.compacted_message_count,
            original_approx_tokens: self.original_approx_tokens,
            compacted_approx_tokens: self.compacted_approx_tokens,
            token_savings_estimate: self.token_savings_estimate,
            token_counter: self.token_counter.clone(),
            original_transcript_blake3: self.original_transcript_blake3.clone(),
            compacted_transcript_blake3: self.compacted_transcript_blake3.clone(),
            original_transcript_sha256: self.original_transcript_sha256.clone(),
            compacted_transcript_sha256: self.compacted_transcript_sha256.clone(),
            allocation_plan_id: self.allocation_plan_id.clone(),
            semantic_memory_fact_ids: self.semantic_memory_fact_ids.clone(),
            semantic_memory_document_ids: self.semantic_memory_document_ids.clone(),
            exact_fallback_refs: self.exact_fallback_refs.clone(),
            summary_loss_report: self.summary_loss_report.clone(),
            warnings: self.warnings.clone(),
            recovery_durability: self.recovery_durability.clone(),
        }
    }

    fn update_local_fields(&mut self, receipt: &ContextCompactionReceiptV1) {
        self.receipt_id = receipt.receipt_id.clone();
        self.session_id = receipt.session_id.clone();
        self.parent_session_id = receipt.parent_session_id.clone();
        self.created_utc = receipt.created_utc;
        self.engine = receipt.engine.clone();
        self.engine_version = receipt.engine_version.clone();
        self.original_message_count = receipt.original_message_count;
        self.compacted_message_count = receipt.compacted_message_count;
        self.original_approx_tokens = receipt.original_approx_tokens;
        self.compacted_approx_tokens = receipt.compacted_approx_tokens;
        self.token_savings_estimate = receipt.token_savings_estimate;
        self.token_counter = receipt.token_counter.clone();
        self.original_transcript_blake3 = receipt.original_transcript_blake3.clone();
        self.compacted_transcript_blake3 = receipt.compacted_transcript_blake3.clone();
        self.original_transcript_sha256 = receipt.original_transcript_sha256.clone();
        self.compacted_transcript_sha256 = receipt.compacted_transcript_sha256.clone();
        self.allocation_plan_id = receipt.allocation_plan_id.clone();
        self.semantic_memory_fact_ids = receipt.semantic_memory_fact_ids.clone();
        self.semantic_memory_document_ids = receipt.semantic_memory_document_ids.clone();
        self.exact_fallback_refs = receipt.exact_fallback_refs.clone();
        self.summary_loss_report = receipt.summary_loss_report.clone();
        self.warnings = receipt.warnings.clone();
        self.recovery_durability = receipt.recovery_durability.clone();
    }
}

#[derive(Serialize)]
struct SourceIdMaterial<'a> {
    schema: &'static str,
    session_id: &'a str,
    generation: u32,
    origin_message_index: usize,
    role: &'a str,
    content_blake3: &'a str,
    content_sha256: &'a str,
    message_blake3: &'a str,
    message_sha256: &'a str,
}

#[derive(Serialize)]
struct LineageSourceIdentityMaterial<'a> {
    source_id: &'a str,
    origin_receipt_schema: &'a str,
    origin_generation: u32,
    origin_message_index: usize,
    origin_item_id: &'a Option<String>,
    role: &'a Option<String>,
    content_blake3: &'a str,
    content_sha256: &'a str,
    message_blake3: &'a Option<String>,
    message_sha256: &'a Option<String>,
}

#[derive(Serialize)]
struct LineageIdentityMaterial<'a> {
    schema: &'static str,
    session_id: &'a str,
    generation: u32,
    parent_schema: Option<&'a str>,
    parent_generation: Option<u32>,
    parent_receipt_identity_blake3: Option<&'a str>,
    parent_receipt_identity_sha256: Option<&'a str>,
    parent_lineage_blake3: Option<&'a str>,
    parent_lineage_sha256: Option<&'a str>,
    original_transcript_blake3: &'a str,
    original_transcript_sha256: &'a str,
    sources: Vec<LineageSourceIdentityMaterial<'a>>,
}

fn canonical_hashes<T: Serialize>(value: &T) -> Result<(String, String), ContextGovernorError> {
    let bytes = serde_json::to_vec(value)?;
    Ok((
        blake3::hash(&bytes).to_hex().to_string(),
        format!("{:x}", Sha256::digest(&bytes)),
    ))
}

fn message_hashes(message: &Message) -> Result<(String, String), ContextGovernorError> {
    canonical_hashes(message)
}

fn is_compaction_projection(message: &Message) -> bool {
    message
        .metadata
        .get("compressed_summary")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
        || (message.name.as_deref() == Some("context_governor")
            && message
                .id
                .as_deref()
                .is_some_and(|id| id.starts_with("summary_")))
}

struct ComputedSourceIdentity {
    source_id: String,
    content_blake3: String,
    content_sha256: String,
    message_blake3: String,
    message_sha256: String,
}

fn compute_source_identity(
    session_id: &str,
    generation: u32,
    origin_message_index: usize,
    message: &Message,
) -> Result<ComputedSourceIdentity, ContextGovernorError> {
    let content_blake3 = hash_text(&message.content);
    let content_sha256 = hash_text_sha256(&message.content);
    let (message_blake3, message_sha256) = message_hashes(message)?;
    let material = SourceIdMaterial {
        schema: "OriginalSourceIdentityV2",
        session_id,
        generation,
        origin_message_index,
        role: &message.role,
        content_blake3: &content_blake3,
        content_sha256: &content_sha256,
        message_blake3: &message_blake3,
        message_sha256: &message_sha256,
    };
    let (source_digest, _) = canonical_hashes(&material)?;
    Ok(ComputedSourceIdentity {
        source_id: format!("ctxs_{source_digest}"),
        content_blake3,
        content_sha256,
        message_blake3,
        message_sha256,
    })
}

fn build_local_sources(
    session_id: &str,
    receipt_id: &str,
    generation: u32,
    messages: &[Message],
    start_index: usize,
) -> Result<Vec<SourceEvidenceItemV2>, ContextGovernorError> {
    let mut sources = Vec::new();
    for (offset, message) in messages.iter().enumerate().skip(start_index) {
        if is_compaction_projection(message) {
            continue;
        }
        let identity = compute_source_identity(session_id, generation, offset, message)?;
        sources.push(SourceEvidenceItemV2 {
            schema: "SourceEvidenceItemV2".to_string(),
            source_id: identity.source_id,
            origin_receipt_id: receipt_id.to_string(),
            origin_generation: generation,
            origin_message_index: offset,
            message: message.clone(),
            content_blake3: identity.content_blake3,
            content_sha256: identity.content_sha256,
            message_blake3: identity.message_blake3,
            message_sha256: identity.message_sha256,
        });
    }
    sources.sort_by(|left, right| left.source_id.cmp(&right.source_id));
    Ok(sources)
}

fn source_ref(source: &SourceEvidenceItemV2) -> OriginalSourceRefV2 {
    OriginalSourceRefV2 {
        source_id: source.source_id.clone(),
        origin_receipt_schema: V2_SCHEMA.to_string(),
        origin_receipt_id: source.origin_receipt_id.clone(),
        origin_generation: source.origin_generation,
        origin_message_index: source.origin_message_index,
        origin_item_id: None,
        role: Some(source.message.role.clone()),
        content_blake3: source.content_blake3.clone(),
        content_sha256: source.content_sha256.clone(),
        message_blake3: Some(source.message_blake3.clone()),
        message_sha256: Some(source.message_sha256.clone()),
    }
}

fn legacy_source_refs(
    response: &CompactResponse,
) -> Result<Vec<OriginalSourceRefV2>, ContextGovernorError> {
    verify_exact_fallback_integrity(response)?;
    let mut sources = Vec::new();
    for exact in &response.exact_store {
        let fallback = response
            .receipt
            .exact_fallback_refs
            .iter()
            .find(|candidate| candidate.item_id == exact.item_id)
            .ok_or_else(|| ContextGovernorError::ExactFallbackIntegrityMismatch {
                item_id: exact.item_id.clone(),
                reason: "missing legacy fallback reference".to_string(),
            })?;
        let material = (
            "LegacyOriginalSourceIdentityV2",
            response.receipt.receipt_id.as_str(),
            exact.item_id.as_str(),
            fallback.start_index,
            exact.content_blake3.as_str(),
            fallback.content_sha256.as_str(),
        );
        let (digest, _) = canonical_hashes(&material)?;
        sources.push(OriginalSourceRefV2 {
            source_id: format!("ctxs_v1_{digest}"),
            origin_receipt_schema: V1_SCHEMA.to_string(),
            origin_receipt_id: response.receipt.receipt_id.clone(),
            origin_generation: 1,
            origin_message_index: fallback.start_index,
            origin_item_id: Some(exact.item_id.clone()),
            role: None,
            content_blake3: exact.content_blake3.clone(),
            content_sha256: fallback.content_sha256.clone(),
            message_blake3: None,
            message_sha256: None,
        });
    }
    sources.sort();
    Ok(sources)
}

fn v1_receipt_identity(
    response: &CompactResponse,
) -> Result<(String, String), ContextGovernorError> {
    // Projection bytes are deliberately excluded: exact recovery may proceed
    // when a summary is corrupt, but the receipt and exact evidence may not.
    canonical_hashes(&(&response.receipt, &response.exact_store))
}

fn parent_reference(
    parent: &VersionedCompactResponse,
) -> Result<ParentReceiptRefV2, ContextGovernorError> {
    match parent {
        VersionedCompactResponse::V1(response) => {
            let (blake3, sha256) = v1_receipt_identity(response)?;
            Ok(ParentReceiptRefV2 {
                receipt_schema: V1_SCHEMA.to_string(),
                receipt_id: response.receipt.receipt_id.clone(),
                generation: 1,
                receipt_identity_blake3: blake3,
                receipt_identity_sha256: sha256,
                lineage_blake3: String::new(),
                lineage_sha256: String::new(),
                compacted_transcript_blake3: response.receipt.compacted_transcript_blake3.clone(),
                compacted_transcript_sha256: response.receipt.compacted_transcript_sha256.clone(),
            })
        }
        VersionedCompactResponse::V2(response) => Ok(ParentReceiptRefV2 {
            receipt_schema: V2_SCHEMA.to_string(),
            receipt_id: response.receipt.receipt_id.clone(),
            generation: response.receipt.generation,
            receipt_identity_blake3: response.receipt.receipt_identity_blake3.clone(),
            receipt_identity_sha256: response.receipt.receipt_identity_sha256.clone(),
            lineage_blake3: response.receipt.lineage_blake3.clone(),
            lineage_sha256: response.receipt.lineage_sha256.clone(),
            compacted_transcript_blake3: response.receipt.compacted_transcript_blake3.clone(),
            compacted_transcript_sha256: response.receipt.compacted_transcript_sha256.clone(),
        }),
    }
}

fn parent_sources(
    parent: &VersionedCompactResponse,
) -> Result<Vec<OriginalSourceRefV2>, ContextGovernorError> {
    match parent {
        VersionedCompactResponse::V1(response) => legacy_source_refs(response),
        VersionedCompactResponse::V2(response) => {
            Ok(response.receipt.covered_original_sources.clone())
        }
    }
}

fn refresh_v2_integrity(response: &mut CompactResponseV2) -> Result<(), ContextGovernorError> {
    response
        .source_evidence
        .sort_by(|left, right| left.source_id.cmp(&right.source_id));
    response.receipt.local_source_ids = response
        .source_evidence
        .iter()
        .map(|source| source.source_id.clone())
        .collect();
    response.receipt.covered_original_sources.sort();

    let parent = response.receipt.parent_receipt.as_ref();
    let sources = response
        .receipt
        .covered_original_sources
        .iter()
        .map(|source| LineageSourceIdentityMaterial {
            source_id: &source.source_id,
            origin_receipt_schema: &source.origin_receipt_schema,
            origin_generation: source.origin_generation,
            origin_message_index: source.origin_message_index,
            origin_item_id: &source.origin_item_id,
            role: &source.role,
            content_blake3: &source.content_blake3,
            content_sha256: &source.content_sha256,
            message_blake3: &source.message_blake3,
            message_sha256: &source.message_sha256,
        })
        .collect();
    let lineage_material = LineageIdentityMaterial {
        schema: "ContextCompactionLineageIdentityV2",
        session_id: &response.receipt.session_id,
        generation: response.receipt.generation,
        parent_schema: parent.map(|value| value.receipt_schema.as_str()),
        parent_generation: parent.map(|value| value.generation),
        parent_receipt_identity_blake3: parent.map(|value| value.receipt_identity_blake3.as_str()),
        parent_receipt_identity_sha256: parent.map(|value| value.receipt_identity_sha256.as_str()),
        parent_lineage_blake3: parent.map(|value| value.lineage_blake3.as_str()),
        parent_lineage_sha256: parent.map(|value| value.lineage_sha256.as_str()),
        original_transcript_blake3: &response.receipt.original_transcript_blake3,
        original_transcript_sha256: &response.receipt.original_transcript_sha256,
        sources,
    };
    let (lineage_blake3, lineage_sha256) = canonical_hashes(&lineage_material)?;
    response.receipt.lineage_blake3 = lineage_blake3;
    response.receipt.lineage_sha256 = lineage_sha256;

    response.receipt.receipt_identity_blake3.clear();
    response.receipt.receipt_identity_sha256.clear();
    let (receipt_blake3, receipt_sha256) = canonical_hashes(&response.receipt)?;
    response.receipt.receipt_identity_blake3 = receipt_blake3;
    response.receipt.receipt_identity_sha256 = receipt_sha256;
    Ok(())
}

fn lineage_error(receipt_id: &str, reason: impl Into<String>) -> ContextGovernorError {
    ContextGovernorError::LineageIntegrityMismatch {
        receipt_id: receipt_id.to_string(),
        reason: reason.into(),
    }
}

fn verify_v2_provenance_integrity(
    response: &CompactResponseV2,
    require_projection_identity: bool,
) -> Result<(), ContextGovernorError> {
    let receipt_id = &response.receipt.receipt_id;
    if response.receipt.schema != V2_SCHEMA {
        return Err(ContextGovernorError::UnsupportedReceiptSchema(
            response.receipt.schema.clone(),
        ));
    }
    if response.receipt.generation == 0 {
        return Err(lineage_error(receipt_id, "generation must be at least one"));
    }
    match (
        &response.receipt.parent_receipt,
        response.receipt.generation,
    ) {
        (None, 1) => {
            if response.receipt.supersedes_receipt_id.is_some() {
                return Err(lineage_error(receipt_id, "root cannot supersede a receipt"));
            }
        }
        (Some(parent), generation) if generation > 1 => {
            if parent.receipt_id == *receipt_id {
                return Err(lineage_error(receipt_id, "receipt cannot parent itself"));
            }
            if generation != parent.generation.saturating_add(1) {
                return Err(lineage_error(
                    receipt_id,
                    "generation is not parent generation plus one",
                ));
            }
            if response.receipt.supersedes_receipt_id.as_deref() != Some(parent.receipt_id.as_str())
            {
                return Err(lineage_error(
                    receipt_id,
                    "supersedes receipt must equal the parent receipt",
                ));
            }
        }
        _ => {
            return Err(lineage_error(
                receipt_id,
                "root/parent fields disagree with generation",
            ));
        }
    }

    let mut sorted_sources = response.receipt.covered_original_sources.clone();
    sorted_sources.sort();
    if sorted_sources != response.receipt.covered_original_sources
        || sorted_sources
            .windows(2)
            .any(|window| window[0].source_id == window[1].source_id)
    {
        return Err(lineage_error(
            receipt_id,
            "covered original sources are not sorted and unique",
        ));
    }
    let mut sorted_evidence = response.source_evidence.clone();
    sorted_evidence.sort_by(|left, right| left.source_id.cmp(&right.source_id));
    if sorted_evidence != response.source_evidence
        || sorted_evidence
            .windows(2)
            .any(|window| window[0].source_id == window[1].source_id)
    {
        return Err(lineage_error(
            receipt_id,
            "local source evidence is not sorted and unique",
        ));
    }
    let local_ids = response
        .source_evidence
        .iter()
        .map(|source| source.source_id.clone())
        .collect::<Vec<_>>();
    if local_ids != response.receipt.local_source_ids {
        return Err(lineage_error(
            receipt_id,
            "local source IDs do not match local source evidence",
        ));
    }

    for source in &response.source_evidence {
        if source.schema != "SourceEvidenceItemV2"
            || source.origin_receipt_id != *receipt_id
            || source.origin_generation != response.receipt.generation
        {
            return Err(lineage_error(
                receipt_id,
                format!("source {} has invalid origin", source.source_id),
            ));
        }
        let expected_identity = compute_source_identity(
            &response.receipt.session_id,
            response.receipt.generation,
            source.origin_message_index,
            &source.message,
        )?;
        if source.content_blake3 != expected_identity.content_blake3
            || source.content_sha256 != expected_identity.content_sha256
            || source.message_blake3 != expected_identity.message_blake3
            || source.message_sha256 != expected_identity.message_sha256
        {
            return Err(lineage_error(
                receipt_id,
                format!("source {} hash mismatch", source.source_id),
            ));
        }
        if source.source_id != expected_identity.source_id {
            return Err(lineage_error(
                receipt_id,
                format!("source {} has a non-canonical identity", source.source_id),
            ));
        }
        let expected_ref = source_ref(source);
        if !response
            .receipt
            .covered_original_sources
            .iter()
            .any(|candidate| candidate == &expected_ref)
        {
            return Err(lineage_error(
                receipt_id,
                format!(
                    "source {} is missing its transitive reference",
                    source.source_id
                ),
            ));
        }
    }

    let local_refs = response
        .source_evidence
        .iter()
        .map(source_ref)
        .collect::<BTreeSet<_>>();
    let current_generation_refs = response
        .receipt
        .covered_original_sources
        .iter()
        .filter(|source| source.origin_generation == response.receipt.generation)
        .cloned()
        .collect::<BTreeSet<_>>();
    if current_generation_refs != local_refs {
        return Err(lineage_error(
            receipt_id,
            "current-generation refs differ from local source evidence",
        ));
    }
    if response
        .receipt
        .covered_original_sources
        .iter()
        .any(|source| {
            source.origin_generation > response.receipt.generation
                || (source.origin_generation == response.receipt.generation
                    && (source.origin_receipt_schema != V2_SCHEMA
                        || source.origin_receipt_id != *receipt_id))
        })
    {
        return Err(lineage_error(
            receipt_id,
            "source reference has an impossible generation or origin",
        ));
    }

    let mut expected = response.clone();
    expected.receipt.receipt_identity_blake3.clear();
    expected.receipt.receipt_identity_sha256.clear();
    refresh_v2_integrity(&mut expected)?;
    if expected.receipt.lineage_blake3 != response.receipt.lineage_blake3
        || expected.receipt.lineage_sha256 != response.receipt.lineage_sha256
    {
        return Err(lineage_error(receipt_id, "lineage identity mismatch"));
    }
    if require_projection_identity
        && (expected.receipt.receipt_identity_blake3 != response.receipt.receipt_identity_blake3
            || expected.receipt.receipt_identity_sha256 != response.receipt.receipt_identity_sha256)
    {
        return Err(lineage_error(receipt_id, "receipt identity mismatch"));
    }
    Ok(())
}

fn verify_v2_full_integrity(response: &CompactResponseV2) -> Result<(), ContextGovernorError> {
    verify_v2_provenance_integrity(response, true)?;
    verify_response_integrity(&response.as_v1_projection())
}

fn verify_versioned(
    response: &VersionedCompactResponse,
    full_projection: bool,
) -> Result<(), ContextGovernorError> {
    match response {
        VersionedCompactResponse::V1(response) => {
            if response.receipt.schema != V1_SCHEMA {
                return Err(ContextGovernorError::UnsupportedReceiptSchema(
                    response.receipt.schema.clone(),
                ));
            }
            if full_projection {
                verify_response_integrity(response)
            } else {
                verify_exact_fallback_integrity(response)
            }
        }
        VersionedCompactResponse::V2(response) => {
            if full_projection {
                verify_v2_full_integrity(response)
            } else {
                verify_v2_provenance_integrity(response, false)
            }
        }
    }
}

/// Canonical authenticated evidence excludes the rebuildable projection and
/// every receipt field derived solely from it. Exact source bytes, their
/// hashes/references, lineage edges, session/generation identity, durability,
/// and signer identity remain covered. Parent projection identities are also
/// excluded; the parent's own evidence signature authenticates its sources.
fn v2_evidence_authentication_value(
    response: &CompactResponseV2,
) -> Result<serde_json::Value, ContextGovernorError> {
    let mut value = serde_json::to_value(response)?;
    let object = value.as_object_mut().ok_or_else(|| {
        ContextGovernorError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "V2 receipt must serialize as an object",
        ))
    })?;
    for field in [
        "allocation_plan",
        "compacted_messages",
        "context_steps",
        "plan_state",
        "structural_floor",
        "evidence_hmac",
        "hmac",
    ] {
        object.remove(field);
    }
    let receipt = object
        .get_mut("receipt")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            ContextGovernorError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "V2 receipt authentication material has no receipt object",
            ))
        })?;
    for field in [
        "allocation_plan_id",
        "compacted_message_count",
        "compacted_approx_tokens",
        "token_savings_estimate",
        "compacted_transcript_blake3",
        "compacted_transcript_sha256",
        "receipt_identity_blake3",
        "receipt_identity_sha256",
        "summary_loss_report",
        "warnings",
    ] {
        receipt.remove(field);
    }
    if let Some(parent) = receipt
        .get_mut("parent_receipt")
        .and_then(serde_json::Value::as_object_mut)
    {
        for field in [
            "receipt_identity_blake3",
            "receipt_identity_sha256",
            "compacted_transcript_blake3",
            "compacted_transcript_sha256",
        ] {
            parent.remove(field);
        }
    }
    Ok(value)
}

fn strict_v2_signature_key_id<'a>(
    response: &'a CompactResponseV2,
    signature: &'a str,
    operation: &str,
) -> Result<&'a str, ContextGovernorError> {
    let receipt_id = response.receipt.receipt_id.clone();
    let Some((signature_key_id, _)) = signature.split_once(':') else {
        return Err(ContextGovernorError::ReceiptIntegrityFailed {
            receipt_id,
            operation: operation.to_string(),
        });
    };
    let declared = response.receipt.signing_key_id.as_deref().ok_or_else(|| {
        ContextGovernorError::ReceiptIntegrityMissing {
            receipt_id: response.receipt.receipt_id.clone(),
            operation: operation.to_string(),
        }
    })?;
    let current_full_id = signature_key_id.len() == 64
        && signature_key_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !current_full_id || signature_key_id != declared {
        return Err(ContextGovernorError::ReceiptIntegrityFailed {
            receipt_id: response.receipt.receipt_id.clone(),
            operation: operation.to_string(),
        });
    }
    Ok(signature_key_id)
}

fn verify_v2_authentication(
    response: &CompactResponseV2,
    ring: &receipt_index::KeyRing,
    full_projection: bool,
    operation: &str,
) -> Result<(), ContextGovernorError> {
    let receipt_id = response.receipt.receipt_id.clone();
    if full_projection || response.evidence_hmac.is_none() {
        let signature = response.hmac.as_deref().ok_or_else(|| {
            ContextGovernorError::ReceiptIntegrityMissing {
                receipt_id: receipt_id.clone(),
                operation: operation.to_string(),
            }
        })?;
        strict_v2_signature_key_id(response, signature, operation)?;
        let value = serde_json::to_value(response)?;
        if !ring.verify_json(&value, "hmac") {
            return Err(ContextGovernorError::ReceiptIntegrityFailed {
                receipt_id,
                operation: operation.to_string(),
            });
        }
    }
    if let Some(signature) = response.evidence_hmac.as_deref() {
        strict_v2_signature_key_id(response, signature, operation)?;
        let material = v2_evidence_authentication_value(response)?;
        let payload = serde_json::to_string(&material)?;
        if !ring.sign_and_verify(&payload, signature) {
            return Err(ContextGovernorError::ReceiptIntegrityFailed {
                receipt_id: response.receipt.receipt_id.clone(),
                operation: operation.to_string(),
            });
        }
    }
    Ok(())
}

fn sign_v2_response(
    response: &mut CompactResponseV2,
    ring: &receipt_index::KeyRing,
) -> Result<(), ContextGovernorError> {
    response.hmac = None;
    response.evidence_hmac = None;
    response.receipt.signing_key_id = Some(ring.active_key_id()?);
    refresh_v2_integrity(response)?;
    let evidence = v2_evidence_authentication_value(response)?;
    let evidence_payload = serde_json::to_string(&evidence)?;
    let active_key_id = ring.active_key_id()?;
    response.evidence_hmac = Some(format!(
        "{active_key_id}:{}",
        receipt_index::sign_receipt_content(&evidence_payload, &ring.active)
    ));
    let value = serde_json::to_value(&*response)?;
    response.hmac = Some(ring.sign_json(&value, "hmac")?);
    Ok(())
}

fn compact_context_v2_with_parent(
    mut request: CompactRequest,
    parent: Option<&VersionedCompactResponse>,
    governed_authority: Option<&receipt_index::KeyRing>,
) -> Result<CompactResponseV2, ContextGovernorError> {
    if request.messages.is_empty() {
        return Err(ContextGovernorError::EmptyMessages);
    }
    let original_messages = request.messages.clone();
    let lineage_policy = request.policy.clone();
    let (generation, parent_ref, inherited_sources, new_source_start) = match parent {
        None => (1, None, Vec::new(), 0),
        Some(parent) => {
            verify_versioned(parent, true)?;
            if parent.session_id() != request.session_id {
                return Err(lineage_error(
                    parent.receipt_id(),
                    "parent and child session IDs differ",
                ));
            }
            let prefix = parent.compacted_messages();
            if original_messages.len() < prefix.len()
                || original_messages[..prefix.len()] != *prefix
            {
                return Err(lineage_error(
                    parent.receipt_id(),
                    "parent compacted transcript is not the exact child-input prefix",
                ));
            }
            let reference = parent_reference(parent)?;
            let generation = next_generation(&reference.receipt_id, reference.generation)?;
            (
                generation,
                Some(reference),
                parent_sources(parent)?,
                prefix.len(),
            )
        }
    };

    // A V2 request is caller data.  It cannot choose a key even when this
    // internal compatibility struct was constructed by a legacy embedding.
    request.hmac_key_path = None;
    let v1 = compact_context(request)?;
    let mut receipt = ContextCompactionReceiptV2::from_v1(&v1.receipt);
    receipt.generation = generation;
    receipt.parent_receipt = parent_ref.clone();
    receipt.supersedes_receipt_id = parent_ref.as_ref().map(|parent| parent.receipt_id.clone());
    let source_evidence = build_local_sources(
        &receipt.session_id,
        &receipt.receipt_id,
        generation,
        &original_messages,
        new_source_start,
    )?;
    let mut covered = inherited_sources;
    covered.extend(source_evidence.iter().map(source_ref));
    covered.sort();
    if covered
        .windows(2)
        .any(|window| window[0].source_id == window[1].source_id)
    {
        return Err(lineage_error(
            &receipt.receipt_id,
            "source identity collision while constructing lineage",
        ));
    }
    receipt.covered_original_sources = covered;

    if let Some(maximum_generation) = lineage_policy.max_lineage_generation {
        if generation > maximum_generation {
            return Err(ContextGovernorError::LineageGenerationLimit {
                generation,
                maximum_generation,
            });
        }
    }
    if let Some(maximum_bytes) = lineage_policy.max_provenance_bytes {
        let actual_bytes = serde_json::to_vec(&receipt.covered_original_sources)?.len();
        if actual_bytes > maximum_bytes {
            return Err(ContextGovernorError::ProvenanceBudgetExceeded {
                actual_bytes,
                maximum_bytes,
            });
        }
    }

    let mut response = CompactResponseV2 {
        receipt,
        allocation_plan: v1.allocation_plan,
        compacted_messages: v1.compacted_messages,
        exact_store: v1.exact_store,
        context_steps: v1.context_steps,
        plan_state: v1.plan_state,
        structural_floor: v1.structural_floor,
        source_evidence,
        evidence_hmac: None,
        hmac: None,
    };
    if let Some(minimum_savings) = lineage_policy.min_net_savings_tokens {
        let before = response.receipt.original_approx_tokens;
        let after = response.receipt.compacted_approx_tokens;
        let host_checkpoint_needed = after > lineage_policy.target_tokens
            && !matches!(lineage_policy.checkpoint.strategy, CheckpointStrategy::Off)
            && lineage_policy.checkpoint.max_checkpoints_per_session != Some(0);
        if before.saturating_sub(after) < minimum_savings && !host_checkpoint_needed {
            return Err(ContextGovernorError::CompactionNoNetBenefit {
                before,
                after,
                minimum_savings,
            });
        }
    }
    refresh_v2_integrity(&mut response)?;
    if let Some(ring) = governed_authority {
        sign_v2_response(&mut response, ring)?;
    }
    verify_v2_full_integrity(&response)?;
    Ok(response)
}

fn next_generation(receipt_id: &str, parent_generation: u32) -> Result<u32, ContextGovernorError> {
    parent_generation
        .checked_add(1)
        .ok_or_else(|| ContextGovernorError::GenerationOverflow {
            receipt_id: receipt_id.to_string(),
        })
}

/// Construct a fresh generation-1 V2 receipt. Store-aware continuation uses
/// `FileContextStore::compact_next_v2` so restart recovery can prove ancestry.
pub fn compact_context_v2(
    request: CompactRequest,
) -> Result<CompactResponseV2, ContextGovernorError> {
    compact_context_v2_with_parent(request, None, None)
}

/// Rebind a V2 local projection after deterministic sanitation or an audited
/// LLM checkpoint. Provenance must validate before any projection mutation.
pub fn finalize_compacted_response_v2(
    response: CompactResponseV2,
    compacted_messages: Vec<Message>,
    governed_authority: &receipt_index::KeyRing,
) -> Result<CompactResponseV2, ContextGovernorError> {
    // Authenticate the exact compact-v2 candidate before applying any host
    // projection. Structural hashes alone are attacker-recomputable and must
    // never authorize a new source/provenance object at this boundary.
    verify_v2_authentication(
        &response,
        governed_authority,
        true,
        "V2 projection finalization",
    )?;
    verify_v2_full_integrity(&response)?;
    let mut output = response;
    let finalized = finalize_compacted_response(output.as_v1_projection(), compacted_messages)?;
    output.receipt.update_local_fields(&finalized.receipt);
    output.allocation_plan = finalized.allocation_plan;
    output.compacted_messages = finalized.compacted_messages;
    output.exact_store = finalized.exact_store;
    output.context_steps = finalized.context_steps;
    output.plan_state = finalized.plan_state;
    output.structural_floor = finalized.structural_floor;
    output.hmac = None;
    output.evidence_hmac = None;
    refresh_v2_integrity(&mut output)?;
    sign_v2_response(&mut output, governed_authority)?;
    verify_v2_full_integrity(&output)?;
    verify_v2_authentication(
        &output,
        governed_authority,
        true,
        "V2 projection finalization",
    )?;
    Ok(output)
}

impl FileContextStore {
    /// The sole persisted-receipt admission boundary. Callers must use this
    /// before receipt data can select a parent, contribute lineage, satisfy an
    /// exact expansion, or protect retention. Structural hashes alone detect
    /// corruption; the configured key ring authenticates the durable evidence.
    pub(crate) fn verify_versioned_for_use(
        &self,
        response: &VersionedCompactResponse,
        full_projection: bool,
        operation: &str,
    ) -> Result<(), ContextGovernorError> {
        match (response, &self.integrity_key_ring) {
            (VersionedCompactResponse::V2(_), None) => {
                return Err(ContextGovernorError::ReceiptIntegrityUnavailable {
                    operation: operation.to_string(),
                    reason: "V2 authority requires governed key descriptors".to_string(),
                });
            }
            (VersionedCompactResponse::V2(response), Some(ring)) => {
                verify_v2_authentication(response, ring, full_projection, operation)?;
            }
            (VersionedCompactResponse::V1(response), Some(ring)) => {
                let receipt_id = response.receipt.receipt_id.clone();
                let value = serde_json::to_value(response.as_ref())?;
                if value
                    .get("hmac")
                    .and_then(serde_json::Value::as_str)
                    .is_none()
                {
                    return Err(ContextGovernorError::ReceiptIntegrityMissing {
                        receipt_id,
                        operation: operation.to_string(),
                    });
                }
                if !ring.verify_json(&value, "hmac") {
                    return Err(ContextGovernorError::ReceiptIntegrityFailed {
                        receipt_id,
                        operation: operation.to_string(),
                    });
                }
            }
            (VersionedCompactResponse::V1(_), None) => {}
        }
        verify_versioned(response, full_projection)?;
        Ok(())
    }

    fn read_versioned_unverified(
        &self,
        receipt_id: &str,
    ) -> Result<VersionedCompactResponse, ContextGovernorError> {
        let path = self.path_for_receipt(receipt_id)?;
        if !path.exists() {
            return Err(ContextGovernorError::ReceiptNotFound(
                receipt_id.to_string(),
            ));
        }
        let response: VersionedCompactResponse = serde_json::from_slice(&fs::read(path)?)?;
        if response.receipt_id() != receipt_id {
            return Err(lineage_error(
                receipt_id,
                format!(
                    "path identity disagrees with payload {}",
                    response.receipt_id()
                ),
            ));
        }
        Ok(response)
    }

    fn collect_lineage(
        &self,
        receipt_id: &str,
        full_projection: bool,
    ) -> Result<Vec<VersionedCompactResponse>, ContextGovernorError> {
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut current_id = receipt_id.to_string();
        loop {
            if !seen.insert(current_id.clone()) {
                return Err(lineage_error(receipt_id, "cycle in parent receipt graph"));
            }
            let current = match self.read_versioned_unverified(&current_id) {
                Ok(response) => response,
                Err(ContextGovernorError::ReceiptNotFound(_)) if !chain.is_empty() => {
                    return Err(ContextGovernorError::LineageMissingAncestor {
                        receipt_id: receipt_id.to_string(),
                        ancestor_id: current_id,
                    });
                }
                Err(error) => return Err(error),
            };
            self.verify_versioned_for_use(&current, full_projection, "lineage traversal")?;
            let parent_ref = match &current {
                VersionedCompactResponse::V1(_) => None,
                VersionedCompactResponse::V2(response) => response.receipt.parent_receipt.clone(),
            };
            chain.push(current);
            let Some(parent_ref) = parent_ref else {
                break;
            };
            current_id = parent_ref.receipt_id;
        }

        for index in 0..chain.len().saturating_sub(1) {
            let child = match &chain[index] {
                VersionedCompactResponse::V2(response) => response,
                VersionedCompactResponse::V1(_) => {
                    return Err(lineage_error(
                        receipt_id,
                        "V1 receipt cannot contain a parent edge",
                    ));
                }
            };
            let parent = &chain[index + 1];
            let expected_parent = parent_reference(parent)?;
            let parent_matches = child.receipt.parent_receipt.as_ref().is_some_and(|actual| {
                if full_projection {
                    actual == &expected_parent
                } else {
                    actual.receipt_schema == expected_parent.receipt_schema
                        && actual.receipt_id == expected_parent.receipt_id
                        && actual.generation == expected_parent.generation
                        && actual.lineage_blake3 == expected_parent.lineage_blake3
                        && actual.lineage_sha256 == expected_parent.lineage_sha256
                }
            });
            if !parent_matches {
                return Err(lineage_error(
                    &child.receipt.receipt_id,
                    "parent provenance identity mismatch",
                ));
            }
            if child.receipt.session_id != parent.session_id() {
                return Err(lineage_error(
                    &child.receipt.receipt_id,
                    "parent and child session IDs differ",
                ));
            }
            let mut expected_sources = parent_sources(parent)?;
            expected_sources.extend(child.source_evidence.iter().map(source_ref));
            expected_sources.sort();
            if expected_sources != child.receipt.covered_original_sources {
                return Err(lineage_error(
                    &child.receipt.receipt_id,
                    "transitive source manifest differs from parent plus local evidence",
                ));
            }
        }

        if let Some(VersionedCompactResponse::V2(root)) = chain.last() {
            if root.receipt.parent_receipt.is_none() {
                let mut local = root
                    .source_evidence
                    .iter()
                    .map(source_ref)
                    .collect::<Vec<_>>();
                local.sort();
                if local != root.receipt.covered_original_sources {
                    return Err(lineage_error(
                        &root.receipt.receipt_id,
                        "root source manifest differs from local evidence",
                    ));
                }
            }
        }
        Ok(chain)
    }

    /// Load V1 or V2 without silently downgrading a V2 schema to V1 fields.
    pub fn load_versioned(
        &self,
        receipt_id: &str,
    ) -> Result<VersionedCompactResponse, ContextGovernorError> {
        let response = self.read_versioned_unverified(receipt_id)?;
        match response {
            VersionedCompactResponse::V1(response) => {
                let response = VersionedCompactResponse::V1(response);
                self.verify_versioned_for_use(&response, true, "receipt load")?;
                Ok(response)
            }
            VersionedCompactResponse::V2(_) => self
                .collect_lineage(receipt_id, true)?
                .into_iter()
                .next()
                .ok_or_else(|| ContextGovernorError::ReceiptNotFound(receipt_id.to_string())),
        }
    }

    pub fn load_v2(&self, receipt_id: &str) -> Result<CompactResponseV2, ContextGovernorError> {
        match self.load_versioned(receipt_id)? {
            VersionedCompactResponse::V2(response) => Ok(*response),
            VersionedCompactResponse::V1(_) => Err(ContextGovernorError::UnsupportedReceiptSchema(
                V1_SCHEMA.to_string(),
            )),
        }
    }

    fn receipt_paths(&self) -> Result<Vec<PathBuf>, ContextGovernorError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut paths = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some(".receipt-index.json") {
                continue;
            }
            paths.push(path);
        }
        paths.sort();
        Ok(paths)
    }

    pub(crate) fn require_authority_if_v2_present(
        &self,
        operation: &str,
    ) -> Result<(), ContextGovernorError> {
        if self.integrity_key_ring.is_some() {
            return Ok(());
        }
        for path in self.receipt_paths()? {
            let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
            if value
                .get("receipt")
                .and_then(|receipt| receipt.get("schema"))
                .and_then(serde_json::Value::as_str)
                == Some(V2_SCHEMA)
            {
                return Err(ContextGovernorError::ReceiptIntegrityUnavailable {
                    operation: operation.to_string(),
                    reason: "V2 authority requires governed key descriptors".to_string(),
                });
            }
        }
        Ok(())
    }

    fn v2_receipts_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<CompactResponseV2>, ContextGovernorError> {
        let mut responses = Vec::new();
        for path in self.receipt_paths()? {
            let value: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
            let schema = value
                .get("receipt")
                .and_then(|receipt| receipt.get("schema"))
                .and_then(serde_json::Value::as_str);
            if schema != Some(V2_SCHEMA) {
                continue;
            }
            let response: CompactResponseV2 = serde_json::from_value(value)?;
            if response.receipt.session_id == session_id {
                self.collect_lineage(&response.receipt.receipt_id, true)?;
                responses.push(response);
            }
        }
        responses.sort_by(|left, right| left.receipt.receipt_id.cmp(&right.receipt.receipt_id));
        Ok(responses)
    }

    /// Resolve the single unsuperseded V2 tip. V1 receipts are deliberately
    /// ignored unless a caller names one explicitly.
    pub fn resolve_lineage_tip(
        &self,
        session_id: &str,
    ) -> Result<Option<String>, ContextGovernorError> {
        let responses = self.v2_receipts_for_session(session_id)?;
        let parent_ids = responses
            .iter()
            .filter_map(|response| response.receipt.parent_receipt.as_ref())
            .map(|parent| parent.receipt_id.as_str())
            .collect::<BTreeSet<_>>();
        let tips = responses
            .iter()
            .filter(|response| !parent_ids.contains(response.receipt.receipt_id.as_str()))
            .map(|response| response.receipt.receipt_id.clone())
            .collect::<Vec<_>>();
        match tips.as_slice() {
            [] => Ok(None),
            [tip] => Ok(Some(tip.clone())),
            _ => Err(ContextGovernorError::AmbiguousLineageTip {
                session_id: session_id.to_string(),
                receipt_ids: tips,
            }),
        }
    }

    /// Construct the next receipt using the canonical store for restart-safe
    /// parent selection. An explicit parent may select a verified V1 bridge.
    pub fn compact_next_v2(
        &self,
        request: CompactRequest,
        explicit_parent_receipt_id: Option<&str>,
    ) -> Result<CompactResponseV2, ContextGovernorError> {
        let governed_authority = self.integrity_key_ring.as_ref().ok_or_else(|| {
            ContextGovernorError::ReceiptIntegrityUnavailable {
                operation: "V2 compaction".to_string(),
                reason: "governed key descriptors are required".to_string(),
            }
        })?;
        let parent_id = match explicit_parent_receipt_id {
            Some(receipt_id) => Some(receipt_id.to_string()),
            None => self.resolve_lineage_tip(&request.session_id)?,
        };
        let parent = match parent_id {
            Some(receipt_id) => {
                let response = self.load_versioned(&receipt_id)?;
                Some(response)
            }
            None => None,
        };
        compact_context_v2_with_parent(request, parent.as_ref(), Some(governed_authority))
    }

    pub fn save_v2(
        &self,
        response: &CompactResponseV2,
    ) -> Result<FileContextStoreSaveResultV1, ContextGovernorError> {
        let pending = self.prepare_v2(response)?;
        let activated = self.activate_v2(ReceiptActivationRequestV2 {
            receipt_id: pending.receipt_id.clone(),
            committed_messages: pending.expected_compacted_messages,
        })?;
        Ok(FileContextStoreSaveResultV1 {
            schema: "FileContextStoreSaveResultV1".to_string(),
            receipt_id: activated.receipt_id,
            path: activated.path,
            exact_recovery_state: if response.receipt.covered_original_sources.is_empty()
                && response.exact_store.is_empty()
            {
                ExactRecoveryStateV1::Unavailable
            } else {
                ExactRecoveryStateV1::Persisted
            },
            verified: activated.verified,
        })
    }

    /// Compatibility helper for an in-process certified caller. It validates
    /// that the supplied key is exactly the store's governed active key, then
    /// performs prepare+activate. Hosts with a separate transcript commit must
    /// call `prepare_v2` and `activate_v2` around that commit instead.
    pub fn save_v2_with_hmac_key(
        &self,
        response: &CompactResponseV2,
        hmac_key: &[u8],
    ) -> Result<FileContextStoreSaveResultV1, ContextGovernorError> {
        let ring = self.require_v2_authority("V2 receipt save")?;
        let supplied_key_id = receipt_index::key_id(hmac_key)?;
        let governed_key_id = ring.active_key_id()?;
        if supplied_key_id != governed_key_id {
            return Err(ContextGovernorError::WrongConfiguredKeyId {
                expected: governed_key_id,
                actual: supplied_key_id,
            });
        }
        self.save_v2(response)
    }

    fn require_v2_authority(
        &self,
        operation: &str,
    ) -> Result<&receipt_index::KeyRing, ContextGovernorError> {
        self.integrity_key_ring.as_ref().ok_or_else(|| {
            ContextGovernorError::ReceiptIntegrityUnavailable {
                operation: operation.to_string(),
                reason: "governed key descriptors are required".to_string(),
            }
        })
    }

    fn validate_v2_append_position(
        &self,
        response: &CompactResponseV2,
    ) -> Result<(), ContextGovernorError> {
        let active_tip = self.resolve_lineage_tip(&response.receipt.session_id)?;
        match &response.receipt.parent_receipt {
            None => {
                if let Some(tip) = active_tip {
                    return Err(lineage_error(
                        &response.receipt.receipt_id,
                        format!("cannot append a second root while tip {tip} exists"),
                    ));
                }
            }
            Some(parent_ref) => {
                let parent = self
                    .load_versioned(&parent_ref.receipt_id)
                    .map_err(|error| {
                        if matches!(error, ContextGovernorError::ReceiptNotFound(_)) {
                            ContextGovernorError::LineageMissingAncestor {
                                receipt_id: response.receipt.receipt_id.clone(),
                                ancestor_id: parent_ref.receipt_id.clone(),
                            }
                        } else {
                            error
                        }
                    })?;
                if parent_reference(&parent)? != *parent_ref {
                    return Err(lineage_error(
                        &response.receipt.receipt_id,
                        "parent changed after child construction",
                    ));
                }
                if let Some(tip) = active_tip {
                    if tip != parent_ref.receipt_id {
                        return Err(lineage_error(
                            &response.receipt.receipt_id,
                            format!("parent was already superseded by active tip {tip}"),
                        ));
                    }
                } else if parent_ref.receipt_schema == V2_SCHEMA {
                    return Err(lineage_error(
                        &response.receipt.receipt_id,
                        "V2 parent is not the active session tip",
                    ));
                }
            }
        }
        Ok(())
    }

    fn pending_root(&self) -> PathBuf {
        self.root.join(".pending")
    }

    fn pending_path_for_receipt(&self, receipt_id: &str) -> Result<PathBuf, ContextGovernorError> {
        self.path_for_receipt(receipt_id)?;
        Ok(self.pending_root().join(format!("{receipt_id}.json")))
    }

    fn persisted_v2_candidate(
        &self,
        response: &CompactResponseV2,
        ring: &receipt_index::KeyRing,
    ) -> Result<CompactResponseV2, ContextGovernorError> {
        let mut persisted = response.clone();
        let exact_recoverable = !persisted.receipt.covered_original_sources.is_empty()
            || !persisted.exact_store.is_empty();
        persisted.receipt.summary_loss_report.exact_recovery_state = if exact_recoverable {
            ExactRecoveryStateV1::Persisted
        } else {
            ExactRecoveryStateV1::Unavailable
        };
        persisted.receipt.recovery_durability = if exact_recoverable {
            RecoveryDurabilityV1::Persisted
        } else {
            RecoveryDurabilityV1::Unavailable
        };
        persisted.hmac = None;
        persisted.evidence_hmac = None;
        refresh_v2_integrity(&mut persisted)?;
        sign_v2_response(&mut persisted, ring)?;
        verify_v2_full_integrity(&persisted)?;
        verify_v2_authentication(&persisted, ring, true, "V2 receipt preparation")?;
        Ok(persisted)
    }

    /// Stage a fully authenticated receipt without publishing it as a lineage
    /// tip. Every signer/ring/structural/ancestry check completes before the
    /// atomic pending-file rename.
    pub fn prepare_v2(
        &self,
        response: &CompactResponseV2,
    ) -> Result<PendingReceiptInfoV2, ContextGovernorError> {
        let ring = self.require_v2_authority("V2 receipt preparation")?;
        verify_v2_authentication(response, ring, true, "V2 receipt preparation")?;
        verify_v2_full_integrity(response)?;

        let persisted = self.persisted_v2_candidate(response, ring)?;
        let bytes = serde_json::to_vec_pretty(&persisted)?;
        // Prove that the exact bytes about to be written decode and verify;
        // no signer or ring admission error is allowed after publication.
        let decoded: CompactResponseV2 = serde_json::from_slice(&bytes)?;
        verify_v2_full_integrity(&decoded)?;
        verify_v2_authentication(&decoded, ring, true, "V2 receipt preparation")?;

        fs::create_dir_all(&self.root)?;
        fs::create_dir_all(self.pending_root())?;
        let _lock = self.lock_store()?;
        let path = self.path_for_receipt(&response.receipt.receipt_id)?;
        let pending_path = self.pending_path_for_receipt(&response.receipt.receipt_id)?;
        if path.exists() || pending_path.exists() {
            return Err(ContextGovernorError::ReceiptAlreadyExists(
                response.receipt.receipt_id.clone(),
            ));
        }
        self.validate_v2_append_position(&persisted)?;

        let temporary_path = self.pending_root().join(format!(
            ".{}.{}.json.tmp",
            response.receipt.receipt_id,
            Uuid::new_v4()
        ));
        let write_result = (|| -> Result<(), ContextGovernorError> {
            let mut temporary = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary_path)?;
            temporary.write_all(&bytes)?;
            temporary.sync_all()?;
            fs::rename(&temporary_path, &pending_path)?;
            Self::sync_directory(&self.pending_root())?;
            Ok(())
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        write_result?;

        Ok(PendingReceiptInfoV2 {
            schema: "PendingReceiptInfoV2".to_string(),
            receipt_id: persisted.receipt.receipt_id.clone(),
            session_id: persisted.receipt.session_id.clone(),
            generation: persisted.receipt.generation,
            created_utc: persisted.receipt.created_utc,
            pending_path,
            expected_compacted_message_count: persisted.compacted_messages.len(),
            expected_compacted_transcript_blake3: persisted
                .receipt
                .compacted_transcript_blake3
                .clone(),
            expected_compacted_transcript_sha256: persisted
                .receipt
                .compacted_transcript_sha256
                .clone(),
            expected_compacted_messages: persisted.compacted_messages,
            verified: true,
        })
    }

    fn read_pending_v2(
        &self,
        receipt_id: &str,
        operation: &str,
    ) -> Result<CompactResponseV2, ContextGovernorError> {
        let ring = self.require_v2_authority(operation)?;
        let path = self.pending_path_for_receipt(receipt_id)?;
        if !path.exists() {
            return Err(ContextGovernorError::PendingReceiptNotFound(
                receipt_id.to_string(),
            ));
        }
        let response: CompactResponseV2 = serde_json::from_slice(&fs::read(path)?)?;
        if response.receipt.receipt_id != receipt_id {
            return Err(lineage_error(receipt_id, "pending path identity mismatch"));
        }
        verify_v2_full_integrity(&response)?;
        verify_v2_authentication(&response, ring, true, operation)?;
        Ok(response)
    }

    pub fn list_pending_v2(
        &self,
        receipt_id: Option<&str>,
    ) -> Result<Vec<PendingReceiptInfoV2>, ContextGovernorError> {
        self.require_v2_authority("pending V2 receipt inspection")?;
        let pending_root = self.pending_root();
        if !pending_root.exists() {
            return Ok(Vec::new());
        }
        let mut ids = fs::read_dir(&pending_root)?
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                (path.extension().and_then(|value| value.to_str()) == Some("json"))
                    .then(|| path.file_stem()?.to_str().map(str::to_string))
                    .flatten()
            })
            .filter(|id| receipt_id.map_or(true, |expected| expected == id))
            .collect::<Vec<_>>();
        ids.sort();
        ids.into_iter()
            .map(|id| {
                let response = self.read_pending_v2(&id, "pending V2 receipt inspection")?;
                Ok(PendingReceiptInfoV2 {
                    schema: "PendingReceiptInfoV2".to_string(),
                    receipt_id: id.clone(),
                    session_id: response.receipt.session_id.clone(),
                    generation: response.receipt.generation,
                    created_utc: response.receipt.created_utc,
                    pending_path: self.pending_path_for_receipt(&id)?,
                    expected_compacted_message_count: response.compacted_messages.len(),
                    expected_compacted_transcript_blake3: response
                        .receipt
                        .compacted_transcript_blake3
                        .clone(),
                    expected_compacted_transcript_sha256: response
                        .receipt
                        .compacted_transcript_sha256
                        .clone(),
                    expected_compacted_messages: response.compacted_messages,
                    verified: true,
                })
            })
            .collect()
    }

    /// Activate a staged receipt only when the host's committed canonical
    /// projection is exactly the one bound by the pending receipt.
    pub fn activate_v2(
        &self,
        request: ReceiptActivationRequestV2,
    ) -> Result<ReceiptActivationResultV2, ContextGovernorError> {
        let response = self.read_pending_v2(&request.receipt_id, "V2 receipt activation")?;
        let actual_count = request.committed_messages.len();
        let actual_blake3 = crate::hash_messages(&request.committed_messages)?;
        let actual_sha256 = crate::hash_messages_sha256(&request.committed_messages)?;
        if actual_count != response.compacted_messages.len()
            || actual_blake3 != response.receipt.compacted_transcript_blake3
            || actual_sha256 != response.receipt.compacted_transcript_sha256
        {
            return Err(ContextGovernorError::CommittedTranscriptMismatch(Box::new(
                crate::CommittedTranscriptMismatchV2 {
                    receipt_id: request.receipt_id,
                    expected_count: response.compacted_messages.len(),
                    actual_count,
                    expected_blake3: response.receipt.compacted_transcript_blake3,
                    actual_blake3,
                    expected_sha256: response.receipt.compacted_transcript_sha256,
                    actual_sha256,
                },
            )));
        }

        fs::create_dir_all(&self.root)?;
        let _lock = self.lock_store()?;
        let path = self.path_for_receipt(&response.receipt.receipt_id)?;
        if path.exists() {
            return Err(ContextGovernorError::ReceiptAlreadyExists(
                response.receipt.receipt_id,
            ));
        }
        // Recheck the active tip under the publication lock. Another pending
        // child may have won activation after this receipt was prepared.
        self.validate_v2_append_position(&response)?;
        let pending_path = self.pending_path_for_receipt(&request.receipt_id)?;
        fs::rename(&pending_path, &path)?;
        Self::sync_directory(&self.root)?;
        let projection = response.as_v1_projection();
        if let Ok(fingerprint) =
            receipt_index::fingerprint_for_path(&response.receipt.receipt_id, &path)
        {
            if receipt_index::upsert_if_present(&self.root, &fingerprint, &projection).is_err() {
                let _ = self.invalidate_index();
            }
        } else {
            let _ = self.invalidate_index();
        }
        Ok(ReceiptActivationResultV2 {
            schema: "ReceiptActivationResultV2".to_string(),
            receipt_id: response.receipt.receipt_id,
            path,
            activated: true,
            verified: true,
        })
    }

    pub fn discard_pending_v2(
        &self,
        receipt_id: &str,
    ) -> Result<ReceiptDiscardResultV2, ContextGovernorError> {
        self.read_pending_v2(receipt_id, "pending V2 receipt discard")?;
        let _lock = self.lock_store()?;
        let path = self.pending_path_for_receipt(receipt_id)?;
        fs::remove_file(path)?;
        Self::sync_directory(&self.pending_root())?;
        Ok(ReceiptDiscardResultV2 {
            schema: "ReceiptDiscardResultV2".to_string(),
            receipt_id: receipt_id.to_string(),
            discarded: true,
        })
    }

    /// Return exact bytes from a V1 local fallback or a verified V2 origin.
    /// Projection integrity is intentionally not required on this path.
    pub fn expand_lineage(
        &self,
        receipt_id: &str,
        source_or_item_id: &str,
        max_chars: usize,
    ) -> Result<ContextExpandResult, ContextGovernorError> {
        let first = self.read_versioned_unverified(receipt_id)?;
        if let VersionedCompactResponse::V1(response) = first {
            let versioned = VersionedCompactResponse::V1(response.clone());
            self.verify_versioned_for_use(&versioned, false, "exact expansion")?;
            verify_exact_fallback_integrity(&response)?;
            verified_exact_fallback_item(&response, source_or_item_id)?;
            return context_expand(&response, source_or_item_id, max_chars).ok_or_else(|| {
                ContextGovernorError::ReceiptNotFound(source_or_item_id.to_string())
            });
        }

        let chain = self.collect_lineage(receipt_id, false)?;
        let newest = match &chain[0] {
            VersionedCompactResponse::V2(response) => response,
            VersionedCompactResponse::V1(_) => unreachable!("handled above"),
        };
        let matches = newest
            .receipt
            .covered_original_sources
            .iter()
            .filter(|source| {
                source.source_id == source_or_item_id
                    || source.origin_item_id.as_deref() == Some(source_or_item_id)
            })
            .collect::<Vec<_>>();
        if matches.len() > 1 {
            return Err(ContextGovernorError::AmbiguousLineageTarget(
                source_or_item_id.to_string(),
            ));
        }
        if let Some(source) = matches.first() {
            let origin = chain
                .iter()
                .find(|response| response.receipt_id() == source.origin_receipt_id)
                .ok_or_else(|| ContextGovernorError::LineageMissingAncestor {
                    receipt_id: receipt_id.to_string(),
                    ancestor_id: source.origin_receipt_id.clone(),
                })?;
            return match origin {
                VersionedCompactResponse::V1(response) => {
                    let item_id = source.origin_item_id.as_deref().ok_or_else(|| {
                        lineage_error(receipt_id, "legacy source has no exact item ID")
                    })?;
                    let item = verified_exact_fallback_item(response, item_id)?;
                    if item.content_blake3 != source.content_blake3
                        || hash_text_sha256(&item.content) != source.content_sha256
                    {
                        return Err(lineage_error(receipt_id, "legacy source hash mismatch"));
                    }
                    expand_content(
                        source_or_item_id,
                        &item.content,
                        &source.content_blake3,
                        max_chars,
                    )
                }
                VersionedCompactResponse::V2(response) => {
                    let item = response
                        .source_evidence
                        .iter()
                        .find(|item| item.source_id == source.source_id)
                        .ok_or_else(|| {
                            lineage_error(
                                receipt_id,
                                format!("original source {} is missing", source.source_id),
                            )
                        })?;
                    if source_ref(item) != **source {
                        return Err(lineage_error(receipt_id, "source reference hash mismatch"));
                    }
                    expand_content(
                        source_or_item_id,
                        &item.message.content,
                        &item.content_blake3,
                        max_chars,
                    )
                }
            };
        }

        // Preserve existing item-ID behavior for local/ancestor exact stores.
        let exact_matches = chain
            .iter()
            .filter_map(|response| {
                let projection = response.as_v1_projection();
                projection
                    .exact_store
                    .iter()
                    .any(|item| item.item_id == source_or_item_id)
                    .then_some(projection)
            })
            .collect::<Vec<_>>();
        if exact_matches.len() > 1 {
            return Err(ContextGovernorError::AmbiguousLineageTarget(
                source_or_item_id.to_string(),
            ));
        }
        let projection = exact_matches
            .first()
            .ok_or_else(|| ContextGovernorError::ReceiptNotFound(source_or_item_id.to_string()))?;
        let item = verified_exact_fallback_item(projection, source_or_item_id)?;
        expand_content(
            source_or_item_id,
            &item.content,
            &item.content_blake3,
            max_chars,
        )
    }

    /// Ancestors referenced by any retained V2 receipt are ineligible for the
    /// legacy count-based prune operation.
    pub(crate) fn lineage_protected_ancestor_ids(
        &self,
    ) -> Result<BTreeSet<String>, ContextGovernorError> {
        let mut protected = BTreeSet::new();
        for path in self.receipt_paths()? {
            let response: VersionedCompactResponse = serde_json::from_slice(&fs::read(&path)?)?;
            let VersionedCompactResponse::V2(response) = response else {
                continue;
            };
            let chain = self.collect_lineage(&response.receipt.receipt_id, false)?;
            for ancestor in chain.iter().skip(1) {
                protected.insert(ancestor.receipt_id().to_string());
            }
        }
        Ok(protected)
    }
}

fn expand_content(
    item_id: &str,
    content: &str,
    content_blake3: &str,
    max_chars: usize,
) -> Result<ContextExpandResult, ContextGovernorError> {
    let actual_blake3 = hash_text(content);
    if actual_blake3 != content_blake3 {
        return Err(ContextGovernorError::ExactFallbackIntegrityMismatch {
            item_id: item_id.to_string(),
            reason: format!("blake3 expected {content_blake3}, got {actual_blake3}"),
        });
    }
    let total_chars = content.chars().count();
    let truncated = total_chars > max_chars;
    Ok(ContextExpandResult {
        item_id: item_id.to_string(),
        exactness_scope: "canonical_utf8_text_v1".to_string(),
        content: if truncated {
            content.chars().take(max_chars).collect()
        } else {
            content.to_string()
        },
        content_blake3: content_blake3.to_string(),
        truncated,
    })
}

/// Return V1-compatible projection fields for CLI rendering/diff helpers.
pub fn v2_projection(response: &CompactResponseV2) -> CompactResponse {
    response.as_v1_projection()
}

/// Read only the schema string without treating unknown/new schemas as V1.
pub fn receipt_schema_from_json(value: &serde_json::Value) -> Option<&str> {
    value
        .get("receipt")
        .and_then(|receipt| receipt.get("schema"))
        .and_then(serde_json::Value::as_str)
}

/// Parse a versioned receipt file for non-authoritative tooling such as index
/// rebuild. The authoritative loader still performs integrity verification.
pub(crate) fn read_versioned_path(
    path: &Path,
) -> Result<VersionedCompactResponse, ContextGovernorError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

/// Build a V1-compatible projection for a mixed-schema derived index.
pub(crate) fn versioned_projection_from_path(
    path: &Path,
) -> Result<CompactResponse, ContextGovernorError> {
    Ok(read_versioned_path(path)?.as_v1_projection())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_boundary_fails_closed_instead_of_saturating() {
        assert!(matches!(
            next_generation("ctxr_boundary", u32::MAX),
            Err(ContextGovernorError::GenerationOverflow { receipt_id }) if receipt_id == "ctxr_boundary"
        ));
    }
}
