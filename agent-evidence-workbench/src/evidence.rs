use crate::model::{AgentClaim, EvidenceItem};
use claim_ledger::{EvidenceBundle, EvidenceLink, EvidenceRelation};

/// Build a claim-ledger evidence bundle from the workbench claim and manifest items.
pub fn bundle_for_claim(claim: &AgentClaim, items: &[EvidenceItem]) -> EvidenceBundle {
    let mut bundle = EvidenceBundle::new(&claim.id);
    bundle.evidence_links = items
        .iter()
        .map(|item| EvidenceLink {
            relation: EvidenceRelation::Mentions,
            source_id: item.source.clone(),
            span_id: claim
                .source_location
                .clone()
                .unwrap_or_else(|| item.id.clone()),
            quote: claim.source_quote.clone(),
            digest: item.digest.clone(),
            support_role: format!("{:?}", item.kind),
        })
        .collect();
    bundle
}

pub fn bundles_for_claims(claims: &[AgentClaim], items: &[EvidenceItem]) -> Vec<EvidenceBundle> {
    claims
        .iter()
        .map(|claim| bundle_for_claim(claim, items))
        .collect()
}
