//! Final response shaping after agency policy checks.
//!
//! This module only prepares local final text and disclosure strings. Agency
//! policy truth remains in `aidens-agency-kit`.

use super::*;

pub(super) fn final_text_after_agency_gate(
    candidate: &str,
    report: &AgencyPolicyReportV1,
) -> String {
    match report.outcome {
        AgencyPolicyOutcomeV1::Allow => candidate.to_string(),
        AgencyPolicyOutcomeV1::AllowWithDisclosure => {
            format!("{}\n\n{candidate}", report.disclosure_text())
        }
        outcome => final_text_for_agency_block(outcome),
    }
}

pub(super) fn final_text_for_agency_block(outcome: AgencyPolicyOutcomeV1) -> String {
    match outcome {
        AgencyPolicyOutcomeV1::RequireAlternatives => {
            "Turn stopped: agency policy requires viable alternatives and tradeoff evidence before this advice can be returned.".into()
        }
        AgencyPolicyOutcomeV1::RequireUserConfirmation => {
            "Turn stopped: agency policy requires user confirmation before repeating this recommendation.".into()
        }
        AgencyPolicyOutcomeV1::DeferToProfessionalOrExternalSource => {
            "Turn stopped: agency policy requires external or professional review before this recommendation.".into()
        }
        AgencyPolicyOutcomeV1::Block | AgencyPolicyOutcomeV1::Quarantine => {
            "Turn stopped: agency policy blocked this influence pattern.".into()
        }
        AgencyPolicyOutcomeV1::Allow | AgencyPolicyOutcomeV1::AllowWithDisclosure => String::new(),
    }
}
