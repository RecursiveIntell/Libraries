use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

// TODO(P32 owner wiring): replace this local alias with the canonical digest
// primitive from stack-ids or the artifact contract owner crate once this
// standalone microkernel is wired into the larger workspace.
pub type DigestHex = String;
// TODO(P32 owner wiring): replace this local alias with the canonical JSON
// pointer/path contract type from the boundary/artifact owner crate.
pub type JsonPointerLikePath = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryLanguage {
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalizationProfile {
    /// Stable sorted object keys and compact JSON. Not full RFC 8785/JCS.
    StableSortedJsonV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmbiguityPolicy {
    Reject,
    Quarantine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnknownFieldPolicy {
    Allow,
    Reject,
    Quarantine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoercionPolicy {
    RejectByDefault,
    AllowDeclared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairPolicy {
    NoRepair,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustBoundary {
    Internal,
    External,
    ToolOutput,
    ModelOutput,
    EvidenceImport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpectedJsonType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCeilingsV1 {
    pub max_bytes: Option<usize>,
    pub max_nesting_depth: Option<usize>,
    pub max_object_keys: Option<usize>,
}

impl Default for ResourceCeilingsV1 {
    fn default() -> Self {
        Self {
            max_bytes: Some(1024 * 1024),
            max_nesting_depth: Some(64),
            max_object_keys: Some(10_000),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryCompilerProfileV1 {
    pub profile_id: String,
    pub language: BoundaryLanguage,
    pub dialect: String,
    pub schema_id: Option<String>,
    pub schema_version: Option<String>,
    pub canonicalization: CanonicalizationProfile,
    pub duplicate_key_policy: AmbiguityPolicy,
    pub unknown_field_policy: UnknownFieldPolicy,
    pub coercion_policy: CoercionPolicy,
    pub repair_policy: RepairPolicy,
    pub resource_ceilings: ResourceCeilingsV1,
    pub trust_boundary: TrustBoundary,
    pub treatment_critical_paths: Vec<JsonPointerLikePath>,
    pub allowed_degradation: Vec<String>,
    /// P31 minimal schema subset: top-level field allowlist.
    pub allowed_top_level_fields: Option<BTreeSet<String>>,
    /// P31 minimal schema subset: top-level type expectations.
    pub expected_field_types: BTreeMap<String, ExpectedJsonType>,
}

impl BoundaryCompilerProfileV1 {
    pub fn strict_json_default() -> Self {
        Self {
            profile_id: "boundary-profile:p31:strict-json".to_string(),
            language: BoundaryLanguage::Json,
            dialect: "json-rfc8259".to_string(),
            schema_id: None,
            schema_version: None,
            canonicalization: CanonicalizationProfile::StableSortedJsonV1,
            duplicate_key_policy: AmbiguityPolicy::Reject,
            unknown_field_policy: UnknownFieldPolicy::Allow,
            coercion_policy: CoercionPolicy::RejectByDefault,
            repair_policy: RepairPolicy::NoRepair,
            resource_ceilings: ResourceCeilingsV1::default(),
            trust_boundary: TrustBoundary::External,
            treatment_critical_paths: Vec::new(),
            allowed_degradation: Vec::new(),
            allowed_top_level_fields: None,
            expected_field_types: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParseStatus {
    Accepted,
    Rejected,
    Quarantined,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryErrorKind {
    MalformedJson,
    DuplicateKey,
    UnknownField,
    TypeMismatch,
    ResourceCeiling,
    TreatmentCriticalMissing,
    CanonicalizationFailure,
    UnsupportedLanguage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryErrorRecordV1 {
    pub kind: BoundaryErrorKind,
    pub path: Option<JsonPointerLikePath>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseReceiptV1 {
    pub receipt_id: String,
    pub raw_digest: DigestHex,
    pub parsed_digest: Option<DigestHex>,
    pub canonical_digest: Option<DigestHex>,
    pub parser: String,
    pub dialect: String,
    pub status: ParseStatus,
    pub errors: Vec<BoundaryErrorRecordV1>,
    pub ambiguity_detected: bool,
    pub resource_ceiling_triggered: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticImpact {
    None,
    NonTreatmentChanging,
    TreatmentChanging,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreatmentIntegrityDecision {
    NotApplicable,
    Preserved,
    MissingCriticalPath,
    ChangedWithoutWaiver,
    ChangedWithWaiver,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreatmentDifferenceV1 {
    pub path: JsonPointerLikePath,
    pub before_digest: Option<DigestHex>,
    pub after_digest: Option<DigestHex>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairReceiptV1 {
    pub receipt_id: String,
    pub repair_operator: String,
    pub before_digest: DigestHex,
    pub after_digest: DigestHex,
    pub changed_paths: Vec<JsonPointerLikePath>,
    pub rationale: String,
    pub semantic_impact: SemanticImpact,
    pub allowed_changes: Vec<String>,
    pub disallowed_changes: Vec<String>,
    pub treatment_integrity_status: TreatmentIntegrityDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreatmentIntegrityReceiptV1 {
    pub receipt_id: String,
    pub treatment_critical_paths: Vec<JsonPointerLikePath>,
    pub before_hashes: BTreeMap<String, Option<DigestHex>>,
    pub after_hashes: BTreeMap<String, Option<DigestHex>>,
    pub differences: Vec<TreatmentDifferenceV1>,
    pub decision: TreatmentIntegrityDecision,
    pub waiver: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryDecisionV1 {
    Accept,
    Reject,
    Quarantine,
    RepairedAccept,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundaryCompileResultV1 {
    pub decision: BoundaryDecisionV1,
    pub value: Option<serde_json::Value>,
    pub canonical_bytes: Option<Vec<u8>>,
    pub raw_digest: DigestHex,
    pub canonical_digest: Option<DigestHex>,
    pub parse_receipt: ParseReceiptV1,
    pub repair_receipt: Option<RepairReceiptV1>,
    pub treatment_integrity_receipt: Option<TreatmentIntegrityReceiptV1>,
    pub errors: Vec<BoundaryErrorRecordV1>,
}
