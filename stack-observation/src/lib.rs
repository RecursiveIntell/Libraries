//! Versioned, dependency-light observation contract for stack telemetry.
//!
//! This crate deliberately does not depend on SQLite, Tauri, or any upstream
//! stack crate. Adapters translate native events into this envelope; collectors
//! validate and persist it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use thiserror::Error;
use uuid::Uuid;

/// Current wire/schema version.
pub const OBSERVATION_SCHEMA_VERSION: u16 = 1;
/// Maximum serialized payload bytes accepted by the contract validator.
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

static GLOBAL_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static GLOBAL_SINK_ID: AtomicU64 = AtomicU64::new(0);
type GlobalSinks = RwLock<Vec<(u64, Arc<dyn ObservationSink>)>>;
static GLOBAL_SINKS: OnceLock<GlobalSinks> = OnceLock::new();

/// Non-blocking sink installed by a host process that wants automatic stack coverage.
pub trait ObservationSink: Send + Sync {
    fn submit(&self, event: ObservationEnvelope) -> bool;
}

/// Registration guard for one process-wide automatic observation sink.
pub struct GlobalSinkGuard {
    id: u64,
}

impl Drop for GlobalSinkGuard {
    fn drop(&mut self) {
        if let Some(slot) = GLOBAL_SINKS.get() {
            if let Ok(mut sinks) = slot.write() {
                sinks.retain(|(id, _)| *id != self.id);
            }
        }
    }
}

/// Allocate a sequence number for a process-wide automatic observation stream.
pub fn next_global_sequence() -> u64 {
    GLOBAL_SEQUENCE.fetch_add(1, Ordering::Relaxed)
}

/// Install one automatic observation sink and return its lifetime guard.
///
/// Existing sinks remain registered and receive a copy of each event. Dropping
/// the returned guard removes only this registration; no host silently disables
/// another sink.
pub fn install_global_sink(sink: Arc<dyn ObservationSink>) -> GlobalSinkGuard {
    let id = GLOBAL_SINK_ID.fetch_add(1, Ordering::Relaxed);
    let slot = GLOBAL_SINKS.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(mut sinks) = slot.write() {
        sinks.push((id, sink));
    }
    GlobalSinkGuard { id }
}

/// Remove all process-wide automatic observation sinks.
///
/// Prefer dropping the `GlobalSinkGuard` returned by `install_global_sink`.
pub fn clear_global_sink() {
    if let Some(slot) = GLOBAL_SINKS.get() {
        if let Ok(mut sinks) = slot.write() {
            sinks.clear();
        }
    }
}

/// Submit an automatically captured event. No-op until a host installs a sink.
pub fn emit_global(event: ObservationEnvelope) -> bool {
    let Some(slot) = GLOBAL_SINKS.get() else {
        return false;
    };
    let Ok(sinks) = slot.read() else {
        return false;
    };
    let mut accepted = false;
    for (_, sink) in sinks.iter() {
        accepted |= sink.submit(event.clone());
    }
    accepted
}

/// A bounded observation envelope exchanged between stack producers and collectors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObservationEnvelope {
    /// Contract version for forward-compatible decoding.
    pub schema_version: u16,
    /// Stable event identity for deduplication.
    pub event_id: Uuid,
    /// Time reported by the producer.
    pub observed_at: DateTime<Utc>,
    /// Time assigned by the collector, when persisted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ingested_at: Option<DateTime<Utc>>,
    /// Producer identity and sequence domain.
    pub producer_id: String,
    /// Process ID when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_id: Option<u32>,
    /// Human/source component name.
    pub source_crate: String,
    /// Adapter that created this event.
    pub adapter_id: String,
    /// Whether the event is native, adapted, inferred, or duplicate.
    pub provenance: Provenance,
    /// Optional canonical correlation identifiers.
    #[serde(default)]
    pub correlation: Correlation,
    /// Monotonic only within one producer sequence domain.
    pub producer_sequence: u64,
    /// Typed event category.
    pub kind: ObservationKind,
    /// Lifecycle state.
    pub status: LifecycleStatus,
    /// Optional timing and accounting fields.
    #[serde(default)]
    pub timing: Timing,
    /// Privacy and redaction state.
    pub privacy: PrivacyMetadata,
    /// Typed-or-structured payload bounded by the validator.
    #[serde(default)]
    pub payload: Value,
    /// Sampling and known-loss metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drop_context: Option<DropContext>,
}

impl ObservationEnvelope {
    /// Construct a metadata-only event with absent lineage preserved.
    pub fn metadata(
        producer_id: impl Into<String>,
        source_crate: impl Into<String>,
        adapter_id: impl Into<String>,
        sequence: u64,
        kind: ObservationKind,
        status: LifecycleStatus,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: OBSERVATION_SCHEMA_VERSION,
            event_id: Uuid::new_v4(),
            observed_at: Utc::now(),
            ingested_at: None,
            producer_id: producer_id.into(),
            process_id: std::process::id().into(),
            source_crate: source_crate.into(),
            adapter_id: adapter_id.into(),
            provenance: Provenance::Adapted,
            correlation: Correlation::default(),
            producer_sequence: sequence,
            kind,
            status,
            timing: Timing::default(),
            privacy: PrivacyMetadata::metadata_only(),
            payload: serde_json::json!({ "summary": summary.into() }),
            sampling: None,
            drop_context: None,
        }
    }

    /// Validate schema, identity, size, and privacy invariants before transport.
    pub fn validate(&self) -> Result<(), ObservationError> {
        if self.schema_version != OBSERVATION_SCHEMA_VERSION {
            return Err(ObservationError::UnsupportedSchema(self.schema_version));
        }
        if self.producer_id.trim().is_empty() {
            return Err(ObservationError::MissingField("producer_id"));
        }
        if self.source_crate.trim().is_empty() {
            return Err(ObservationError::MissingField("source_crate"));
        }
        if self.adapter_id.trim().is_empty() {
            return Err(ObservationError::MissingField("adapter_id"));
        }
        let bytes = serde_json::to_vec(&self.payload)?.len();
        if bytes > MAX_PAYLOAD_BYTES {
            return Err(ObservationError::PayloadTooLarge {
                actual: bytes,
                maximum: MAX_PAYLOAD_BYTES,
            });
        }
        if self.privacy.tier == PrivacyTier::MetadataOnly
            && self.privacy.content_fields > 0
            && self.privacy.redaction == RedactionState::ContentDisabled
        {
            return Err(ObservationError::PrivacyViolation(
                "metadata-only events cannot declare content fields",
            ));
        }
        Ok(())
    }

    /// Apply a collector privacy policy before persistence or export.
    pub fn apply_privacy_policy(&mut self, policy: &PrivacyPolicy) -> PrivacyReport {
        let mut report = PrivacyReport::default();
        if !policy.allow_sensitive_metadata {
            if self.timing.model.take().is_some() {
                report.redacted_fields = report.redacted_fields.saturating_add(1);
            }
            if self.timing.provider.take().is_some() {
                report.redacted_fields = report.redacted_fields.saturating_add(1);
            }
            if self.timing.estimated_cost.take().is_some() {
                report.redacted_fields = report.redacted_fields.saturating_add(1);
            }
            self.timing.currency = None;
            self.timing.error_category = None;
        }
        sanitize_value(&mut self.payload, policy, &mut report);
        self.privacy.content_fields = report.content_fields;
        if report.redacted_fields > 0 {
            self.privacy.redaction = if report.content_fields > 0 {
                RedactionState::Redacted
            } else {
                RedactionState::PartiallyRedacted
            };
        }
        self.privacy.tier = if report.content_fields > 0 {
            if policy.allow_redacted_content {
                PrivacyTier::RedactedContent
            } else {
                PrivacyTier::MetadataOnly
            }
        } else if policy.allow_sensitive_metadata {
            PrivacyTier::SensitiveMetadata
        } else {
            PrivacyTier::MetadataOnly
        };
        report
    }

    /// Mark this event as collected at the supplied time.
    pub fn with_ingested_at(mut self, timestamp: DateTime<Utc>) -> Self {
        self.ingested_at = Some(timestamp);
        self
    }
}

/// Correlation identifiers. Missing source data stays `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Correlation {
    pub session_id: Option<String>,
    pub run_id: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub parent_span_id: Option<String>,
    pub node_id: Option<String>,
    pub attempt_id: Option<String>,
    pub trial_id: Option<String>,
    pub request_id: Option<String>,
}

/// Normalized lifecycle categories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStatus {
    Started,
    Streaming,
    Completed,
    Failed,
    Cancelled,
    Retried,
    Health,
}

/// Typed event categories with an escape hatch for forward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObservationKind {
    LlmCall,
    TokenProgress,
    Retry,
    Parse,
    Transport,
    Cost,
    GraphRun,
    GraphNode,
    Tool,
    Memory,
    Embedding,
    Receipt,
    Health,
    TracingFallback,
    Unknown(String),
}

/// Native/adapted/inferred evidence provenance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Canonical,
    Adapted,
    Inferred,
    Duplicate,
}

/// Timing, usage, and error metadata when supplied by the source.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Timing {
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub estimated_cost: Option<f64>,
    pub currency: Option<String>,
    pub error_category: Option<String>,
}

/// Privacy tier and redaction state attached to every event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivacyMetadata {
    pub tier: PrivacyTier,
    pub redaction: RedactionState,
    pub content_fields: u16,
}

impl PrivacyMetadata {
    /// Default operational metadata policy.
    pub fn metadata_only() -> Self {
        Self {
            tier: PrivacyTier::MetadataOnly,
            redaction: RedactionState::ContentDisabled,
            content_fields: 0,
        }
    }
}

/// Collector privacy controls. The default is metadata-only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PrivacyPolicy {
    pub allow_sensitive_metadata: bool,
    pub allow_redacted_content: bool,
}

/// Counts produced by privacy sanitization.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PrivacyReport {
    pub redacted_fields: u16,
    pub content_fields: u16,
}

fn sanitize_value(value: &mut Value, policy: &PrivacyPolicy, report: &mut PrivacyReport) {
    match value {
        Value::Object(object) => {
            for (key, child) in object.iter_mut() {
                let key_lower = key.to_ascii_lowercase();
                let key_compact = key_lower.replace(['_', '-'], "");
                let sensitive_key = matches!(
                    key_compact.as_str(),
                    "apikey"
                        | "password"
                        | "passwd"
                        | "secret"
                        | "clientsecret"
                        | "authorization"
                        | "cookie"
                        | "accesstoken"
                        | "refreshtoken"
                );
                let content_key = matches!(
                    key_lower.as_str(),
                    "prompt"
                        | "response"
                        | "content"
                        | "token"
                        | "tokens_text"
                        | "tool_args"
                        | "tool_arguments"
                        | "tool_result"
                        | "interrupt_payload"
                        | "search_snippet"
                );
                if sensitive_key {
                    report.redacted_fields = report.redacted_fields.saturating_add(1);
                    *child = Value::String("[redacted secret]".into());
                } else if content_key {
                    report.content_fields = report.content_fields.saturating_add(1);
                    report.redacted_fields = report.redacted_fields.saturating_add(1);
                    *child = if policy.allow_redacted_content {
                        Value::String("[redacted content]".into())
                    } else {
                        Value::String("[content disabled]".into())
                    };
                } else {
                    sanitize_value(child, policy, report);
                }
            }
        }
        Value::Array(items) => {
            for child in items {
                sanitize_value(child, policy, report);
            }
        }
        Value::String(text) if looks_like_secret(text) => {
            *value = Value::String("[redacted secret]".into());
            report.redacted_fields = report.redacted_fields.saturating_add(1);
        }
        _ => {}
    }
}

fn looks_like_secret(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("bearer ")
        || lower.contains("authorization:")
        || lower.contains("api_key")
        || lower.contains("password=")
        || lower.contains("cookie=")
        || lower.contains("secret=")
        || lower.contains("sk-")
}

/// Privacy capture tier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrivacyTier {
    MetadataOnly,
    SensitiveMetadata,
    RedactedContent,
}

/// Whether and how content was redacted.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RedactionState {
    ContentDisabled,
    NotRedacted,
    PartiallyRedacted,
    Redacted,
}

/// Sampling state for stream/token events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SamplingState {
    pub sampled: bool,
    pub sample_every: Option<u32>,
    pub sampled_count: u64,
}

/// Known loss or producer sequence gaps.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DropContext {
    pub dropped_before: u64,
    pub sequence_gap_start: Option<u64>,
    pub sequence_gap_end: Option<u64>,
    pub reason: String,
}

/// Contract validation/serialization failures.
#[derive(Debug, Error)]
pub enum ObservationError {
    #[error("unsupported observation schema version {0}")]
    UnsupportedSchema(u16),
    #[error("missing required field {0}")]
    MissingField(&'static str),
    #[error("observation payload is {actual} bytes; maximum is {maximum}")]
    PayloadTooLarge { actual: usize, maximum: usize },
    #[error("privacy violation: {0}")]
    PrivacyViolation(&'static str),
    #[error("observation serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_event_preserves_absent_lineage_and_validates() {
        let event = ObservationEnvelope::metadata(
            "producer-a",
            "llm-pipeline",
            "test-adapter",
            1,
            ObservationKind::LlmCall,
            LifecycleStatus::Started,
            "call started",
        );
        assert!(event.correlation.trace_id.is_none());
        assert!(event.correlation.attempt_id.is_none());
        assert_eq!(event.privacy.tier, PrivacyTier::MetadataOnly);
        event.validate().unwrap();
    }

    #[test]
    fn metadata_only_rejects_declared_content() {
        let mut event = ObservationEnvelope::metadata(
            "producer-a",
            "llm-pipeline",
            "test-adapter",
            1,
            ObservationKind::TokenProgress,
            LifecycleStatus::Streaming,
            "sampled",
        );
        event.privacy.content_fields = 1;
        assert!(matches!(
            event.validate(),
            Err(ObservationError::PrivacyViolation(_))
        ));
    }

    #[test]
    fn oversized_payload_is_rejected() {
        let mut event = ObservationEnvelope::metadata(
            "producer-a",
            "llm-pipeline",
            "test-adapter",
            1,
            ObservationKind::Health,
            LifecycleStatus::Health,
            "health",
        );
        event.payload = Value::String("x".repeat(MAX_PAYLOAD_BYTES + 1));
        assert!(matches!(
            event.validate(),
            Err(ObservationError::PayloadTooLarge { .. })
        ));
    }

    #[test]
    fn default_policy_redacts_content_and_secret_sentinels() {
        let mut event = ObservationEnvelope::metadata(
            "producer-a",
            "llm-pipeline",
            "test-adapter",
            1,
            ObservationKind::LlmCall,
            LifecycleStatus::Completed,
            "done",
        );
        event.payload = serde_json::json!({
            "prompt": "private prompt",
            "authorization": "Bearer sk-secret-value",
            "nested": {"password=super-secret": "value"}
        });
        let report = event.apply_privacy_policy(&PrivacyPolicy::default());
        assert_eq!(report.content_fields, 1);
        assert!(report.redacted_fields >= 2);
        let encoded = serde_json::to_string(&event.payload).unwrap();
        assert!(!encoded.contains("private prompt"));
        assert!(!encoded.contains("sk-secret-value"));
        assert!(event.validate().is_ok());
        assert_eq!(event.privacy.redaction, RedactionState::Redacted);
    }

    #[test]
    fn opt_in_content_is_still_redacted_and_marked() {
        let mut event = ObservationEnvelope::metadata(
            "producer-a",
            "llm-pipeline",
            "test-adapter",
            1,
            ObservationKind::LlmCall,
            LifecycleStatus::Completed,
            "done",
        );
        event.payload = serde_json::json!({"response": "response text"});
        let policy = PrivacyPolicy {
            allow_sensitive_metadata: true,
            allow_redacted_content: true,
        };
        event.apply_privacy_policy(&policy);
        assert_eq!(event.privacy.tier, PrivacyTier::RedactedContent);
        assert_eq!(event.payload["response"], "[redacted content]");
    }

    #[test]
    fn sensitive_keys_are_redacted_without_secret_markers() {
        let mut event = ObservationEnvelope::metadata(
            "producer-a",
            "llm-pipeline",
            "test-adapter",
            1,
            ObservationKind::Health,
            LifecycleStatus::Health,
            "health",
        );
        event.payload = serde_json::json!({
            "api_key": "abc123",
            "nested": {"password": "plain", "client-secret": "opaque"}
        });

        let report = event.apply_privacy_policy(&PrivacyPolicy::default());
        let encoded = serde_json::to_string(&event.payload).unwrap();
        assert!(report.redacted_fields >= 3);
        assert!(!encoded.contains("abc123"));
        assert!(!encoded.contains("plain"));
        assert!(!encoded.contains("opaque"));
    }

    #[test]
    fn global_sink_registrations_compose_and_guards_are_independent() {
        struct Counter(std::sync::atomic::AtomicU64);
        impl ObservationSink for Counter {
            fn submit(&self, _event: ObservationEnvelope) -> bool {
                self.0.fetch_add(1, Ordering::Relaxed);
                true
            }
        }

        clear_global_sink();
        let first = Arc::new(Counter(std::sync::atomic::AtomicU64::new(0)));
        let second = Arc::new(Counter(std::sync::atomic::AtomicU64::new(0)));
        let first_guard = install_global_sink(Arc::clone(&first) as Arc<dyn ObservationSink>);
        let second_guard = install_global_sink(Arc::clone(&second) as Arc<dyn ObservationSink>);
        assert!(emit_global(ObservationEnvelope::metadata(
            "producer-a",
            "health",
            "test",
            1,
            ObservationKind::Health,
            LifecycleStatus::Health,
            "one",
        )));
        assert_eq!(first.0.load(Ordering::Relaxed), 1);
        assert_eq!(second.0.load(Ordering::Relaxed), 1);

        drop(first_guard);
        assert!(emit_global(ObservationEnvelope::metadata(
            "producer-a",
            "health",
            "test",
            2,
            ObservationKind::Health,
            LifecycleStatus::Health,
            "two",
        )));
        assert_eq!(first.0.load(Ordering::Relaxed), 1);
        assert_eq!(second.0.load(Ordering::Relaxed), 2);
        drop(second_guard);
        assert!(!emit_global(ObservationEnvelope::metadata(
            "producer-a",
            "health",
            "test",
            3,
            ObservationKind::Health,
            LifecycleStatus::Health,
            "three",
        )));
    }
}
