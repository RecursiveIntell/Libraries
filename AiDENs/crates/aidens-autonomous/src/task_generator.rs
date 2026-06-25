//! Task generation — converts detected gaps into daemon queue jobs.
//!
//! The [`TaskGenerator`] takes a slice of [`DetectedGap`] values, builds a
//! [`JobV1`] for each one using type-specific prompt templates, and enqueues
//! them via a [`DaemonControllerV1`]. The returned job IDs can be tracked
//! through the daemon's normal lease/acquire/complete lifecycle.

use crate::gap_detector::{DetectedGap, GapType};
use aidens_contracts::{
    CanonicalToolSideEffectClass as RiskClass, JobV1,
};
use aidens_daemon_kit::{DaemonControllerV1, DaemonError};
use anyhow::Result;
use sha2::{Digest, Sha256};

/// Generates and enqueues daemon jobs from detected knowledge-base gaps.
#[derive(Clone)]
pub struct TaskGenerator {
    queue: DaemonControllerV1,
}

impl TaskGenerator {
    /// Create a task generator bound to an open daemon controller.
    pub fn new(queue: DaemonControllerV1) -> Self {
        Self { queue }
    }

    /// Build a [`JobV1`] for each gap and enqueue it.
    ///
    /// Returns the list of job IDs (as strings) that were successfully enqueued.
    /// If a particular gap fails to enqueue, the error is propagated immediately.
    pub async fn generate_tasks(&self, gaps: &[DetectedGap]) -> Result<Vec<String>> {
        let mut job_ids = Vec::with_capacity(gaps.len());

        for gap in gaps {
            let prompt = build_prompt(gap);
            let idempotency_key = compute_idempotency_key(&gap.fact_id, &gap.gap_type);

            let payload = serde_json::json!({
                "prompt": prompt,
                "gap_type": gap.gap_type.to_string(),
                "fact_id": gap.fact_id,
                "description": gap.description,
            });

            let namespace_id = self.queue.namespace_id();
            let job = JobV1::new(
                namespace_id,
                idempotency_key,
                "gap-detector",
                payload,
                RiskClass::ReadOnly,
                None,
                None,
            );

            let outcome = self.enqueue(job)?;

            if let Some(job) = outcome.job {
                job_ids.push(job.job_id.to_string());
            } else if let Some(existing) = outcome.existing_job {
                // Duplicate suppression — the job was already enqueued previously.
                job_ids.push(existing.job_id.to_string());
            }
        }

        Ok(job_ids)
    }

    /// Synchronous enqueue wrapper (the daemon controller is sync).
    fn enqueue(&self, job: JobV1) -> Result<aidens_queue_kit::QueueEnqueueOutcomeV1, DaemonError> {
        self.queue.enqueue_job(job)
    }
}

// ---------------------------------------------------------------------------
// Prompt templates
// ---------------------------------------------------------------------------

/// Build a remediation prompt for a detected gap based on its type.
fn build_prompt(gap: &DetectedGap) -> String {
    match gap.gap_type {
        GapType::MissingContext => format!(
            "The knowledge base has a fact with no connections: {}. \
             Search for related concepts and explain how it connects to the broader knowledge graph.",
            gap.description
        ),
        GapType::MissingLink => format!(
            "Two facts in the knowledge base are not connected: {}. \
             Search for their relationship and explain how they relate.",
            gap.description
        ),
        GapType::StaleFact => format!(
            "A fact may be outdated: {}. \
             Search for current information and verify whether it's still accurate.",
            gap.description
        ),
        GapType::ContradictionGap => format!(
            "A potential contradiction was detected: {}. \
             Search for conflicting information and determine which version is correct.",
            gap.description
        ),
    }
}

/// Compute a deterministic idempotency key: `sha256(fact_id + gap_type)`.
fn compute_idempotency_key(fact_id: &str, gap_type: &GapType) -> String {
    let mut hasher = Sha256::new();
    hasher.update(fact_id.as_bytes());
    hasher.update(gap_type.to_string().as_bytes());
    let digest = hasher.finalize();
    format!("gap-detector:{:x}", digest)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gap_detector::{DetectedGap, GapType};

    #[test]
    fn prompt_for_missing_context() {
        let gap = DetectedGap {
            gap_type: GapType::MissingContext,
            fact_id: "fact:abc".to_string(),
            description: "Fact 'abc' is isolated".to_string(),
            suggested_task: "connect it".to_string(),
            priority: 0.8,
        };
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("no connections"));
        assert!(prompt.contains("Fact 'abc' is isolated"));
    }

    #[test]
    fn prompt_for_missing_link() {
        let gap = DetectedGap {
            gap_type: GapType::MissingLink,
            fact_id: "fact:a|fact:b".to_string(),
            description: "Facts a and b are not connected".to_string(),
            suggested_task: "find relationship".to_string(),
            priority: 0.6,
        };
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("not connected"));
        assert!(prompt.contains("Facts a and b are not connected"));
    }

    #[test]
    fn prompt_for_stale_fact() {
        let gap = DetectedGap {
            gap_type: GapType::StaleFact,
            fact_id: "db-integrity".to_string(),
            description: "Integrity check failed".to_string(),
            suggested_task: "reconcile".to_string(),
            priority: 0.9,
        };
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("outdated"));
        assert!(prompt.contains("Integrity check failed"));
    }

    #[test]
    fn prompt_for_contradiction_gap() {
        let gap = DetectedGap {
            gap_type: GapType::ContradictionGap,
            fact_id: "fact:conflict".to_string(),
            description: "Two facts disagree".to_string(),
            suggested_task: "resolve".to_string(),
            priority: 0.7,
        };
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("contradiction"));
        assert!(prompt.contains("Two facts disagree"));
    }

    #[test]
    fn idempotency_key_is_deterministic() {
        let key_a = compute_idempotency_key("fact:123", &GapType::MissingContext);
        let key_b = compute_idempotency_key("fact:123", &GapType::MissingContext);
        assert_eq!(key_a, key_b);
        assert!(key_a.starts_with("gap-detector:"));
    }

    #[test]
    fn idempotency_key_differs_by_gap_type() {
        let key_a = compute_idempotency_key("fact:123", &GapType::MissingContext);
        let key_b = compute_idempotency_key("fact:123", &GapType::StaleFact);
        assert_ne!(key_a, key_b);
    }

    #[test]
    fn idempotency_key_differs_by_fact_id() {
        let key_a = compute_idempotency_key("fact:123", &GapType::MissingContext);
        let key_b = compute_idempotency_key("fact:456", &GapType::MissingContext);
        assert_ne!(key_a, key_b);
    }

    #[test]
    fn payload_contains_required_fields() {
        let gap = DetectedGap {
            gap_type: GapType::MissingContext,
            fact_id: "fact:abc".to_string(),
            description: "isolated fact".to_string(),
            suggested_task: "connect".to_string(),
            priority: 0.8,
        };
        let payload = serde_json::json!({
            "prompt": build_prompt(&gap),
            "gap_type": gap.gap_type.to_string(),
            "fact_id": gap.fact_id,
            "description": gap.description,
        });

        assert!(payload.get("prompt").is_some());
        assert_eq!(
            payload.get("gap_type").and_then(|v| v.as_str()),
            Some("missing-context")
        );
        assert_eq!(
            payload.get("fact_id").and_then(|v| v.as_str()),
            Some("fact:abc")
        );
        assert_eq!(
            payload.get("description").and_then(|v| v.as_str()),
            Some("isolated fact")
        );
    }
}