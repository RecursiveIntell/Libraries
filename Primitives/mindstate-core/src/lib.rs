//! # mindstate-core
//!
//! Serializable mindstate payload types for `forge-engine`.
//!
//! This crate owns the stable data structures used to render and persist a
//! compiled mindstate. It does not own retrieval, execution, or scoring logic.

//! # mindstate-core
//!
//! Deterministic mindstate rendering and hashing for internal forge flows.
//!
//! This Tier 3 crate defines the serializable `MindState` payload and its
//! reproducible hash/render helpers. Higher-level runtime orchestration belongs
//! to `forge-engine`.

use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
pub enum MindStateError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MindState {
    pub request: String,
    pub repo_context: String,
    pub evidence: Vec<EvidenceItem>,
    pub traces: Vec<TraceRef>,
    pub basis_version_id: String,
    pub config_overrides: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct EvidenceItem {
    pub key: String,
    pub content: String,
    pub score: OrderedFloat,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct TraceRef {
    pub question_sig: String,
    pub strategy_tags: Vec<String>,
    pub score: OrderedFloat,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct OrderedFloat(pub f64);

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl MindState {
    pub fn render(&self) -> Result<String, MindStateError> {
        let mut evidence = self.evidence.clone();
        evidence.sort();

        let mut traces = self.traces.clone();
        traces.sort();

        let rendered = RenderedMindState {
            request: &self.request,
            repo_context: &self.repo_context,
            evidence: &evidence,
            traces: &traces,
            basis_version_id: &self.basis_version_id,
            config_overrides: &self.config_overrides,
        };

        Ok(serde_json::to_string_pretty(&rendered)?)
    }

    pub fn hash(&self) -> Result<String, MindStateError> {
        let rendered = self.render()?;
        Ok(blake3::hash(rendered.as_bytes()).to_hex().to_string())
    }
}

pub fn compute_question_sig(request: &str, repo_context: &str) -> String {
    blake3::hash(format!("{request}\n{repo_context}").as_bytes())
        .to_hex()
        .to_string()
}

pub fn budget_evidence(mut evidence: Vec<EvidenceItem>, budget: usize) -> Vec<EvidenceItem> {
    evidence.truncate(budget);
    evidence
}

#[derive(serde::Serialize)]
struct RenderedMindState<'a> {
    request: &'a str,
    repo_context: &'a str,
    evidence: &'a [EvidenceItem],
    traces: &'a [TraceRef],
    basis_version_id: &'a str,
    config_overrides: &'a BTreeMap<String, String>,
}

#[cfg(test)]
mod wave1_tests {
    use super::*;

    fn sample_mindstate(evidence: Vec<EvidenceItem>, traces: Vec<TraceRef>) -> MindState {
        MindState {
            request: "fix flaky test".into(),
            repo_context: "workspace".into(),
            evidence,
            traces,
            basis_version_id: "v1".into(),
            config_overrides: BTreeMap::from([("mode".into(), "strict".into())]),
        }
    }

    #[test]
    fn render_and_hash_are_stable_across_input_ordering() {
        let a = sample_mindstate(
            vec![
                EvidenceItem {
                    key: "b".into(),
                    content: "second".into(),
                    score: OrderedFloat(0.2),
                },
                EvidenceItem {
                    key: "a".into(),
                    content: "first".into(),
                    score: OrderedFloat(0.9),
                },
            ],
            vec![
                TraceRef {
                    question_sig: "q2".into(),
                    strategy_tags: vec!["fallback".into()],
                    score: OrderedFloat(0.1),
                },
                TraceRef {
                    question_sig: "q1".into(),
                    strategy_tags: vec!["primary".into()],
                    score: OrderedFloat(0.8),
                },
            ],
        );
        let b = sample_mindstate(
            a.evidence.iter().cloned().rev().collect(),
            a.traces.iter().cloned().rev().collect(),
        );

        assert_eq!(a.render().unwrap(), b.render().unwrap());
        assert_eq!(a.hash().unwrap(), b.hash().unwrap());
    }

    #[test]
    fn compute_question_sig_is_deterministic_and_input_sensitive() {
        let baseline = compute_question_sig("question", "repo");
        assert_eq!(baseline, compute_question_sig("question", "repo"));
        assert_ne!(baseline, compute_question_sig("question changed", "repo"));
    }

    #[test]
    fn budget_evidence_truncates_without_reordering() {
        let evidence = vec![
            EvidenceItem {
                key: "a".into(),
                content: "first".into(),
                score: OrderedFloat(0.9),
            },
            EvidenceItem {
                key: "b".into(),
                content: "second".into(),
                score: OrderedFloat(0.8),
            },
            EvidenceItem {
                key: "c".into(),
                content: "third".into(),
                score: OrderedFloat(0.7),
            },
        ];

        let budgeted = budget_evidence(evidence, 2);
        assert_eq!(budgeted.len(), 2);
        assert_eq!(budgeted[0].key, "a");
        assert_eq!(budgeted[1].key, "b");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> MindState {
        MindState {
            request: "How do we fix the failing test?".into(),
            repo_context: "workspace-a".into(),
            evidence: vec![
                EvidenceItem {
                    key: "zeta".into(),
                    content: "later evidence".into(),
                    score: OrderedFloat(0.2),
                },
                EvidenceItem {
                    key: "alpha".into(),
                    content: "earlier evidence".into(),
                    score: OrderedFloat(0.8),
                },
            ],
            traces: vec![
                TraceRef {
                    question_sig: "sig-b".into(),
                    strategy_tags: vec!["mechanical".into()],
                    score: OrderedFloat(0.1),
                },
                TraceRef {
                    question_sig: "sig-a".into(),
                    strategy_tags: vec!["refactor".into()],
                    score: OrderedFloat(0.9),
                },
            ],
            basis_version_id: "v1".into(),
            config_overrides: BTreeMap::from([
                ("mode".into(), "strict".into()),
                ("budget".into(), "small".into()),
            ]),
        }
    }

    #[test]
    fn render_sorts_evidence_and_traces_deterministically() {
        let rendered = sample_state().render().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(parsed["evidence"][0]["key"], "alpha");
        assert_eq!(parsed["evidence"][1]["key"], "zeta");
        assert_eq!(parsed["traces"][0]["question_sig"], "sig-a");
        assert_eq!(parsed["traces"][1]["question_sig"], "sig-b");
    }

    #[test]
    fn hash_is_order_independent_for_equivalent_content() {
        let first = sample_state();
        let mut second = sample_state();
        second.evidence.reverse();
        second.traces.reverse();

        assert_eq!(first.hash().unwrap(), second.hash().unwrap());
    }

    #[test]
    fn compute_question_sig_changes_with_inputs() {
        let baseline = compute_question_sig("request", "repo-a");
        let different_repo = compute_question_sig("request", "repo-b");
        let different_request = compute_question_sig("request-2", "repo-a");

        assert_ne!(baseline, different_repo);
        assert_ne!(baseline, different_request);
    }

    #[test]
    fn budget_evidence_truncates_without_reordering() {
        let evidence = sample_state().evidence;
        let limited = budget_evidence(evidence.clone(), 1);

        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].key, evidence[0].key);
        assert_eq!(limited[0].content, evidence[0].content);
    }
}
