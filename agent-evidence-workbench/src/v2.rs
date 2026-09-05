//! Explicit, deterministic claim-to-evidence evaluation for the Release Truth Gate.
//!
//! This module intentionally emits an AEW provisional policy result. It uses the
//! ClaimLedger support vocabulary but does not claim ClaimLedger admission until a
//! canonical admission API is available and invoked by an adapter.

use crate::error::{Error, Result};
use claim_ledger::{ProofDebt, SupportState};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const RELEASE_TRUTH_INPUT_V2: &str = "aew.release-truth-input.v2";
pub const RELEASE_TRUTH_REPORT_V2: &str = "aew.release-truth-report.v2";
pub const PROVISIONAL_POLICY_METHOD: &str = "aew_provisional_deterministic_policy_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandOutcomeV2 {
    Passed,
    Failed,
    Skipped,
    Blocked,
    TimedOut,
    Error,
}

impl CommandOutcomeV2 {
    fn supports_claim(&self) -> bool {
        matches!(self, Self::Passed)
    }

    fn unavailable(&self) -> bool {
        matches!(
            self,
            Self::Skipped | Self::Blocked | Self::TimedOut | Self::Error
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelationV2 {
    Supports,
    PartiallySupports,
    Contradicts,
    Mentions,
    SourceSpanAnchorOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplicitClaimV2 {
    pub id: String,
    pub text: String,
    /// Exact command/evidence IDs required for this claim to receive support.
    pub required_evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandEvidenceV2 {
    pub id: String,
    pub execution_mode: String,
    pub argv: Vec<String>,
    pub cwd: String,
    pub outcome: CommandOutcomeV2,
    /// Inputs must already be redacted before durable persistence.
    pub stdout: String,
    /// Inputs must already be redacted before durable persistence.
    pub stderr: String,
    /// When the command was observed to start/end, as an immutable input.
    pub observed_at: String,
    /// When this receipt was recorded into the run, as an immutable input.
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimEvidenceLinkV2 {
    pub claim_id: String,
    pub evidence_id: String,
    pub relation: EvidenceRelationV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseTruthInputV2 {
    pub schema_version: String,
    pub run_id: String,
    pub claims: Vec<ExplicitClaimV2>,
    pub commands: Vec<CommandEvidenceV2>,
    pub links: Vec<ClaimEvidenceLinkV2>,
    #[serde(default)]
    pub source_binding: Option<SourceBindingV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimEvaluationV2 {
    pub claim_id: String,
    pub support_state: SupportState,
    pub command_outcomes: Vec<CommandOutcomeV2>,
    pub proof_debt: Vec<ProofDebt>,
    pub rationale: String,
    pub adjudication_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseTruthReportV2 {
    pub schema_version: String,
    pub run_id: String,
    pub claims: Vec<ClaimEvaluationV2>,
    /// Digest over the complete sanitized input, source binding, and policy output.
    /// Observation timestamps are immutable identity inputs, so idempotence applies
    /// to exact replay rather than semantically similar later observations.
    pub canonical_digest: String,
    pub terminal_release_decision: Option<String>,
    pub source_binding: SourceBindingV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceSnapshotV2 {
    pub repository_path: String,
    pub head: String,
    pub tree: String,
    pub status: String,
    pub is_clean: bool,
    pub diff_digest: String,
    pub workspace_content_digest: String,
    pub observed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceBindingV2 {
    pub pre: SourceSnapshotV2,
    pub post: SourceSnapshotV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunEventV2 {
    pub schema_version: String,
    pub event_id: String,
    pub kind: String,
    pub payload: serde_json::Value,
    pub observed_at: String,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedactionResultV2 {
    pub text: String,
    pub redaction_count: usize,
}

fn unique_ids(kind: &str, ids: impl Iterator<Item = String>) -> Result<()> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if id.trim().is_empty() {
            return Err(Error::Invalid(format!("{kind} id must not be empty")));
        }
        if !seen.insert(id.clone()) {
            return Err(Error::Invalid(format!("duplicate {kind} id: {id}")));
        }
    }
    Ok(())
}

fn canonical_digest(input: &ReleaseTruthInputV2, report: &ReleaseTruthReportV2) -> Result<String> {
    let canonical = serde_json::json!({
        "schema_version": report.schema_version,
        "run_id": report.run_id,
        "claims": report.claims,
        "terminal_release_decision": report.terminal_release_decision,
        "source_binding": report.source_binding,
        "input": input,
    });
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

/// Evaluates only explicit requirements and explicit claim/evidence links.
/// It never treats a command string or arbitrary output as supporting evidence.
pub fn evaluate(input: &ReleaseTruthInputV2) -> Result<ReleaseTruthReportV2> {
    if input.schema_version != RELEASE_TRUTH_INPUT_V2 {
        return Err(Error::Invalid(format!(
            "unsupported release-truth input schema: {}",
            input.schema_version
        )));
    }
    if input.run_id.trim().is_empty() {
        return Err(Error::Invalid("run id must not be empty".into()));
    }
    let source_binding = input
        .source_binding
        .clone()
        .ok_or_else(|| Error::Invalid("V2 source_binding is required".into()))?;
    let mut pre_state = source_binding.pre.clone();
    let mut post_state = source_binding.post.clone();
    pre_state.observed_at.clear();
    post_state.observed_at.clear();
    if pre_state != post_state {
        return Err(Error::Invalid(
            "source binding changed during evaluation".into(),
        ));
    }
    if source_binding.pre.repository_path.trim().is_empty()
        || source_binding.pre.head.trim().is_empty()
        || source_binding.pre.tree.trim().is_empty()
        || source_binding
            .pre
            .workspace_content_digest
            .trim()
            .is_empty()
        || source_binding.pre.observed_at.trim().is_empty()
        || source_binding.post.observed_at.trim().is_empty()
    {
        return Err(Error::Invalid("malformed source binding".into()));
    }
    unique_ids("claim", input.claims.iter().map(|claim| claim.id.clone()))?;
    unique_ids(
        "command evidence",
        input.commands.iter().map(|command| command.id.clone()),
    )?;

    let commands: BTreeMap<&str, &CommandEvidenceV2> = input
        .commands
        .iter()
        .map(|command| (command.id.as_str(), command))
        .collect();
    let claims: BTreeSet<&str> = input.claims.iter().map(|claim| claim.id.as_str()).collect();
    let mut supports: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut partial_supports: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut contradictions: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();

    for link in &input.links {
        if !claims.contains(link.claim_id.as_str()) {
            return Err(Error::Invalid(format!(
                "link references unknown claim: {}",
                link.claim_id
            )));
        }
        if !commands.contains_key(link.evidence_id.as_str()) {
            return Err(Error::Invalid(format!(
                "link references unknown evidence: {}",
                link.evidence_id
            )));
        }
        match link.relation {
            EvidenceRelationV2::Supports => {
                supports
                    .entry(link.claim_id.as_str())
                    .or_default()
                    .insert(link.evidence_id.as_str());
            }
            EvidenceRelationV2::PartiallySupports => {
                partial_supports
                    .entry(link.claim_id.as_str())
                    .or_default()
                    .insert(link.evidence_id.as_str());
            }
            EvidenceRelationV2::Contradicts => {
                contradictions
                    .entry(link.claim_id.as_str())
                    .or_default()
                    .insert(link.evidence_id.as_str());
            }
            EvidenceRelationV2::Mentions | EvidenceRelationV2::SourceSpanAnchorOnly => {}
        }
    }

    let mut evaluations = Vec::with_capacity(input.claims.len());
    for claim in &input.claims {
        let required = &claim.required_evidence;
        if required.is_empty() {
            return Err(Error::Invalid(format!(
                "claim {} has no declared required evidence",
                claim.id
            )));
        }
        unique_ids("required evidence", required.iter().cloned())?;
        let mut outcomes = Vec::with_capacity(required.len());
        let mut missing = Vec::new();
        let mut unavailable = false;
        let mut failed = false;
        let mut all_linked = true;
        let mut any_partial = false;
        for evidence_id in required {
            let Some(command) = commands.get(evidence_id.as_str()) else {
                missing.push(evidence_id.clone());
                all_linked = false;
                continue;
            };
            outcomes.push(command.outcome.clone());
            if command.outcome.unavailable() {
                unavailable = true;
            }
            if !command.outcome.supports_claim() {
                failed = true;
            }
            if !supports
                .get(claim.id.as_str())
                .is_some_and(|linked| linked.contains(evidence_id.as_str()))
            {
                all_linked = false;
                if partial_supports
                    .get(claim.id.as_str())
                    .is_some_and(|linked| linked.contains(evidence_id.as_str()))
                {
                    any_partial = true;
                }
            }
        }
        let contradicted = contradictions
            .get(claim.id.as_str())
            .is_some_and(|evidence_ids| {
                evidence_ids.iter().any(|id| {
                    commands
                        .get(id)
                        .is_some_and(|command| command.outcome.supports_claim())
                })
            });
        let (support_state, proof_debt, rationale) = if contradicted {
            (
                SupportState::Contradicted,
                vec![ProofDebt::MissingRepro],
                "an explicit contradictory evidence link has passing evidence".to_string(),
            )
        } else if unavailable {
            (
                SupportState::Unknown,
                vec![ProofDebt::MissingRepro],
                "required evidence is unavailable, skipped, blocked, timed out, or errored"
                    .to_string(),
            )
        } else if !missing.is_empty() {
            (
                SupportState::Unsupported,
                vec![ProofDebt::MissingSourceBasis],
                format!("required evidence is absent: {}", missing.join(", ")),
            )
        } else if failed || (!all_linked && !any_partial) {
            (
                SupportState::Unsupported,
                vec![ProofDebt::MissingRepro],
                "required evidence did not pass with an explicit Supports relation".to_string(),
            )
        } else if !all_linked {
            (
                SupportState::PartiallySupported,
                vec![ProofDebt::MissingRepro],
                "all required evidence passed but at least one requirement has only an explicit PartiallySupports relation".to_string(),
            )
        } else {
            (
                SupportState::Supported,
                vec![ProofDebt::None],
                "all declared evidence passed and each requirement has an explicit Supports relation".to_string(),
            )
        };
        evaluations.push(ClaimEvaluationV2 {
            claim_id: claim.id.clone(),
            support_state,
            command_outcomes: outcomes,
            proof_debt,
            rationale,
            adjudication_method: PROVISIONAL_POLICY_METHOD.into(),
        });
    }

    let mut report = ReleaseTruthReportV2 {
        schema_version: RELEASE_TRUTH_REPORT_V2.into(),
        run_id: input.run_id.clone(),
        claims: evaluations,
        canonical_digest: String::new(),
        // Only verification-control can construct a terminal release decision.
        terminal_release_decision: None,
        source_binding,
    };
    report.canonical_digest = canonical_digest(input, &report)?;
    Ok(report)
}

/// Redacts common credential-shaped values before content reaches durable storage.
/// This is a bounded local baseline, not a universal secret-detection guarantee.
pub fn redact_text(input: &str) -> RedactionResultV2 {
    let patterns = [
        r"(?i)bearer\s+[a-z0-9._-]+",
        r#"(?i)(api[_-]?key\s*[=:]\s*)[^\s\"']+"#,
        r"\bsk-[A-Za-z0-9_-]+",
        r"-----BEGIN(?: [A-Z]+)? PRIVATE KEY-----[\s\S]*?-----END(?: [A-Z]+)? PRIVATE KEY-----",
    ];
    let mut text = input.to_string();
    let mut redaction_count = 0;
    for pattern in patterns {
        let regex = Regex::new(pattern).expect("static redaction regex");
        text = regex
            .replace_all(&text, |_captures: &regex::Captures<'_>| {
                redaction_count += 1;
                "[REDACTED]"
            })
            .into_owned();
    }
    RedactionResultV2 {
        text,
        redaction_count,
    }
}

/// Returns a copy suitable for durable local storage and the total redaction count.
pub fn sanitize_input(input: &ReleaseTruthInputV2) -> (ReleaseTruthInputV2, usize) {
    let mut sanitized = input.clone();
    let mut total = 0;
    for claim in &mut sanitized.claims {
        let x = redact_text(&claim.id);
        total += x.redaction_count;
        claim.id = x.text;
        let x = redact_text(&claim.text);
        total += x.redaction_count;
        claim.text = x.text;
        for id in &mut claim.required_evidence {
            let x = redact_text(id);
            total += x.redaction_count;
            *id = x.text;
        }
    }
    for command in &mut sanitized.commands {
        for value in &mut command.argv {
            let x = redact_text(value);
            total += x.redaction_count;
            *value = x.text;
        }
        let x = redact_text(&command.cwd);
        total += x.redaction_count;
        command.cwd = x.text;
        let x = redact_text(&command.id);
        total += x.redaction_count;
        command.id = x.text;
        let stdout = redact_text(&command.stdout);
        let stderr = redact_text(&command.stderr);
        command.stdout = stdout.text;
        command.stderr = stderr.text;
        total += stdout.redaction_count + stderr.redaction_count;
    }
    for link in &mut sanitized.links {
        let x = redact_text(&link.claim_id);
        total += x.redaction_count;
        link.claim_id = x.text;
        let x = redact_text(&link.evidence_id);
        total += x.redaction_count;
        link.evidence_id = x.text;
    }
    if let Some(binding) = &mut sanitized.source_binding {
        for snapshot in [&mut binding.pre, &mut binding.post] {
            let x = redact_text(&snapshot.repository_path);
            total += x.redaction_count;
            snapshot.repository_path = x.text;
            let x = redact_text(&snapshot.status);
            total += x.redaction_count;
            snapshot.status = x.text;
        }
    }
    (sanitized, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_digest_is_stable_for_identical_inputs() {
        let input = ReleaseTruthInputV2 {
            schema_version: RELEASE_TRUTH_INPUT_V2.into(),
            run_id: "stable".into(),
            claims: vec![ExplicitClaimV2 {
                id: "claim".into(),
                text: "claim".into(),
                required_evidence: vec!["command".into()],
            }],
            commands: vec![CommandEvidenceV2 {
                id: "command".into(),
                execution_mode: "argv".into(),
                argv: vec!["true".into()],
                cwd: "/fixture".into(),
                outcome: CommandOutcomeV2::Passed,
                stdout: String::new(),
                stderr: String::new(),
                observed_at: "2026-01-01T00:00:00Z".into(),
                recorded_at: "2026-01-01T00:00:01Z".into(),
            }],
            links: vec![ClaimEvidenceLinkV2 {
                claim_id: "claim".into(),
                evidence_id: "command".into(),
                relation: EvidenceRelationV2::Supports,
            }],
            source_binding: Some(SourceBindingV2 {
                pre: SourceSnapshotV2 {
                    repository_path: "/fixture".into(),
                    head: "h".into(),
                    tree: "t".into(),
                    status: String::new(),
                    is_clean: true,
                    diff_digest: "d".into(),
                    workspace_content_digest: "w".into(),
                    observed_at: "2026-01-01T00:00:00Z".into(),
                },
                post: SourceSnapshotV2 {
                    repository_path: "/fixture".into(),
                    head: "h".into(),
                    tree: "t".into(),
                    status: String::new(),
                    is_clean: true,
                    diff_digest: "d".into(),
                    workspace_content_digest: "w".into(),
                    observed_at: "2026-01-01T00:00:00Z".into(),
                },
            }),
        };
        let first = evaluate(&input).expect("first evaluation");
        let second = evaluate(&input).expect("second evaluation");
        assert_eq!(first.canonical_digest, second.canonical_digest);
        let mut missing_post_time = input;
        missing_post_time
            .source_binding
            .as_mut()
            .expect("binding")
            .post
            .observed_at
            .clear();
        assert!(evaluate(&missing_post_time).is_err());
    }
}
