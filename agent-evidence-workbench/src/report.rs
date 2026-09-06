use crate::model::*;
use crate::{
    error::{Error, Result},
    v2::{
        CommandEvidenceV2, ReleaseTruthInputV2, ReleaseTruthReportV2, RunEventV2, SourceBindingV2,
    },
};
use serde::Serialize;
use std::collections::BTreeMap;

const REVIEW_PACKET_V2: &str = "aew.v2-review-packet.v1";
const NON_AUTHORITY_BOUNDARY: &str = "This local projection does not make a terminal release decision, provide independent review, attest evidence, or authorize release.";

#[derive(Serialize)]
struct ReviewPacketV2 {
    schema_version: &'static str,
    event: ReviewEventIdentityV2,
    canonical_digest: String,
    source_binding: SourceBindingV2,
    claims: Vec<ReviewClaimV2>,
    commands: Vec<ReviewCommandV2>,
    human_reviewer_decision: Option<String>,
    terminal_release_decision: Option<String>,
    non_authority_boundary: &'static str,
}

#[derive(Serialize)]
struct ReviewEventIdentityV2 {
    event_id: String,
    kind: String,
    observed_at: String,
    recorded_at: String,
}

#[derive(Serialize)]
struct ReviewClaimV2 {
    id: String,
    assertion: String,
    required_evidence: Vec<String>,
    support_state: claim_ledger::SupportState,
    proof_debt: Vec<claim_ledger::ProofDebt>,
    rationale: String,
}

#[derive(Serialize)]
struct ReviewCommandV2 {
    id: String,
    execution_mode: String,
    argv: Vec<String>,
    cwd: String,
    outcome: crate::v2::CommandOutcomeV2,
}

fn recorded_content(event: &RunEventV2) -> Result<(ReleaseTruthInputV2, ReleaseTruthReportV2)> {
    let content = event
        .payload
        .get("content")
        .ok_or_else(|| Error::Invalid("missing V2 event content".into()))?;
    Ok((
        serde_json::from_value(
            content
                .get("input")
                .cloned()
                .ok_or_else(|| Error::Invalid("missing V2 event input".into()))?,
        )?,
        serde_json::from_value(
            content
                .get("report")
                .cloned()
                .ok_or_else(|| Error::Invalid("missing V2 event report".into()))?,
        )?,
    ))
}

fn review_packet_v2(event: &RunEventV2) -> Result<ReviewPacketV2> {
    let (input, report) = recorded_content(event)?;
    let evaluations = report
        .claims
        .iter()
        .map(|claim| (claim.claim_id.as_str(), claim))
        .collect::<BTreeMap<_, _>>();
    if evaluations.len() != report.claims.len() {
        return Err(Error::Invalid("duplicate V2 report claim ID".into()));
    }
    let claims = input
        .claims
        .iter()
        .map(|claim| {
            let evaluation = evaluations
                .get(claim.id.as_str())
                .ok_or_else(|| Error::Invalid("missing V2 claim evaluation".into()))?;
            Ok(ReviewClaimV2 {
                id: claim.id.clone(),
                assertion: claim.text.clone(),
                required_evidence: claim.required_evidence.clone(),
                support_state: evaluation.support_state,
                proof_debt: evaluation.proof_debt.clone(),
                rationale: evaluation.rationale.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if claims.len() != report.claims.len() {
        return Err(Error::Invalid(
            "V2 report has unknown claim evaluation".into(),
        ));
    }
    Ok(ReviewPacketV2 {
        schema_version: REVIEW_PACKET_V2,
        event: ReviewEventIdentityV2 {
            event_id: event.event_id.clone(),
            kind: event.kind.clone(),
            observed_at: event.observed_at.clone(),
            recorded_at: event.recorded_at.clone(),
        },
        canonical_digest: report.canonical_digest,
        source_binding: report.source_binding,
        claims,
        commands: input.commands.iter().map(review_command_v2).collect(),
        human_reviewer_decision: None,
        terminal_release_decision: None,
        non_authority_boundary: NON_AUTHORITY_BOUNDARY,
    })
}

fn review_command_v2(command: &CommandEvidenceV2) -> ReviewCommandV2 {
    ReviewCommandV2 {
        id: command.id.clone(),
        execution_mode: command.execution_mode.clone(),
        argv: command.argv.clone(),
        cwd: command.cwd.clone(),
        outcome: command.outcome.clone(),
    }
}

/// Serializes the deterministic, local-only V2 review projection.
pub fn generate_v2_review_json(event: &RunEventV2) -> Result<String> {
    Ok(serde_json::to_string_pretty(&review_packet_v2(event)?)?)
}

/// Renders the deterministic Markdown form of the local V2 review projection.
pub fn generate_v2_review_markdown(event: &RunEventV2) -> Result<String> {
    let packet = review_packet_v2(event)?;
    let mut output = format!(
        "# AEW V2 Review Packet\n\nEvent: `{}` (`{}`)\n\nCanonical digest: `{}`\n\nHuman reviewer decision: `null`\n\nTerminal release decision: `null`\n\n{}\n\n## Claims\n\n| Assertion | Support | Proof debt | Rationale |\n|---|---|---|---|\n",
        packet.event.event_id,
        packet.event.kind,
        packet.canonical_digest,
        packet.non_authority_boundary,
    );
    for claim in packet.claims {
        output.push_str(&format!(
            "| {} | {:?} | {:?} | {} |\n",
            claim.assertion, claim.support_state, claim.proof_debt, claim.rationale
        ));
    }
    output.push_str("\n## Source binding\n\n```json\n");
    output.push_str(&serde_json::to_string_pretty(&packet.source_binding)?);
    output.push_str("\n```\n\n## Commands\n\n");
    for command in packet.commands {
        output.push_str(&format!(
            "- `{}`: `{}` ({:?})\n",
            command.id,
            command.argv.join(" "),
            command.outcome
        ));
    }
    Ok(output)
}

pub fn generate_markdown(r: &RunReport) -> String {
    let mut s = format!(
        "# Agent Evidence Report\n\n**Run:** `{}`\n\n**Verdict:** **{:?}**\n\n## Claims\n\n| Claim | Status |\n|---|---|\n",
        r.run_id, r.verdict
    );
    for c in &r.claims {
        s.push_str(&format!(
            "| {} | {:?} |\n",
            c.text,
            crate::adjudicator::support_state(
                &c.status,
                r.evidence_manifest.iter().any(|e| c
                    .source_location
                    .as_deref()
                    .is_some_and(|s| s.contains(&e.id) || s.contains(&e.source)))
            )
        ));
    }
    s.push_str("\n## Checks\n\n| Command | Exit | Passed |\n|---|---:|---|\n");
    for c in &r.checks {
        s.push_str(&format!(
            "| `{}` | {:?} | {} |\n",
            c.command, c.exit_code, c.passed
        ));
    }
    let manifest =
        serde_json::to_string_pretty(&r.evidence_manifest).unwrap_or_else(|_| "[]".into());
    s.push_str(&format!(
        "\n## Diff\n\n```diff\n{}\n```\n\n## Evidence manifest\n\n{}\n",
        r.diff, manifest
    ));
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renders_status() {
        let r = RunReport {
            run_id: "x".into(),
            verdict: RunVerdict::Partial,
            claims: vec![],
            checks: vec![],
            diff: String::new(),
            evidence_manifest: vec![],
        };
        assert!(generate_markdown(&r).contains("Partial"));
    }
}
