//! P18 — Mechanism/theory search, experiment runtime, and falsifiable model library.
//!
//! Candidate mechanisms can be fit, refuted, versioned, superseded, and replayed.
//! Mechanisms are non-authoritative hypotheses — they never promote without
//! refuter suites and governance disposition.

use serde::{Deserialize, Serialize};

/// A candidate mechanism/theory artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismBundleV1 {
    pub mechanism_id: String,
    pub name: String,
    pub description: String,
    pub fit_score: f64,
    pub refuter_suite: Option<RefuterSuiteV1>,
    pub status: MechanismStatus,
    pub version: u32,
    pub superseded_by: Option<String>,
    pub timestamp: String,
}

/// A refuter suite — attempts to disprove a mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefuterSuiteV1 {
    pub suite_id: String,
    pub refuters: Vec<RefuterV1>,
    pub all_passed: bool,
}

/// A single refuter attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefuterV1 {
    pub refuter_id: String,
    pub refutation_method: String,
    pub result: RefutationResult,
    pub evidence: String,
}

/// Result of a refutation attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefutationResult {
    /// Could not refute — mechanism survives.
    Survived,
    /// Successfully refuted — mechanism is falsified.
    Refuted,
    /// Inconclusive — more evidence needed.
    Inconclusive,
}

/// Status of a mechanism in the publication lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MechanismStatus {
    /// Just fit, no refuters run yet.
    Candidate,
    /// Refuters run, all survived.
    Tested,
    /// Refuted by at least one refuter.
    Refuted,
    /// Superseded by a newer version.
    Superseded,
    /// Published (requires refuter suite + governance approval).
    Published,
}

/// Adapter for mechanism search and lifecycle.
pub struct MechanismAdapter;

impl MechanismAdapter {
    /// Fit a candidate mechanism to data.
    pub fn fit_mechanism(name: &str, description: &str, fit_score: f64) -> MechanismBundleV1 {
        MechanismBundleV1 {
            mechanism_id: format!("mech:{}:{}", name, chrono::Utc::now().timestamp()),
            name: name.to_string(),
            description: description.to_string(),
            fit_score,
            refuter_suite: None,
            status: MechanismStatus::Candidate,
            version: 1,
            superseded_by: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Run a refuter suite against a mechanism.
    pub fn refute_mechanism(mechanism: &mut MechanismBundleV1, refuters: Vec<RefuterV1>) {
        let all_passed = refuters.iter().all(|r| r.result == RefutationResult::Survived);
        let any_refuted = refuters.iter().any(|r| r.result == RefutationResult::Refuted);

        mechanism.refuter_suite = Some(RefuterSuiteV1 {
            suite_id: format!("rs:{}:{}", mechanism.mechanism_id, chrono::Utc::now().timestamp()),
            refuters,
            all_passed,
        });

        mechanism.status = if any_refuted {
            MechanismStatus::Refuted
        } else if all_passed {
            MechanismStatus::Tested
        } else {
            MechanismStatus::Candidate
        };
    }

    /// Create a new version of a mechanism, superseding the old one.
    pub fn supersede_mechanism(old: &mut MechanismBundleV1, new_name: &str, new_description: &str, new_fit_score: f64) -> MechanismBundleV1 {
        old.status = MechanismStatus::Superseded;
        let new_version = old.version + 1;
        let new_id = format!("mech:{}:{}", new_name, chrono::Utc::now().timestamp());
        old.superseded_by = Some(new_id.clone());

        MechanismBundleV1 {
            mechanism_id: new_id,
            name: new_name.to_string(),
            description: new_description.to_string(),
            fit_score: new_fit_score,
            refuter_suite: None,
            status: MechanismStatus::Candidate,
            version: new_version,
            superseded_by: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Check if a mechanism can be promoted to published.
    pub fn can_promote(mechanism: &MechanismBundleV1) -> bool {
        // Mechanism must have a refuter suite where all refuters survived
        match &mechanism.refuter_suite {
            Some(suite) => suite.all_passed && mechanism.status == MechanismStatus::Tested,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_mechanism_starts_as_candidate() {
        let m = MechanismAdapter::fit_mechanism("test-mech", "a test", 0.85);
        assert_eq!(m.status, MechanismStatus::Candidate);
        assert!(m.refuter_suite.is_none());
    }

    #[test]
    fn mechanism_fit_never_promotes_without_refuter_suite() {
        let m = MechanismAdapter::fit_mechanism("test-mech", "a test", 0.99);
        assert!(!MechanismAdapter::can_promote(&m));
    }

    #[test]
    fn refuted_mechanism_cannot_promote() {
        let mut m = MechanismAdapter::fit_mechanism("test-mech", "a test", 0.85);
        MechanismAdapter::refute_mechanism(&mut m, vec![RefuterV1 {
            refuter_id: "r1".into(),
            refutation_method: "counterexample".into(),
            result: RefutationResult::Refuted,
            evidence: "found counterexample".into(),
        }]);
        assert_eq!(m.status, MechanismStatus::Refuted);
        assert!(!MechanismAdapter::can_promote(&m));
    }

    #[test]
    fn survived_mechanism_can_promote() {
        let mut m = MechanismAdapter::fit_mechanism("test-mech", "a test", 0.85);
        MechanismAdapter::refute_mechanism(&mut m, vec![RefuterV1 {
            refuter_id: "r1".into(),
            refutation_method: "counterexample".into(),
            result: RefutationResult::Survived,
            evidence: "no counterexample found".into(),
        }]);
        assert_eq!(m.status, MechanismStatus::Tested);
        assert!(MechanismAdapter::can_promote(&m));
    }

    #[test]
    fn supersede_creates_new_version() {
        let mut old = MechanismAdapter::fit_mechanism("mech-v1", "first", 0.7);
        let new = MechanismAdapter::supersede_mechanism(&mut old, "mech-v2", "second", 0.9);
        assert_eq!(old.status, MechanismStatus::Superseded);
        assert_eq!(new.version, 2);
        assert_eq!(old.superseded_by.as_deref(), Some(new.mechanism_id.as_str()));
    }

    #[test]
    fn inconclusive_refuter_keeps_candidate() {
        let mut m = MechanismAdapter::fit_mechanism("test", "test", 0.5);
        MechanismAdapter::refute_mechanism(&mut m, vec![RefuterV1 {
            refuter_id: "r1".into(),
            refutation_method: "fuzz".into(),
            result: RefutationResult::Inconclusive,
            evidence: "need more data".into(),
        }]);
        assert_eq!(m.status, MechanismStatus::Candidate);
        assert!(!MechanismAdapter::can_promote(&m));
    }
}
