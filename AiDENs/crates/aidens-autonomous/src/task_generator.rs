//! Task generation — converts detected gaps into daemon queue jobs.
//!
//! The [`TaskGenerator`] takes a slice of [`DetectedGap`] values, builds a
//! [`JobV1`] for each one using type-specific prompt templates, and enqueues
//! them via a [`DaemonControllerV1`]. The returned job IDs can be tracked
//! through the daemon's normal lease/acquire/complete lifecycle.

use crate::gap_detector::{DetectedGap, GapType};
use aidens_contracts::{CanonicalToolSideEffectClass as RiskClass, JobV1};
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
    let snippet = gap.content_snippet.as_deref().unwrap_or(&gap.description);

    match gap.gap_type {
        GapType::MissingContext => format!(
            "The knowledge base has an isolated fact with no graph connections: '{}'. \
             Analyze this fact and explain what it relates to in the context of the \
             RecursiveIntell project ecosystem. Be specific about connections to other \
             projects, crates, or concepts.",
            snippet
        ),
        GapType::MissingLink => {
            let fact_a = snippet;
            let fact_b = gap
                .fact_id_b
                .as_deref()
                .unwrap_or("another fact in the same namespace");
            let ns = gap.namespace.as_deref().unwrap_or("the");
            format!(
                "Two facts in the '{}' namespace lack a graph edge: '{}' and '{}'. \
                 Analyze their relationship and explain how they connect.",
                ns, fact_a, fact_b
            )
        }
        GapType::StaleFact => format!(
            "The semantic memory integrity check reported: {}. \
             Analyze what might cause this and suggest how to verify and fix the issue.",
            gap.description
        ),
        GapType::ContradictionGap => {
            let fact_a = snippet;
            let fact_b = gap
                .content_b
                .as_deref()
                .or(gap.fact_id_b.as_deref())
                .unwrap_or("another fact with conflicting information");
            format!(
                "Two facts may contradict each other: '{}' vs '{}'. \
                 Analyze whether this is a real contradiction or a scope/time difference.",
                fact_a, fact_b
            )
        }
        GapType::DuplicateFact => {
            let fact_b = gap
                .content_b
                .as_deref()
                .or(gap.fact_id_b.as_deref())
                .unwrap_or("another fact");
            format!(
                "A fact appears to duplicate another: '{}' vs '{}'. \
                 Determine which version is more complete and accurate.",
                snippet, fact_b
            )
        }
        GapType::StaleByDate => {
            let date = gap.date.as_deref().unwrap_or("an old date");
            format!(
                "A fact references date {} which may be outdated: '{}'. \
                 Check if the information is still current.",
                date, snippet
            )
        }
        GapType::LowQualityFact => format!(
            "A fact appears to be low quality: '{}'. \
             Determine if it should be kept, improved, or removed.",
            snippet
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

    fn make_gap(gt: GapType, fact_id: &str, snippet: &str) -> DetectedGap {
        DetectedGap {
            gap_type: gt,
            fact_id: fact_id.to_string(),
            description: "description".to_string(),
            suggested_task: "task".to_string(),
            priority: 0.5,
            content_snippet: Some(snippet.to_string()),
            fact_id_b: None,
            content_b: None,
            namespace: Some("general".to_string()),
            date: None,
        }
    }

    #[test]
    fn prompt_for_missing_context() {
        let gap = make_gap(
            GapType::MissingContext,
            "fact:abc",
            "Rust is a systems programming language",
        );
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("isolated fact"));
        assert!(prompt.contains("Rust is a systems programming language"));
        assert!(prompt.contains("RecursiveIntell"));
    }

    #[test]
    fn prompt_for_missing_link() {
        let mut gap = make_gap(GapType::MissingLink, "fact:a|fact:b", "Fact A content");
        gap.fact_id_b = Some("fact:b".to_string());
        gap.namespace = Some("research".to_string());
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("research"));
        assert!(prompt.contains("Fact A content"));
        assert!(prompt.contains("fact:b"));
    }

    #[test]
    fn prompt_for_stale_fact() {
        let gap = make_gap(GapType::StaleFact, "db-integrity", "some snippet");
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("integrity check"));
        assert!(prompt.contains("description"));
    }

    #[test]
    fn prompt_for_contradiction_gap() {
        let mut gap = make_gap(
            GapType::ContradictionGap,
            "fact:conflict",
            "Rust has 49 tests",
        );
        gap.fact_id_b = Some("fact:other".to_string());
        gap.content_b = Some("Rust has 50 tests".to_string());
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("contradict"));
        assert!(prompt.contains("Rust has 49 tests"));
        assert!(prompt.contains("Rust has 50 tests"));
    }

    #[test]
    fn prompt_for_contradiction_gap_falls_back_to_fact_id_b() {
        let mut gap = make_gap(
            GapType::ContradictionGap,
            "fact:conflict",
            "Rust has 49 tests",
        );
        gap.fact_id_b = Some("fact:other".to_string());
        // content_b is None — should fall back to fact_id_b
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("contradict"));
        assert!(prompt.contains("Rust has 49 tests"));
        assert!(prompt.contains("fact:other"));
    }

    #[test]
    fn prompt_for_duplicate_fact() {
        let mut gap = make_gap(
            GapType::DuplicateFact,
            "fact:dup",
            "Rust is a systems language",
        );
        gap.fact_id_b = Some("fact:dup2".to_string());
        gap.content_b = Some("Rust is a systems programming language".to_string());
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("duplicate"));
        assert!(prompt.contains("Rust is a systems language"));
        assert!(prompt.contains("Rust is a systems programming language"));
    }

    #[test]
    fn prompt_for_stale_by_date() {
        let mut gap = make_gap(GapType::StaleByDate, "fact:stale", "Released in 2023");
        gap.date = Some("2023".to_string());
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("2023"));
        assert!(prompt.contains("outdated"));
    }

    #[test]
    fn prompt_for_low_quality_fact() {
        let gap = make_gap(
            GapType::LowQualityFact,
            "fact:low",
            "short url heavy content",
        );
        let prompt = build_prompt(&gap);
        assert!(prompt.contains("low quality"));
        assert!(prompt.contains("short url heavy content"));
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
        let gap = make_gap(GapType::MissingContext, "fact:abc", "some content snippet");
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
            Some("description")
        );
    }
}
