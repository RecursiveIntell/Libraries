use crate::evidence::bundle_for_claim;
use crate::model::{AgentClaim, ClaimStatus, EvidenceItem};
use chrono::Utc;
use claim_ledger::{Claim, SupportJudgment, SupportState};

pub struct AdjudicatedClaim {
    pub claim: Claim,
    pub judgment: SupportJudgment,
    pub bundle: claim_ledger::EvidenceBundle,
}

pub fn support_state(status: &ClaimStatus, has_evidence: bool) -> SupportState {
    match status {
        ClaimStatus::Verified => SupportState::Supported,
        ClaimStatus::Partial => SupportState::PartiallySupported,
        ClaimStatus::Unsupported => SupportState::Unsupported,
        ClaimStatus::Contradicted => SupportState::Contradicted,
        ClaimStatus::NotChecked if has_evidence => SupportState::HeuristicOnly,
        ClaimStatus::NotChecked => SupportState::Unknown,
    }
}

pub fn adjudicate(claim: &AgentClaim, evidence: &[EvidenceItem]) -> AdjudicatedClaim {
    let bundle = bundle_for_claim(claim, evidence);
    let state = support_state(&claim.status, !evidence.is_empty());
    let mut mapped = Claim::new(
        "agent-evidence-workbench",
        &claim.id,
        &claim.text,
        &claim.normalized_predicate,
    );
    mapped.claim_id = claim.id.clone();
    mapped.source_id = "agent-evidence-workbench".into();
    mapped.span_id = claim
        .source_location
        .clone()
        .unwrap_or_else(|| claim.id.clone());
    mapped.support_judgment_ref = format!("sj_{}", claim.id);
    mapped.evidence_bundle_ref = bundle.evidence_bundle_id.clone();
    mapped.status = "adjudicated".into();
    mapped.confidence = match state {
        SupportState::Supported => 1.0,
        SupportState::PartiallySupported => 0.5,
        SupportState::HeuristicOnly => 0.25,
        _ => 0.0,
    };
    let judgment = SupportJudgment {
        support_judgment_id: format!("sj_{}", claim.id),
        claim_id: claim.id.clone(),
        evidence_bundle_ref: bundle.evidence_bundle_id.clone(),
        support_state: state,
        method: "aew_adjudicator".into(),
        rationale: claim.source_quote.clone(),
        contradiction_refs: Vec::new(),
        proof_debt: Vec::new(),
        created_recorded_time: Utc::now(),
    };
    AdjudicatedClaim {
        claim: mapped,
        judgment,
        bundle,
    }
}
