//! AiDENs-local artifact envelope, manifest, and lifecycle DTOs.
//!
//! These types describe local operator artifacts and carry canonical-owner
//! backpointers. They do not create AiDENs-owned domain truth.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactLifecycleStateV1 {
    Created,
    Validated,
    Admitted,
    Projected,
    Proposed,
    Verified,
    Refuted,
    Contradicted,
    Quarantined,
    Promoted,
    Superseded,
    Retired,
    Subtracted,
    Disputed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactAuthorityClassV1 {
    CanonicalOwnerReference,
    AiDENsExecutionAuthoritative,
    AiDENsDisplayOnly,
    AdmittedFacade,
    FixtureBacked,
    ExternalQuarantine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactEnvelopeV1 {
    pub artifact_ref: ArtifactId,
    pub family: String,
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_digest: Option<DisplayDigestV1>,
    pub issuer: String,
    pub created_recorded_time: DateTime<Utc>,
    pub namespace: String,
    pub authority_class: ArtifactAuthorityClassV1,
    pub lifecycle_state: ArtifactLifecycleStateV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitemporal_applicability: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signature_or_attestation_refs: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ArtifactEnvelopeV1 {
    pub fn from_json(
        family: impl Into<String>,
        schema_version: u32,
        value: &serde_json::Value,
        authority_class: ArtifactAuthorityClassV1,
        namespace: impl Into<String>,
        issuer: impl Into<String>,
    ) -> Self {
        let family = family.into();
        let namespace = namespace.into();
        let issuer = issuer.into();
        let content_digest = DisplayDigestV1::for_json_value(value);
        let material = format!(
            "{}|{}|{}|{}|{}",
            family, schema_version, namespace, issuer, content_digest.digest
        );
        Self {
            artifact_ref: generated_artifact_id_from_material("artifact", &material),
            family,
            schema_version,
            content_digest: Some(content_digest),
            issuer,
            created_recorded_time: Utc::now(),
            namespace,
            authority_class,
            lifecycle_state: ArtifactLifecycleStateV1::Created,
            bitemporal_applicability: None,
            signature_or_attestation_refs: Vec::new(),
            canonical_backpointers: Vec::new(),
            reason_codes: vec!["artifact-envelope-created".into()],
        }
    }

    pub fn with_canonical_backpointer(mut self, backpointer: CanonicalBackpointerV1) -> Self {
        self.canonical_backpointers.push(backpointer);
        self.reason_codes
            .push("canonical-owner-backpointer-declared".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn is_promotable(&self) -> bool {
        self.lifecycle_state == ArtifactLifecycleStateV1::Verified
            && !matches!(
                self.authority_class,
                ArtifactAuthorityClassV1::AiDENsDisplayOnly
                    | ArtifactAuthorityClassV1::ExternalQuarantine
            )
    }

    pub fn apply_transition(
        &mut self,
        new_state: ArtifactLifecycleStateV1,
        triggering_operator: impl Into<String>,
        actor: impl Into<String>,
        execution_context_ref: Option<ArtifactId>,
    ) -> Result<ArtifactTransitionReceiptV1, String> {
        let previous_state = self.lifecycle_state;
        let receipt = ArtifactTransitionReceiptV1::new(
            self.artifact_ref.clone(),
            previous_state,
            new_state,
            triggering_operator,
            actor,
            execution_context_ref,
        )?;
        self.lifecycle_state = new_state;
        self.reason_codes.push(format!(
            "artifact-transition:{}-to-{}",
            previous_state.as_str(),
            new_state.as_str()
        ));
        self.reason_codes.sort();
        self.reason_codes.dedup();
        Ok(receipt)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactManifestEntryV1 {
    pub artifact_ref: ArtifactId,
    pub family: String,
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_digest: Option<DisplayDigestV1>,
    pub lifecycle_state: ArtifactLifecycleStateV1,
}

impl From<&ArtifactEnvelopeV1> for ArtifactManifestEntryV1 {
    fn from(envelope: &ArtifactEnvelopeV1) -> Self {
        Self {
            artifact_ref: envelope.artifact_ref.clone(),
            family: envelope.family.clone(),
            schema_version: envelope.schema_version,
            content_digest: envelope.content_digest.clone(),
            lifecycle_state: envelope.lifecycle_state,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactManifestV1 {
    pub manifest_id: ArtifactId,
    pub inputs: Vec<ArtifactManifestEntryV1>,
    pub outputs: Vec<ArtifactManifestEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_or_opaque_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_or_opaque_ref_records: Vec<MissingOrOpaqueRefRecordV1>,
    pub canonicalization_profile: String,
    pub schema_identities: BTreeMap<String, String>,
    pub created_recorded_time: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ArtifactManifestV1 {
    pub fn new(
        inputs: Vec<ArtifactManifestEntryV1>,
        outputs: Vec<ArtifactManifestEntryV1>,
        canonicalization_profile: impl Into<String>,
        schema_identities: BTreeMap<String, String>,
    ) -> Self {
        let canonicalization_profile = canonicalization_profile.into();
        let material = serde_json::json!({
            "inputs": inputs,
            "outputs": outputs,
            "canonicalization_profile": canonicalization_profile,
            "schema_identities": schema_identities,
        });
        Self {
            manifest_id: generated_artifact_id_from_material(
                "artifact-manifest",
                &non_authoritative_json_display_digest(&material),
            ),
            inputs,
            outputs,
            missing_or_opaque_refs: Vec::new(),
            missing_or_opaque_ref_records: Vec::new(),
            canonicalization_profile,
            schema_identities,
            created_recorded_time: Utc::now(),
            reason_codes: vec!["artifact-manifest-created".into()],
        }
    }

    pub fn complete(&self) -> bool {
        self.missing_or_opaque_refs.is_empty() && self.missing_or_opaque_ref_records.is_empty()
    }

    pub fn with_missing_or_opaque_ref(
        mut self,
        ref_id: impl Into<String>,
        reason: impl Into<String>,
        degradation_record_id: ArtifactId,
    ) -> Self {
        let record =
            MissingOrOpaqueRefRecordV1::new(ref_id.into(), reason.into(), degradation_record_id);
        self.missing_or_opaque_refs.push(record.ref_id.clone());
        self.missing_or_opaque_refs.sort();
        self.missing_or_opaque_refs.dedup();
        self.missing_or_opaque_ref_records.push(record);
        self.reason_codes
            .push("manifest-has-missing-or-opaque-ref".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MissingOrOpaqueRefRecordV1 {
    pub record_id: ArtifactId,
    pub ref_id: String,
    pub reason: String,
    pub degradation_record_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl MissingOrOpaqueRefRecordV1 {
    pub fn new(ref_id: String, reason: String, degradation_record_id: ArtifactId) -> Self {
        Self {
            record_id: generated_artifact_id_from_material(
                "missing-opaque-ref",
                &format!("{ref_id}|{reason}|{}", degradation_record_id.0),
            ),
            ref_id,
            reason,
            degradation_record_id,
            reason_codes: vec!["missing-or-opaque-ref-degradation-linked".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactTransitionReceiptV1 {
    pub receipt_id: ArtifactId,
    pub artifact_ref: ArtifactId,
    pub previous_state: ArtifactLifecycleStateV1,
    pub new_state: ArtifactLifecycleStateV1,
    pub triggering_operator: String,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_context_ref: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_refs: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degradation_refs: Vec<ArtifactId>,
    pub recorded_time: DateTime<Utc>,
    pub allowed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
}

impl ArtifactTransitionReceiptV1 {
    pub fn new(
        artifact_ref: ArtifactId,
        previous_state: ArtifactLifecycleStateV1,
        new_state: ArtifactLifecycleStateV1,
        triggering_operator: impl Into<String>,
        actor: impl Into<String>,
        execution_context_ref: Option<ArtifactId>,
    ) -> Result<Self, String> {
        if !artifact_transition_allowed(previous_state, new_state) {
            return Err(format!(
                "artifact lifecycle transition rejected: {} -> {}",
                previous_state.as_str(),
                new_state.as_str()
            ));
        }
        if new_state == ArtifactLifecycleStateV1::Promoted
            && previous_state != ArtifactLifecycleStateV1::Verified
        {
            return Err(format!(
                "artifact promotion rejected: previous state {} is not verified",
                previous_state.as_str()
            ));
        }
        let triggering_operator = triggering_operator.into();
        let actor = actor.into();
        let material = format!(
            "{}|{}|{}|{}|{}",
            artifact_ref.0,
            previous_state.as_str(),
            new_state.as_str(),
            triggering_operator,
            actor
        );
        Ok(Self {
            receipt_id: generated_artifact_id_from_material("artifact-transition", &material),
            artifact_ref,
            previous_state,
            new_state,
            triggering_operator,
            actor,
            execution_context_ref,
            proof_refs: Vec::new(),
            degradation_refs: Vec::new(),
            recorded_time: Utc::now(),
            allowed: true,
            reason_codes: vec!["artifact-transition-allowed".into()],
            canonical_backpointers: canonical_owner_backpointer(
                "verification-control",
                "ControlReceipt",
                "canonical-lifecycle-transition-owner",
            ),
        })
    }
}

impl ArtifactLifecycleStateV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Admitted => "admitted",
            Self::Projected => "projected",
            Self::Proposed => "proposed",
            Self::Verified => "verified",
            Self::Refuted => "refuted",
            Self::Contradicted => "contradicted",
            Self::Quarantined => "quarantined",
            Self::Promoted => "promoted",
            Self::Superseded => "superseded",
            Self::Retired => "retired",
            Self::Subtracted => "subtracted",
            Self::Disputed => "disputed",
        }
    }
}

pub fn artifact_transition_allowed(
    previous_state: ArtifactLifecycleStateV1,
    new_state: ArtifactLifecycleStateV1,
) -> bool {
    use ArtifactLifecycleStateV1::*;
    match previous_state {
        Created => matches!(new_state, Validated | Quarantined),
        Validated => matches!(new_state, Admitted | Proposed | Quarantined | Refuted),
        Admitted => matches!(
            new_state,
            Projected | Proposed | Verified | Quarantined | Disputed
        ),
        Projected => matches!(new_state, Verified | Superseded | Retired | Quarantined),
        Proposed => matches!(
            new_state,
            Verified | Refuted | Quarantined | Superseded | Disputed
        ),
        Verified => matches!(
            new_state,
            Promoted | Contradicted | Superseded | Retired | Disputed
        ),
        Refuted => matches!(new_state, Quarantined | Superseded | Retired),
        Contradicted => matches!(new_state, Quarantined | Verified | Superseded | Disputed),
        Quarantined => matches!(
            new_state,
            Validated | Admitted | Refuted | Superseded | Retired
        ),
        Promoted => matches!(new_state, Superseded | Retired | Contradicted | Disputed),
        Superseded => matches!(new_state, Retired),
        Retired => matches!(new_state, Disputed),
        Subtracted => false,
        Disputed => matches!(
            new_state,
            Quarantined | Verified | Promoted | Superseded | Retired
        ),
    }
}
