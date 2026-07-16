//! Proof profile, debt, waiver, and promotion-eligibility DTOs.
//!
//! These records expose proof/debt state for AiDENs operators. Waiver remains
//! governance evidence, not proof, and canonical proof authority is delegated.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ProofUseRestrictionV1 {
    MayPromote,
    AdvisoryOnly,
    QuarantineOnly,
    BlockRelease,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProofObligationV1 {
    pub obligation_id: ArtifactId,
    pub description: String,
    pub required_evidence_family: String,
    pub satisfied_by: Vec<ArtifactId>,
    pub waived_by: Vec<ArtifactId>,
}

impl ProofObligationV1 {
    pub fn new(
        description: impl Into<String>,
        required_evidence_family: impl Into<String>,
    ) -> Self {
        let description = description.into();
        let required_evidence_family = required_evidence_family.into();
        Self {
            obligation_id: generated_artifact_id_from_material(
                "proof-obligation",
                &format!("{description}|{required_evidence_family}"),
            ),
            description,
            required_evidence_family,
            satisfied_by: Vec::new(),
            waived_by: Vec::new(),
        }
    }

    pub fn is_proven(&self) -> bool {
        !self.satisfied_by.is_empty()
    }

    pub fn is_waived(&self) -> bool {
        !self.waived_by.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LocalProofProfileV1 {
    pub profile_id: ArtifactId,
    pub proof_class: String,
    pub exactness_tier: String,
    pub obligations: Vec<ProofObligationV1>,
    pub promotion_requires_all_obligations: bool,
    pub waiver_policy: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl LocalProofProfileV1 {
    pub fn local_exact(obligations: Vec<ProofObligationV1>) -> Self {
        let material = obligations
            .iter()
            .map(|obligation| obligation.obligation_id.as_str().to_string())
            .collect::<Vec<_>>()
            .join("|");
        Self {
            profile_id: generated_artifact_id_from_material("proof-profile", &material),
            proof_class: "local-executable-evidence".into(),
            exactness_tier: "exact-check-required".into(),
            obligations,
            promotion_requires_all_obligations: true,
            waiver_policy: "waiver-is-governance-not-proof".into(),
            reason_codes: vec!["proof-profile-declared".into()],
        }
    }

    pub fn proof_satisfied(&self) -> bool {
        self.obligations.iter().all(ProofObligationV1::is_proven)
    }

    pub fn has_waiver_without_proof(&self) -> bool {
        self.obligations
            .iter()
            .any(|obligation| obligation.is_waived() && !obligation.is_proven())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProofDebtItemV1 {
    pub debt_id: ArtifactId,
    pub obligation_id: ArtifactId,
    pub restriction: ProofUseRestrictionV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_uses: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escalated_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escalation_reason: Option<String>,
    pub reason: String,
}

impl ProofDebtItemV1 {
    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn escalated(mut self, escalated_at: DateTime<Utc>, reason: impl Into<String>) -> Self {
        self.escalated_at = Some(escalated_at);
        self.escalation_reason = Some(reason.into());
        self.restriction = ProofUseRestrictionV1::BlockRelease;
        self.allowed_uses = Vec::new();
        self
    }

    pub fn is_expired_at(&self, now: DateTime<Utc>) -> bool {
        self.expires_at
            .map(|expires_at| now >= expires_at)
            .unwrap_or(false)
    }

    pub fn is_escalated_at(&self, now: DateTime<Utc>) -> bool {
        self.escalated_at
            .map(|escalated_at| now >= escalated_at)
            .unwrap_or(false)
            || self.is_expired_at(now)
    }

    pub fn allows_use_at(&self, requested_use: &str, now: DateTime<Utc>) -> bool {
        !self.is_escalated_at(now)
            && matches!(self.restriction, ProofUseRestrictionV1::AdvisoryOnly)
            && self
                .allowed_uses
                .iter()
                .any(|allowed| allowed == requested_use)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProofDebtLedgerV1 {
    pub ledger_id: ArtifactId,
    pub artifact_ref: ArtifactId,
    pub items: Vec<ProofDebtItemV1>,
    pub generated_at: DateTime<Utc>,
}

impl ProofDebtLedgerV1 {
    pub fn from_profile(artifact_ref: ArtifactId, profile: &LocalProofProfileV1) -> Self {
        let items = profile
            .obligations
            .iter()
            .filter(|obligation| !obligation.is_proven())
            .map(|obligation| ProofDebtItemV1 {
                debt_id: generated_artifact_id_from_material(
                    "proof-debt",
                    &format!(
                        "{}|{}",
                        artifact_ref.as_str(),
                        obligation.obligation_id.as_str()
                    ),
                ),
                obligation_id: obligation.obligation_id.clone(),
                restriction: if obligation.is_waived() {
                    ProofUseRestrictionV1::AdvisoryOnly
                } else {
                    ProofUseRestrictionV1::BlockRelease
                },
                allowed_uses: if obligation.is_waived() {
                    vec!["local-advisory-display".into(), "operator-triage".into()]
                } else {
                    Vec::new()
                },
                expires_at: None,
                escalated_at: None,
                escalation_reason: None,
                reason: if obligation.is_waived() {
                    "waiver-recorded-but-proof-missing".into()
                } else {
                    "proof-obligation-unsatisfied".into()
                },
            })
            .collect::<Vec<_>>();
        let material = items
            .iter()
            .map(|item| item.debt_id.as_str().to_string())
            .collect::<Vec<_>>()
            .join("|");
        Self {
            ledger_id: generated_artifact_id_from_material("proof-debt-ledger", &material),
            artifact_ref,
            items,
            generated_at: Utc::now(),
        }
    }

    pub fn blocks_promotion(&self) -> bool {
        self.items.iter().any(|item| {
            matches!(
                item.restriction,
                ProofUseRestrictionV1::BlockRelease
                    | ProofUseRestrictionV1::AdvisoryOnly
                    | ProofUseRestrictionV1::QuarantineOnly
            )
        })
    }

    pub fn active_items_at(&self, now: DateTime<Utc>) -> Vec<&ProofDebtItemV1> {
        self.items
            .iter()
            .filter(|item| !item.is_expired_at(now))
            .collect()
    }

    pub fn expired_items_at(&self, now: DateTime<Utc>) -> Vec<&ProofDebtItemV1> {
        self.items
            .iter()
            .filter(|item| item.is_expired_at(now))
            .collect()
    }

    pub fn escalated_items_at(&self, now: DateTime<Utc>) -> Vec<&ProofDebtItemV1> {
        self.items
            .iter()
            .filter(|item| item.is_escalated_at(now))
            .collect()
    }

    pub fn allows_use_at(&self, requested_use: &str, now: DateTime<Utc>) -> bool {
        !self.items.is_empty()
            && self
                .items
                .iter()
                .all(|item| item.allows_use_at(requested_use, now))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProofWaiverReceiptV1 {
    pub receipt_id: ArtifactId,
    pub obligation_id: ArtifactId,
    pub approver: String,
    pub rationale: String,
    pub waiver_is_not_proof: bool,
    pub recorded_at: DateTime<Utc>,
}

impl ProofWaiverReceiptV1 {
    pub fn new(
        obligation_id: ArtifactId,
        approver: impl Into<String>,
        rationale: impl Into<String>,
    ) -> Self {
        let approver = approver.into();
        let rationale = rationale.into();
        Self {
            receipt_id: generated_artifact_id_from_material(
                "proof-waiver",
                &format!("{}|{}|{}", obligation_id.as_str(), approver, rationale),
            ),
            obligation_id,
            approver,
            rationale,
            waiver_is_not_proof: true,
            recorded_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PromotionEligibilityReportV1 {
    pub report_id: ArtifactId,
    pub artifact_ref: ArtifactId,
    pub proof_profile_id: ArtifactId,
    pub proof_debt_ledger_id: ArtifactId,
    pub eligible: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl PromotionEligibilityReportV1 {
    pub fn new(
        artifact_ref: ArtifactId,
        profile: &LocalProofProfileV1,
        debt: &ProofDebtLedgerV1,
    ) -> Self {
        let eligible = profile.proof_satisfied() && !debt.blocks_promotion();
        Self {
            report_id: generated_artifact_id_from_material(
                "promotion-eligibility",
                &format!(
                    "{}|{}|{}",
                    artifact_ref.as_str(),
                    profile.profile_id.as_str(),
                    debt.ledger_id.as_str()
                ),
            ),
            artifact_ref,
            proof_profile_id: profile.profile_id.clone(),
            proof_debt_ledger_id: debt.ledger_id.clone(),
            eligible,
            reason_codes: if eligible {
                vec!["promotion-proof-satisfied".into()]
            } else {
                vec!["promotion-blocked-by-proof-debt".into()]
            },
            generated_at: Utc::now(),
        }
    }
}
