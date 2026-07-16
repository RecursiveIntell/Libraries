use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BoundaryRepairReportV1 {
    pub receipt_id: ArtifactId,
    pub changed: bool,
    pub repair_kind: String,
    pub before_digest: Option<String>,
    pub after_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_repair_record_ids: Vec<StackBoundaryRepairRecordId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DisplayDigestV1 {
    pub algorithm: String,
    pub canonicalization: String,
    pub digest: String,
    #[serde(default = "default_true")]
    pub non_authoritative: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl DisplayDigestV1 {
    pub fn for_json_value(value: &serde_json::Value) -> Self {
        Self {
            algorithm: "blake3".into(),
            canonicalization: "stack-ids-json-c14n-v1-display-only".into(),
            digest: non_authoritative_json_display_digest(value),
            non_authoritative: true,
            reason_codes: vec!["display-only-not-artifact-identity".into()],
        }
    }

    pub fn for_text(text: &str) -> Self {
        Self {
            algorithm: "blake3".into(),
            canonicalization: "stack-ids-utf8-text-v1-display-only".into(),
            digest: non_authoritative_text_display_digest(text),
            non_authoritative: true,
            reason_codes: vec!["display-only-not-artifact-identity".into()],
        }
    }

    /// Construct a DisplayDigestV1 from a pre-computed hex digest string.
    ///
    /// Use this when the digest has already been computed by a boundary
    /// compiler or other subsystem that produces SHA-256 hex strings.
    pub fn from_hex(hex_digest: impl Into<String>) -> Self {
        Self {
            algorithm: "sha256".into(),
            canonicalization: "boundary-compiler-sha256-hex-display-only".into(),
            digest: hex_digest.into(),
            non_authoritative: true,
            reason_codes: vec!["display-only-not-artifact-identity".into()],
        }
    }

    pub fn from_stack_content_digest_for_display(
        digest: StackContentDigest,
        canonicalization: impl Into<String>,
    ) -> Self {
        Self {
            algorithm: "blake3".into(),
            canonicalization: format!("{}-display-only", canonicalization.into()),
            digest: non_authoritative_display_digest_string(&digest),
            non_authoritative: true,
            reason_codes: vec!["display-only-not-artifact-identity".into()],
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DuplicateKeyFindingV1 {
    pub finding_id: ArtifactId,
    pub path: String,
    pub key: String,
    pub first_offset: Option<usize>,
    pub duplicate_offset: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl DuplicateKeyFindingV1 {
    pub fn new(
        path: impl Into<String>,
        key: impl Into<String>,
        first_offset: Option<usize>,
        duplicate_offset: Option<usize>,
    ) -> Self {
        Self {
            finding_id: display_only_unstable_id("duplicate-key-finding"),
            path: path.into(),
            key: key.into(),
            first_offset,
            duplicate_offset,
            reason_codes: vec!["duplicate-json-object-key".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct JsonBoundaryRepairDisplayReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub changed: bool,
    pub repair_kind: String,
    pub degraded: bool,
    pub before_raw_digest: Option<String>,
    pub after_raw_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before_display_digest: Option<DisplayDigestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_display_digest: Option<DisplayDigestV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub treatment_critical_fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub treatment_integrity_warnings: Vec<String>,
    #[serde(default)]
    pub hard_failed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_repair_record_ids: Vec<StackBoundaryRepairRecordId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
}

impl JsonBoundaryRepairDisplayReportV1 {
    pub fn none() -> Self {
        Self {
            receipt_id: display_only_unstable_id("json-repair"),
            kind: ArtifactKindV1::BoundaryRepair,
            changed: false,
            repair_kind: "none".into(),
            degraded: false,
            before_raw_digest: None,
            after_raw_digest: None,
            before_display_digest: None,
            after_display_digest: None,
            treatment_critical_fields: Vec::new(),
            treatment_integrity_warnings: Vec::new(),
            hard_failed: false,
            warnings: Vec::new(),
            reason_codes: Vec::new(),
            canonical_repair_record_ids: Vec::new(),
            canonical_backpointers: canonical_owner_backpointer(
                "verification-control",
                "BoundaryRepairRecord",
                "canonical-boundary-repair-owner",
            ),
        }
    }
}

impl From<JsonBoundaryRepairDisplayReportV1> for BoundaryRepairReportV1 {
    fn from(receipt: JsonBoundaryRepairDisplayReportV1) -> Self {
        Self {
            receipt_id: receipt.receipt_id,
            changed: receipt.changed,
            repair_kind: receipt.repair_kind,
            before_digest: receipt.before_raw_digest,
            after_digest: receipt.after_raw_digest,
            canonical_repair_record_ids: receipt.canonical_repair_record_ids,
            canonical_backpointers: receipt.canonical_backpointers,
            warnings: receipt
                .warnings
                .into_iter()
                .chain(receipt.treatment_integrity_warnings)
                .collect(),
        }
    }
}

impl From<&JsonBoundaryRepairDisplayReportV1> for BoundaryRepairReportV1 {
    fn from(receipt: &JsonBoundaryRepairDisplayReportV1) -> Self {
        receipt.clone().into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SchemaValidationReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_digest: Option<DisplayDigestV1>,
    pub input_digest: DisplayDigestV1,
    pub valid: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_control_receipt_ids: Vec<StackControlReceiptId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub checked_at: DateTime<Utc>,
}

impl SchemaValidationReportV1 {
    pub fn new(
        schema: Option<&serde_json::Value>,
        input: &serde_json::Value,
        errors: Vec<String>,
    ) -> Self {
        let valid = errors.is_empty();
        Self {
            receipt_id: display_only_unstable_id("schema-validation"),
            kind: ArtifactKindV1::SchemaValidation,
            run_id: None,
            attempt_id: None,
            tool_id: None,
            schema_digest: schema.map(DisplayDigestV1::for_json_value),
            input_digest: DisplayDigestV1::for_json_value(input),
            valid,
            errors,
            reason_codes: if valid {
                vec!["schema-validation-passed".into()]
            } else {
                vec!["schema-validation-failed".into()]
            },
            canonical_control_receipt_ids: Vec::new(),
            canonical_backpointers: canonical_owner_backpointer(
                "verification-control",
                "ControlReceipt",
                "canonical-schema-validation-owner",
            ),
            checked_at: Utc::now(),
        }
    }

    pub fn with_tool_id(mut self, tool_id: impl Into<String>) -> Self {
        self.tool_id = Some(tool_id.into());
        self
    }

    pub fn with_execution_context(mut self, context: &AidensRunContextV1) -> Self {
        self.run_id = Some(context.run_id.clone());
        self.attempt_id = Some(context.attempt_id.clone());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BoundaryCompileRequestV1 {
    pub request_id: ArtifactId,
    pub input: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    pub schema_dialect: String,
    pub allow_markdown_fence_repair: bool,
    pub allow_json_substring_extract: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub treatment_critical_fields: Vec<String>,
    #[serde(default)]
    pub hard_fail_on_treatment_change: bool,
}

impl BoundaryCompileRequestV1 {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            request_id: display_only_unstable_id("boundary-compile-request"),
            input: input.into(),
            schema: None,
            schema_dialect: "json-schema-2020-12-subset".into(),
            allow_markdown_fence_repair: false,
            allow_json_substring_extract: false,
            treatment_critical_fields: Vec::new(),
            hard_fail_on_treatment_change: false,
        }
    }

    pub fn with_schema(mut self, schema: serde_json::Value) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn with_treatment_critical_fields(mut self, fields: Vec<String>) -> Self {
        self.treatment_critical_fields = fields;
        self
    }

    pub fn with_hard_fail_on_treatment_change(mut self, hard_fail: bool) -> Self {
        self.hard_fail_on_treatment_change = hard_fail;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BoundaryCompileOutcomeV1 {
    pub outcome_id: ArtifactId,
    pub request_id: ArtifactId,
    pub accepted: bool,
    pub degraded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_digest: Option<DisplayDigestV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub duplicate_key_findings: Vec<DuplicateKeyFindingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_validation: Option<SchemaValidationReportV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repair_receipt: Option<JsonBoundaryRepairDisplayReportV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub compiled_at: DateTime<Utc>,
}

impl BoundaryCompileOutcomeV1 {
    pub fn rejected(request_id: ArtifactId, reason_codes: Vec<String>) -> Self {
        Self {
            outcome_id: display_only_unstable_id("boundary-compile-outcome"),
            request_id,
            accepted: false,
            degraded: true,
            value: None,
            display_digest: None,
            duplicate_key_findings: Vec::new(),
            schema_validation: None,
            repair_receipt: None,
            reason_codes,
            compiled_at: Utc::now(),
        }
    }
}

/// AiDENs-local, non-authoritative schema index for product/display/report DTOs.
///
/// Canonical stack artifact family schemas are owned by their owner crates and
/// `contract-schema-gen`, not by this registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactFamilyRegistryV1 {
    pub registry_id: ArtifactId,
    pub registry_version: u32,
    pub families: Vec<ArtifactFamilyRegistrationV1>,
    #[serde(default = "default_schema_canonical_truth_owner")]
    pub canonical_truth_owner: String,
    #[serde(default = "schema_registry_governance_local_display")]
    pub governance_status: SchemaRegistryGovernanceStatusV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaRegistryGovernanceStatusV1 {
    LocalDisplayIndexOnly,
    QuarantinedExternalFamilies,
    AdmittedByCanonicalOwner,
}

fn default_schema_canonical_truth_owner() -> String {
    "contract-schema-gen".into()
}

fn schema_registry_governance_local_display() -> SchemaRegistryGovernanceStatusV1 {
    SchemaRegistryGovernanceStatusV1::LocalDisplayIndexOnly
}

impl ArtifactFamilyRegistryV1 {
    pub fn new(mut families: Vec<ArtifactFamilyRegistrationV1>) -> Self {
        families.sort_by(|left, right| {
            left.family
                .cmp(&right.family)
                .then(left.version.cmp(&right.version))
        });
        Self {
            registry_id: ArtifactId::new("artifact-family-registry:v1"),
            registry_version: 1,
            families,
            canonical_truth_owner: "contract-schema-gen".into(),
            governance_status: SchemaRegistryGovernanceStatusV1::LocalDisplayIndexOnly,
            reason_codes: vec!["aidens-local-non-authoritative-schema-index".into()],
        }
    }

    pub fn contains_family_version(&self, family: &str, version: u32) -> bool {
        self.families
            .iter()
            .any(|entry| entry.family == family && entry.version == version)
    }
}

/// AiDENs-local schema registration metadata.
///
/// Entries in this structure are not canonical stack artifact ownership claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactFamilyRegistrationV1 {
    pub family: String,
    pub version: u32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub schema_identity: String,
    pub rust_type: String,
    pub owner_crate: String,
    #[serde(default = "schema_family_admission_local_display")]
    pub admission: SchemaFamilyAdmissionV1,
    pub first_pass: String,
    pub schema_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture_path: Option<String>,
    pub compatibility_policy: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaFamilyAdmissionV1 {
    LocalDisplayOnly,
    QuarantinedExternal,
    AdmittedByCanonicalOwner,
}

fn schema_family_admission_local_display() -> SchemaFamilyAdmissionV1 {
    SchemaFamilyAdmissionV1::LocalDisplayOnly
}

impl SchemaFamilyAdmissionV1 {
    pub fn allows_local_generation(self) -> bool {
        matches!(
            self,
            Self::LocalDisplayOnly | Self::AdmittedByCanonicalOwner
        )
    }

    pub fn allows_truth_ownership_claim(self) -> bool {
        matches!(self, Self::AdmittedByCanonicalOwner)
    }
}

impl ArtifactFamilyRegistrationV1 {
    pub fn new(
        family: impl Into<String>,
        version: u32,
        rust_type: impl Into<String>,
        first_pass: impl Into<String>,
        fixture_path: Option<String>,
        compatibility_policy: impl Into<String>,
    ) -> Self {
        let family = family.into();
        Self {
            schema_path: format!("{family}/v{version}.schema.json"),
            schema_identity: format!("schema:{family}:v{version}"),
            family,
            version,
            rust_type: rust_type.into(),
            owner_crate: "aidens-orchestration".into(),
            admission: SchemaFamilyAdmissionV1::LocalDisplayOnly,
            first_pass: first_pass.into(),
            fixture_path,
            compatibility_policy: compatibility_policy.into(),
        }
    }

    pub fn quarantined_external(
        family: impl Into<String>,
        version: u32,
        schema_path: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        let family = family.into();
        let reason = reason.into();
        Self {
            schema_identity: format!("schema:{family}:v{version}:external-quarantined"),
            schema_path: schema_path.into(),
            family,
            version,
            rust_type: "external-unadmitted".into(),
            owner_crate: "external-unadmitted".into(),
            admission: SchemaFamilyAdmissionV1::QuarantinedExternal,
            first_pass: "external".into(),
            fixture_path: None,
            compatibility_policy: format!("quarantined:{reason}"),
        }
    }
}

/// AiDENs-local, non-authoritative manifest for generated display/report schemas.
///
/// Canonical family schema generation is delegated to `contract-schema-gen`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GeneratedSchemaManifestV1 {
    pub manifest_id: ArtifactId,
    pub manifest_version: u32,
    pub registry_digest: String,
    pub schemas: Vec<GeneratedSchemaEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl GeneratedSchemaManifestV1 {
    pub fn new(
        registry: &ArtifactFamilyRegistryV1,
        mut schemas: Vec<GeneratedSchemaEntryV1>,
    ) -> Self {
        schemas.sort_by(|left, right| {
            left.family
                .cmp(&right.family)
                .then(left.version.cmp(&right.version))
        });
        Self {
            manifest_id: ArtifactId::new("generated-schema-manifest:v1"),
            manifest_version: 1,
            registry_digest: non_authoritative_json_display_digest(
                &serde_json::to_value(registry).unwrap_or(serde_json::Value::Null),
            ),
            schemas,
            reason_codes: vec!["aidens-local-display-report-schemas-only".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GeneratedSchemaEntryV1 {
    pub family: String,
    pub version: u32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub schema_identity: String,
    pub rust_type: String,
    pub schema_path: String,
    pub schema_digest: String,
}

/// AiDENs-local, non-authoritative generated schema document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GeneratedSchemaDocumentV1 {
    pub registration: ArtifactFamilyRegistrationV1,
    pub schema: serde_json::Value,
}

impl GeneratedSchemaDocumentV1 {
    pub fn entry(&self) -> GeneratedSchemaEntryV1 {
        GeneratedSchemaEntryV1 {
            family: self.registration.family.clone(),
            version: self.registration.version,
            schema_identity: self.content_addressed_identity(),
            rust_type: self.registration.rust_type.clone(),
            schema_path: self.registration.schema_path.clone(),
            schema_digest: non_authoritative_json_display_digest(&self.schema),
        }
    }

    pub fn content_addressed_identity(&self) -> String {
        format!(
            "{}:{}",
            self.registration.schema_identity,
            non_authoritative_json_display_digest(&self.schema)
        )
    }

    pub fn pretty_json(&self) -> String {
        let mut encoded = serde_json::to_string_pretty(&self.schema).unwrap_or_default();
        encoded.push('\n');
        encoded
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaCompatibilityModeV1 {
    Backward,
    Forward,
    Full,
    Transitive,
}

impl fmt::Display for SchemaCompatibilityModeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Backward => "backward",
            Self::Forward => "forward",
            Self::Full => "full",
            Self::Transitive => "transitive",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaChangeClassV1 {
    Exact,
    AdditiveMinor,
    MajorBreaking,
    UnknownIncompatible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SchemaCompatibilityCheckV1 {
    pub family: String,
    pub version: u32,
    pub mode: SchemaCompatibilityModeV1,
    #[serde(default = "schema_change_class_unknown_incompatible")]
    pub change_class: SchemaChangeClassV1,
    #[serde(default)]
    pub requires_major_bump: bool,
    pub compatible: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

fn schema_change_class_unknown_incompatible() -> SchemaChangeClassV1 {
    SchemaChangeClassV1::UnknownIncompatible
}

impl SchemaCompatibilityCheckV1 {
    pub fn exact(family: impl Into<String>, version: u32, mode: SchemaCompatibilityModeV1) -> Self {
        Self {
            family: family.into(),
            version,
            mode,
            change_class: SchemaChangeClassV1::Exact,
            requires_major_bump: false,
            compatible: true,
            reason_codes: vec![format!("schema-compatible-{mode}")],
        }
    }

    pub fn incompatible(
        family: impl Into<String>,
        version: u32,
        mode: SchemaCompatibilityModeV1,
        reason_codes: Vec<String>,
    ) -> Self {
        Self {
            family: family.into(),
            version,
            mode,
            change_class: SchemaChangeClassV1::UnknownIncompatible,
            requires_major_bump: true,
            compatible: false,
            reason_codes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SchemaCompatibilityReportV1 {
    pub report_id: ArtifactId,
    pub registry_digest: String,
    pub compatible: bool,
    pub checked_schema_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<SchemaCompatibilityCheckV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_schema_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unregistered_schema_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incompatible_schema_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path_collision_findings: Vec<SchemaPathCollisionFindingV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SchemaPathCollisionFindingV1 {
    pub finding_id: ArtifactId,
    pub normalized_path: String,
    pub colliding_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl SchemaPathCollisionFindingV1 {
    pub fn new(normalized_path: impl Into<String>, colliding_paths: Vec<String>) -> Self {
        Self {
            finding_id: display_only_unstable_id("schema-path-collision"),
            normalized_path: normalized_path.into(),
            colliding_paths: sorted_unique_strings(colliding_paths),
            reason_codes: vec!["schema-path-case-fold-collision".into()],
        }
    }
}

impl SchemaCompatibilityReportV1 {
    pub fn new(
        registry: &ArtifactFamilyRegistryV1,
        checked_schema_count: usize,
        checks: Vec<SchemaCompatibilityCheckV1>,
        missing_schema_paths: Vec<String>,
        unregistered_schema_paths: Vec<String>,
        incompatible_schema_paths: Vec<String>,
        path_collision_findings: Vec<SchemaPathCollisionFindingV1>,
    ) -> Self {
        let compatible = missing_schema_paths.is_empty()
            && unregistered_schema_paths.is_empty()
            && incompatible_schema_paths.is_empty()
            && path_collision_findings.is_empty()
            && checks.iter().all(|check| check.compatible);
        let mut reason_codes = Vec::new();
        if compatible {
            reason_codes.push("schema-compatibility-passed".into());
        }
        if !missing_schema_paths.is_empty() {
            reason_codes.push("registered-schema-missing".into());
        }
        if !unregistered_schema_paths.is_empty() {
            reason_codes.push("unregistered-artifact-family-schema".into());
        }
        if !incompatible_schema_paths.is_empty() {
            reason_codes.push("schema-content-drift-without-major-bump".into());
        }
        if !path_collision_findings.is_empty() {
            reason_codes.push("schema-path-case-fold-collision".into());
        }
        Self {
            report_id: ArtifactId::new("schema-compatibility-report:v1"),
            registry_digest: non_authoritative_json_display_digest(
                &serde_json::to_value(registry).unwrap_or(serde_json::Value::Null),
            ),
            compatible,
            checked_schema_count,
            checks,
            missing_schema_paths,
            unregistered_schema_paths,
            incompatible_schema_paths,
            path_collision_findings,
            reason_codes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MigrationPhaseV1 {
    Expand,
    Backfill,
    FlipRead,
    Contract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MigrationPlanV1 {
    pub plan_id: ArtifactId,
    pub artifact_family: String,
    pub from_version: u32,
    pub to_version: u32,
    pub phases: Vec<MigrationPhaseV1>,
    pub requires_major_bump: bool,
    pub compatible_without_backfill: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl MigrationPlanV1 {
    pub fn expand_backfill_flip_contract(
        artifact_family: impl Into<String>,
        from_version: u32,
        to_version: u32,
        requires_major_bump: bool,
    ) -> Self {
        Self {
            plan_id: ArtifactId::new("migration-plan:v1"),
            artifact_family: artifact_family.into(),
            from_version,
            to_version,
            phases: vec![
                MigrationPhaseV1::Expand,
                MigrationPhaseV1::Backfill,
                MigrationPhaseV1::FlipRead,
                MigrationPhaseV1::Contract,
            ],
            requires_major_bump,
            compatible_without_backfill: !requires_major_bump,
            reason_codes: vec!["expand-backfill-flip-read-contract".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BackfillReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub migration_plan_id: ArtifactId,
    pub artifact_family: String,
    pub from_version: u32,
    pub to_version: u32,
    pub migrated_fixture_count: usize,
    pub succeeded: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_fixture_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub backfilled_at: DateTime<Utc>,
}

impl BackfillReportV1 {
    pub fn succeeded(
        plan: &MigrationPlanV1,
        migrated_fixture_count: usize,
        source_fixture_paths: Vec<String>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("backfill"),
            kind: ArtifactKindV1::Backfill,
            migration_plan_id: plan.plan_id.clone(),
            artifact_family: plan.artifact_family.clone(),
            from_version: plan.from_version,
            to_version: plan.to_version,
            migrated_fixture_count,
            succeeded: true,
            source_fixture_paths,
            reason_codes: vec!["historical-fixtures-readable".into()],
            backfilled_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceDomainV1 {
    PlanConfig,
    ProviderRoute,
    ToolExposure,
    Permit,
    BoundaryRepair,
    ReceiptLineage,
    TemporalQuery,
    ProofDebt,
    SemanticState,
    ViewDisclosure,
}

impl fmt::Display for ReferenceDomainV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PlanConfig => "plan-config",
            Self::ProviderRoute => "provider-route",
            Self::ToolExposure => "tool-exposure",
            Self::Permit => "permit",
            Self::BoundaryRepair => "boundary-repair",
            Self::ReceiptLineage => "receipt-lineage",
            Self::TemporalQuery => "temporal-query",
            Self::ProofDebt => "proof-debt",
            Self::SemanticState => "semantic-state",
            Self::ViewDisclosure => "view-disclosure",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReferenceCaseV1 {
    pub case_id: ArtifactId,
    pub domain: ReferenceDomainV1,
    pub title: String,
    pub input: serde_json::Value,
    pub expected: serde_json::Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_provider_kinds: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_risk_classes: Vec<CanonicalToolSideEffectClass>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_memory_modes: Vec<MemoryModeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_receipt_levels: Vec<ReportLevelV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_tool_lifecycle_states: Vec<ToolLifecycleStateV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_fixture_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ReferenceCaseV1 {
    pub fn new(
        domain: ReferenceDomainV1,
        title: impl Into<String>,
        input: serde_json::Value,
        expected: serde_json::Value,
    ) -> Self {
        Self {
            case_id: display_only_unstable_id("reference-case"),
            domain,
            title: title.into(),
            input,
            expected,
            covered_provider_kinds: Vec::new(),
            covered_risk_classes: Vec::new(),
            covered_memory_modes: Vec::new(),
            covered_receipt_levels: Vec::new(),
            covered_tool_lifecycle_states: Vec::new(),
            source_fixture_paths: Vec::new(),
            reason_codes: vec!["reference-semantics-case".into()],
        }
    }

    pub fn with_provider_kind(mut self, provider_kind: impl Into<String>) -> Self {
        self.covered_provider_kinds.push(provider_kind.into());
        self
    }

    pub fn with_risk_class(mut self, risk_class: CanonicalToolSideEffectClass) -> Self {
        self.covered_risk_classes.push(risk_class);
        self
    }

    pub fn with_memory_mode(mut self, memory_mode: MemoryModeV1) -> Self {
        self.covered_memory_modes.push(memory_mode);
        self
    }

    pub fn with_receipt_level(mut self, receipt_level: ReportLevelV1) -> Self {
        self.covered_receipt_levels.push(receipt_level);
        self
    }

    pub fn with_tool_lifecycle_state(mut self, state: ToolLifecycleStateV1) -> Self {
        self.covered_tool_lifecycle_states.push(state);
        self
    }

    pub fn with_source_fixture(mut self, path: impl Into<String>) -> Self {
        self.source_fixture_paths.push(path.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DifferentialConformanceFindingV1 {
    pub finding_id: ArtifactId,
    pub case_id: ArtifactId,
    pub domain: ReferenceDomainV1,
    pub production_subject: String,
    pub path: String,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
    pub human_diff: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl DifferentialConformanceFindingV1 {
    pub fn mismatch(
        case: &ReferenceCaseV1,
        production_subject: impl Into<String>,
        path: impl Into<String>,
        expected: serde_json::Value,
        actual: serde_json::Value,
    ) -> Self {
        let path = path.into();
        let human_diff = format!(
            "{} mismatch at {path}: expected {}, actual {}",
            case.title,
            display_json_string(&expected),
            display_json_string(&actual)
        );
        Self {
            finding_id: display_only_unstable_id("differential-conformance-finding"),
            case_id: case.case_id.clone(),
            domain: case.domain,
            production_subject: production_subject.into(),
            path,
            expected,
            actual,
            human_diff,
            reason_codes: vec!["reference-production-mismatch".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReferenceInterpreterReportV1 {
    pub report_id: ArtifactId,
    pub interpreter_id: String,
    pub case_count: usize,
    pub passed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<DifferentialConformanceFindingV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_provider_kinds: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_risk_classes: Vec<CanonicalToolSideEffectClass>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_memory_modes: Vec<MemoryModeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_receipt_levels: Vec<ReportLevelV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_tool_lifecycle_states: Vec<ToolLifecycleStateV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl ReferenceInterpreterReportV1 {
    pub fn new(
        interpreter_id: impl Into<String>,
        case_count: usize,
        findings: Vec<DifferentialConformanceFindingV1>,
        cases: &[ReferenceCaseV1],
    ) -> Self {
        let passed = findings.is_empty();
        Self {
            report_id: display_only_unstable_id("reference-interpreter-report"),
            interpreter_id: interpreter_id.into(),
            case_count,
            passed,
            findings,
            covered_provider_kinds: sorted_unique_strings(
                cases
                    .iter()
                    .flat_map(|case| case.covered_provider_kinds.iter().cloned())
                    .collect(),
            ),
            covered_risk_classes: sorted_unique_risk_classes(
                cases
                    .iter()
                    .flat_map(|case| case.covered_risk_classes.iter().cloned())
                    .collect(),
            ),
            covered_memory_modes: unique_memory_modes(
                cases
                    .iter()
                    .flat_map(|case| case.covered_memory_modes.iter().cloned())
                    .collect(),
            ),
            covered_receipt_levels: unique_receipt_levels(
                cases
                    .iter()
                    .flat_map(|case| case.covered_receipt_levels.iter().cloned())
                    .collect(),
            ),
            covered_tool_lifecycle_states: unique_tool_lifecycle_states(
                cases
                    .iter()
                    .flat_map(|case| case.covered_tool_lifecycle_states.iter().cloned())
                    .collect(),
            ),
            reason_codes: if passed {
                vec!["reference-conformance-passed".into()]
            } else {
                vec!["reference-conformance-failed".into()]
            },
            generated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GoldenFixtureManifestV1 {
    pub manifest_id: ArtifactId,
    pub fixture_paths: Vec<String>,
    pub reference_case_ids: Vec<ArtifactId>,
    pub provider_kinds: Vec<String>,
    pub risk_classes: Vec<CanonicalToolSideEffectClass>,
    pub memory_modes: Vec<MemoryModeV1>,
    pub receipt_levels: Vec<ReportLevelV1>,
    pub tool_lifecycle_states: Vec<ToolLifecycleStateV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl GoldenFixtureManifestV1 {
    pub fn new(fixture_paths: Vec<String>, cases: &[ReferenceCaseV1]) -> Self {
        Self {
            manifest_id: display_only_unstable_id("golden-fixture-manifest"),
            fixture_paths: sorted_unique_strings(fixture_paths),
            reference_case_ids: cases.iter().map(|case| case.case_id.clone()).collect(),
            provider_kinds: sorted_unique_strings(
                cases
                    .iter()
                    .flat_map(|case| case.covered_provider_kinds.iter().cloned())
                    .collect(),
            ),
            risk_classes: sorted_unique_risk_classes(
                cases
                    .iter()
                    .flat_map(|case| case.covered_risk_classes.iter().cloned())
                    .collect(),
            ),
            memory_modes: unique_memory_modes(
                cases
                    .iter()
                    .flat_map(|case| case.covered_memory_modes.iter().cloned())
                    .collect(),
            ),
            receipt_levels: unique_receipt_levels(
                cases
                    .iter()
                    .flat_map(|case| case.covered_receipt_levels.iter().cloned())
                    .collect(),
            ),
            tool_lifecycle_states: unique_tool_lifecycle_states(
                cases
                    .iter()
                    .flat_map(|case| case.covered_tool_lifecycle_states.iter().cloned())
                    .collect(),
            ),
            reason_codes: vec!["golden-fixture-coverage-manifest".into()],
            generated_at: Utc::now(),
        }
    }
}

pub(crate) fn sorted_unique_artifact_ids(mut values: Vec<ArtifactId>) -> Vec<ArtifactId> {
    values.sort();
    values.dedup();
    values
}

pub(crate) fn sorted_unique_invariants(
    mut values: Vec<PreservedInvariantV1>,
) -> Vec<PreservedInvariantV1> {
    values.sort();
    values.dedup();
    values
}

pub(crate) fn sorted_unique_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn sorted_unique_risk_classes(
    mut values: Vec<CanonicalToolSideEffectClass>,
) -> Vec<CanonicalToolSideEffectClass> {
    values.sort();
    values.dedup();
    values
}

fn unique_memory_modes(values: Vec<MemoryModeV1>) -> Vec<MemoryModeV1> {
    let mut out = Vec::new();
    for candidate in [
        MemoryModeV1::Disabled,
        MemoryModeV1::Optional,
        MemoryModeV1::Required,
    ] {
        if values.contains(&candidate) {
            out.push(candidate);
        }
    }
    out
}

fn unique_receipt_levels(values: Vec<ReportLevelV1>) -> Vec<ReportLevelV1> {
    let mut out = Vec::new();
    for candidate in [
        ReportLevelV1::Minimal,
        ReportLevelV1::Standard,
        ReportLevelV1::Full,
    ] {
        if values.contains(&candidate) {
            out.push(candidate);
        }
    }
    out
}

fn unique_tool_lifecycle_states(values: Vec<ToolLifecycleStateV1>) -> Vec<ToolLifecycleStateV1> {
    let mut out = Vec::new();
    for candidate in [
        ToolLifecycleStateV1::Declared,
        ToolLifecycleStateV1::Registered,
        ToolLifecycleStateV1::Executable,
        ToolLifecycleStateV1::Exposed,
        ToolLifecycleStateV1::ExposedThisTurn,
        ToolLifecycleStateV1::Invoked,
        ToolLifecycleStateV1::Succeeded,
        ToolLifecycleStateV1::Failed,
        ToolLifecycleStateV1::Hidden,
        ToolLifecycleStateV1::Blocked,
    ] {
        if values.contains(&candidate) {
            out.push(candidate);
        }
    }
    out
}

pub fn current_artifact_family_registry() -> ArtifactFamilyRegistryV1 {
    ArtifactFamilyRegistryV1::new(artifact_family_registrations())
}

pub fn generated_schema_documents() -> Vec<GeneratedSchemaDocumentV1> {
    generated_schema_documents_inner()
}

pub fn generated_schema_manifest() -> GeneratedSchemaManifestV1 {
    let registry = current_artifact_family_registry();
    GeneratedSchemaManifestV1::new(
        &registry,
        generated_schema_documents()
            .into_iter()
            .map(|document| document.entry())
            .collect(),
    )
}

pub fn generated_schema_manifest_pretty_json() -> String {
    let mut encoded =
        serde_json::to_string_pretty(&generated_schema_manifest()).unwrap_or_default();
    encoded.push('\n');
    encoded
}

macro_rules! artifact_registration {
    ($family:literal, $version:literal, $type_name:literal, $pass:literal, $fixture:expr, $policy:literal) => {{
        let fixture_path: Option<&str> = $fixture;
        ArtifactFamilyRegistrationV1::new(
            $family,
            $version,
            $type_name,
            $pass,
            fixture_path.map(str::to_string),
            $policy,
        )
    }};
}

macro_rules! schema_document {
    ($family:literal, $version:literal, $ty:ty, $type_name:literal, $pass:literal, $fixture:expr, $policy:literal) => {
        GeneratedSchemaDocumentV1 {
            registration: artifact_registration!(
                $family, $version, $type_name, $pass, $fixture, $policy
            ),
            schema: serde_json::to_value(schemars::schema_for!($ty))
                .unwrap_or(serde_json::Value::Null),
        }
    };
}

fn artifact_family_registrations() -> Vec<ArtifactFamilyRegistrationV1> {
    generated_schema_documents_inner()
        .into_iter()
        .map(|document| document.registration)
        .collect()
}

fn generated_schema_documents_inner() -> Vec<GeneratedSchemaDocumentV1> {
    vec![
        schema_document!(
            "agent-spec",
            1,
            AgentSpecV1,
            "AgentSpecV1",
            "P26",
            Some("tests/fixtures/p26/agent_spec_v1.json"),
            "local-agent-declarative-contract-major-immutable"
        ),
        schema_document!(
            "api-honesty-receipt",
            1,
            ApiHonestyReportV1,
            "ApiHonestyReportV1",
            "P01",
            Some("tests/fixtures/p01/api_honesty_receipt_v1.json"),
            "additive-compatible-with-defaults"
        ),
        schema_document!(
            "approval-decision",
            1,
            ApprovalDecisionV1,
            "ApprovalDecisionV1",
            "P04",
            Some("tests/fixtures/p04/approval_decision_v1.json"),
            "append-only-approval-outcome"
        ),
        schema_document!(
            "approval-request",
            1,
            ApprovalRequestV1,
            "ApprovalRequestV1",
            "P04",
            Some("tests/fixtures/p04/approval_request_v1.json"),
            "scope-additions-require-defaults"
        ),
        schema_document!(
            "artifact-family-registry",
            1,
            ArtifactFamilyRegistryV1,
            "ArtifactFamilyRegistryV1",
            "P07",
            Some("tests/fixtures/p07/artifact_family_registry_v1.json"),
            "registered-family-additions-only"
        ),
        schema_document!(
            "artifact-envelope",
            1,
            ArtifactEnvelopeV1,
            "ArtifactEnvelopeV1",
            "P30",
            None,
            "aidens-local-envelope-additive-compatible"
        ),
        schema_document!(
            "backfill-receipt",
            1,
            BackfillReportV1,
            "BackfillReportV1",
            "P07",
            Some("tests/fixtures/p07/backfill_receipt_v1.json"),
            "append-only-backfill-evidence"
        ),
        schema_document!(
            "boundary-compile-outcome",
            1,
            BoundaryCompileOutcomeV1,
            "BoundaryCompileOutcomeV1",
            "P06",
            Some("tests/fixtures/p06/boundary_compile_outcome_v1.json"),
            "additive-compatible-with-defaults"
        ),
        schema_document!(
            "boundary-compile-request",
            1,
            BoundaryCompileRequestV1,
            "BoundaryCompileRequestV1",
            "P06",
            Some("tests/fixtures/p06/boundary_compile_request_v1.json"),
            "policy-field-additions-require-defaults"
        ),
        schema_document!(
            "budget-exhaustion-receipt",
            1,
            BudgetExhaustionReportV1,
            "BudgetExhaustionReportV1",
            "P03",
            Some("tests/fixtures/p03/budget_exhaustion_receipt_v1.json"),
            "receipt-major-immutable"
        ),
        schema_document!(
            "display-digest",
            1,
            DisplayDigestV1,
            "DisplayDigestV1",
            "P06",
            Some("tests/fixtures/p06/display_digest_v1.json"),
            "algorithm-change-requires-new-version"
        ),
        schema_document!(
            "capability-gate-decision",
            1,
            CapabilityGateDecisionV1,
            "CapabilityGateDecisionV1",
            "P04",
            Some("tests/fixtures/p04/capability_gate_decision_v1.json"),
            "additive-compatible-with-defaults"
        ),
        schema_document!(
            "config-apply-receipt",
            1,
            ConfigApplyReportV1,
            "ConfigApplyReportV1",
            "P01",
            Some("tests/fixtures/p01/config_apply_receipt_v1.json"),
            "receipt-major-immutable"
        ),
        schema_document!(
            "codex-packet",
            1,
            CodexPacketV1,
            "CodexPacketV1",
            "P10",
            Some("tests/fixtures/p10/codex_packet_v1.json"),
            "handoff-context-major-immutable"
        ),
        schema_document!(
            "command-run-receipt",
            1,
            CommandRunReportV1,
            "CommandRunReportV1",
            "P10",
            Some("tests/fixtures/p10/command_run_receipt_v1.json"),
            "command-evidence-major-immutable"
        ),
        schema_document!(
            "completion-audit-report",
            1,
            CompletionAuditReportV1,
            "CompletionAuditReportV1",
            "P19",
            Some("tests/fixtures/p19/completion_audit_report_v1.json"),
            "completion-state-and-release-bar-major-immutable"
        ),
        schema_document!(
            "example-app-manifest",
            1,
            ExampleAppManifestV1,
            "ExampleAppManifestV1",
            "P14",
            Some("tests/fixtures/p14/example_app_manifest_v1.json"),
            "example-coverage-major-immutable"
        ),
        schema_document!(
            "duplicate-key-finding",
            1,
            DuplicateKeyFindingV1,
            "DuplicateKeyFindingV1",
            "P06",
            Some("tests/fixtures/p06/duplicate_key_finding_v1.json"),
            "finding-major-immutable"
        ),
        schema_document!(
            "cross-pass-traceability-matrix",
            1,
            CrossPassTraceabilityMatrixV1,
            "CrossPassTraceabilityMatrixV1",
            "P19",
            Some("tests/fixtures/p19/cross_pass_traceability_matrix_v1.json"),
            "requirement-evidence-mapping-major-immutable"
        ),
        schema_document!(
            "fake-ready-finding",
            1,
            FakeReadyFindingV1,
            "FakeReadyFindingV1",
            "P00",
            Some("tests/fixtures/p00/fake_ready_finding_v1.json"),
            "finding-major-immutable"
        ),
        schema_document!(
            "generated-schema-manifest",
            1,
            GeneratedSchemaManifestV1,
            "GeneratedSchemaManifestV1",
            "P07",
            Some("tests/fixtures/p07/generated_schema_manifest_v1.json"),
            "generated-output-major-immutable"
        ),
        schema_document!(
            "golden-fixture-manifest",
            1,
            GoldenFixtureManifestV1,
            "GoldenFixtureManifestV1",
            "P08",
            Some("tests/fixtures/reference/golden_fixture_manifest_v1.json"),
            "fixture-coverage-major-immutable"
        ),
        schema_document!(
            "install-smoke-report",
            1,
            InstallSmokeReportV1,
            "InstallSmokeReportV1",
            "P14",
            Some("tests/fixtures/p14/install_smoke_receipt_v1.json"),
            "operator-smoke-evidence-major-immutable"
        ),
        schema_document!(
            "known-limitations-register",
            1,
            KnownLimitationsRegisterV1,
            "KnownLimitationsRegisterV1",
            "P19",
            Some("tests/fixtures/p19/known_limitations_register_v1.json"),
            "limitations-disclosure-major-immutable"
        ),
        schema_document!(
            "job",
            1,
            JobV1,
            "JobV1",
            "P11",
            Some("tests/fixtures/p11/job_v1.json"),
            "job-identity-idempotency-major-immutable"
        ),
        schema_document!(
            "queue-lease",
            1,
            QueueLeaseV1,
            "QueueLeaseV1",
            "P11",
            Some("tests/fixtures/p11/queue_lease_v1.json"),
            "lease-owner-expiry-major-immutable"
        ),
        schema_document!(
            "daemon-namespace",
            1,
            DaemonNamespaceV1,
            "DaemonNamespaceV1",
            "P11",
            Some("tests/fixtures/p11/daemon_namespace_v1.json"),
            "daemon-namespace-owner-major-immutable"
        ),
        schema_document!(
            "duplicate-suppression-receipt",
            1,
            DuplicateSuppressionReportV1,
            "DuplicateSuppressionReportV1",
            "P11",
            Some("tests/fixtures/p11/duplicate_suppression_receipt_v1.json"),
            "idempotency-suppression-major-immutable"
        ),
        schema_document!(
            "migration-plan",
            1,
            MigrationPlanV1,
            "MigrationPlanV1",
            "P07",
            Some("tests/fixtures/p07/migration_plan_v1.json"),
            "phase-law-major-immutable"
        ),
        schema_document!(
            "patch-apply-receipt",
            1,
            PatchApplyReportV1,
            "PatchApplyReportV1",
            "P10",
            Some("tests/fixtures/p10/patch_apply_receipt_v1.json"),
            "write-evidence-major-immutable"
        ),
        schema_document!(
            "operator-status-report",
            1,
            OperatorStatusReportV1,
            "OperatorStatusReportV1",
            "P14",
            Some("tests/fixtures/p14/operator_status_report_v1.json"),
            "operator-diagnostics-additive-compatible"
        ),
        schema_document!(
            "patch-proposal",
            1,
            PatchProposalV1,
            "PatchProposalV1",
            "P10",
            Some("tests/fixtures/p10/patch_proposal_v1.json"),
            "proposal-is-non-mutating"
        ),
        schema_document!(
            "permit-grant",
            1,
            PermitGrantV1,
            "PermitGrantV1",
            "P04",
            Some("tests/fixtures/p04/permit_grant_v1.json"),
            "scope-narrowing-requires-new-version"
        ),
        schema_document!(
            "permit-use-receipt",
            1,
            PermitUseReportV1,
            "PermitUseReportV1",
            "P04",
            Some("tests/fixtures/p04/permit_use_receipt_v1.json"),
            "receipt-major-immutable"
        ),
        schema_document!(
            "plan-runtime-parity-report",
            1,
            PlanRuntimeParityReportV1,
            "PlanRuntimeParityReportV1",
            "P01",
            Some("tests/fixtures/p01/plan_runtime_parity_report_v1.json"),
            "additive-compatible-with-defaults"
        ),
        schema_document!(
            "queue-hop-receipt",
            1,
            QueueHopReportV1,
            "QueueHopReportV1",
            "P11",
            Some("tests/fixtures/p11/queue_hop_receipt_v1.json"),
            "queue-state-transition-major-immutable"
        ),
        schema_document!(
            "provider-backend-matrix",
            1,
            ProviderBackendMatrixV1,
            "ProviderBackendMatrixV1",
            "P02",
            Some("tests/fixtures/p02/provider_backend_matrix_v1.json"),
            "provider-additions-compatible"
        ),
        schema_document!(
            "provider-certification-fixture",
            1,
            ProviderCertificationFixtureV1,
            "ProviderCertificationFixtureV1",
            "P02",
            Some("tests/fixtures/p02/provider_certification_fixture_v1.json"),
            "expectation-meaning-major-immutable"
        ),
        schema_document!(
            "provider-readiness-receipt",
            1,
            ProviderReadinessReportV1,
            "ProviderReadinessReportV1",
            "P02",
            Some("tests/fixtures/p02/provider_readiness_receipt_v1.json"),
            "receipt-major-immutable"
        ),
        schema_document!(
            "provider-route-receipt",
            2,
            ProviderRouteReportV2,
            "ProviderRouteReportV2",
            "P02",
            Some("tests/fixtures/p02/provider_route_receipt_v2.json"),
            "native-capability-major-immutable"
        ),
        schema_document!(
            "release-readiness-report",
            1,
            ReleaseReadinessReportV1,
            "ReleaseReadinessReportV1",
            "P14",
            Some("tests/fixtures/p14/release_readiness_report_v1.json"),
            "release-blocking-semantics-major-immutable"
        ),
        schema_document!(
            "release-artifact-manifest",
            1,
            ReleaseArtifactManifestV1,
            "ReleaseArtifactManifestV1",
            "P19",
            Some("tests/fixtures/p19/release_artifact_manifest_v1.json"),
            "release-package-content-major-immutable"
        ),
        schema_document!(
            "regression-debt-ledger",
            1,
            RegressionDebtLedgerV1,
            "RegressionDebtLedgerV1",
            "P19",
            Some("tests/fixtures/p19/regression_debt_ledger_v1.json"),
            "regression-debt-status-major-immutable"
        ),
        schema_document!(
            "repo-list-receipt",
            1,
            RepoListReportV1,
            "RepoListReportV1",
            "P10",
            Some("tests/fixtures/p10/repo_list_receipt_v1.json"),
            "sandbox-list-evidence-major-immutable"
        ),
        schema_document!(
            "repo-read-receipt",
            1,
            RepoReadReportV1,
            "RepoReadReportV1",
            "P10",
            Some("tests/fixtures/p10/repo_read_receipt_v1.json"),
            "sandbox-read-evidence-major-immutable"
        ),
        schema_document!(
            "run-receipt",
            1,
            RunReportV1,
            "RunReportV1",
            "P05",
            Some("tests/fixtures/p05/run_receipt_v1.json"),
            "receipt-major-immutable"
        ),
        schema_document!(
            "aidens-run-bundle",
            2,
            AiDENsRunBundleV2,
            "AiDENsRunBundleV2",
            "P24",
            Some("tests/fixtures/p24/aidens_run_bundle_v2.json"),
            "operator-bundle-major-immutable"
        ),
        schema_document!(
            "aidens-run-bundle",
            3,
            AiDENsRunBundleV3,
            "AiDENsRunBundleV3",
            "P26",
            Some("tests/fixtures/p26/aidens_run_bundle_v3.json"),
            "operator-bundle-major-immutable"
        ),
        schema_document!(
            "scaffold-surface-report",
            1,
            ScaffoldSurfaceReportV1,
            "ScaffoldSurfaceReportV1",
            "P00",
            Some("tests/fixtures/p00/scaffold_surface_report_v1.json"),
            "status-meaning-major-immutable"
        ),
        schema_document!(
            "schema-compatibility-report",
            1,
            SchemaCompatibilityReportV1,
            "SchemaCompatibilityReportV1",
            "P07",
            Some("tests/fixtures/p07/schema_compatibility_report_v1.json"),
            "report-additions-compatible"
        ),
        schema_document!(
            "sandbox-capability-truth",
            1,
            SandboxCapabilityTruthV1,
            "SandboxCapabilityTruthV1",
            "P10",
            Some("tests/fixtures/p10/sandbox_capability_truth_v1.json"),
            "sandbox-policy-major-immutable"
        ),
        schema_document!(
            "safe-mode-receipt",
            1,
            SafeModeReportV1,
            "SafeModeReportV1",
            "P11",
            Some("tests/fixtures/p11/safe_mode_receipt_v1.json"),
            "safe-mode-law-major-immutable"
        ),
        schema_document!(
            "schedule-occurrence",
            1,
            ScheduleOccurrenceV1,
            "ScheduleOccurrenceV1",
            "P11",
            Some("tests/fixtures/p11/schedule_occurrence_v1.json"),
            "occurrence-idempotency-major-immutable"
        ),
        schema_document!(
            "source-basis-lock",
            1,
            SourceBasisLockV1,
            "SourceBasisLockV1",
            "P00",
            Some("tests/fixtures/p00/source_basis_lock_v1.json"),
            "source-basis-major-immutable"
        ),
        schema_document!(
            "super-pass-status",
            1,
            SuperPassStatusV1,
            "SuperPassStatusV1",
            "P00",
            Some("tests/fixtures/p00/super_pass_status_v1.json"),
            "status-meaning-major-immutable"
        ),
        schema_document!(
            "tool-call-request",
            1,
            ToolCallRequestV1,
            "ToolCallRequestV1",
            "P03",
            Some("tests/fixtures/p03/tool_call_request_v1.json"),
            "parser-fallback-meaning-major-immutable"
        ),
        schema_document!(
            "tool-call-result",
            1,
            ToolCallResultV1,
            "ToolCallResultV1",
            "P03",
            Some("tests/fixtures/p03/tool_call_result_v1.json"),
            "digest-link-major-immutable"
        ),
        schema_document!(
            "tool-exposure-plan",
            2,
            ToolExposurePlanV1,
            "ToolExposurePlanV2",
            "P04",
            Some("tests/fixtures/p04/tool_exposure_plan_v2.json"),
            "lifecycle-meaning-major-immutable"
        ),
        schema_document!(
            "tool-invocation-receipt",
            1,
            ToolInvocationReportV1,
            "ToolInvocationReportV1",
            "P03",
            Some("tests/fixtures/p03/tool_invocation_receipt_v1.json"),
            "receipt-major-immutable"
        ),
        schema_document!(
            "turn-execution-plan",
            1,
            TurnExecutionPlanV1,
            "TurnExecutionPlanV1",
            "P03",
            Some("tests/fixtures/p03/turn_execution_plan_v1.json"),
            "mode-meaning-major-immutable"
        ),
        schema_document!(
            "turn-receipt",
            1,
            TurnReportV1,
            "TurnReportV1",
            "P03",
            Some("tests/fixtures/p03/turn_receipt_v1.json"),
            "final-state-major-immutable"
        ),
        schema_document!(
            "wake-signal",
            1,
            WakeSignalV1,
            "WakeSignalV1",
            "P11",
            Some("tests/fixtures/p11/wake_signal_v1.json"),
            "wake-idempotency-major-immutable"
        ),
    ]
}
