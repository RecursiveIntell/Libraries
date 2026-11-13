//! Public semantic-memory `Embedder` observation wrapper.

use crate::EmitStatus;
use semantic_memory::embedder::{EmbedBatchFuture, EmbedFuture, Embedder};
use semantic_memory::{LlmReceiptMetadataV1, MemoryError};
use stack_observation::{LifecycleStatus, ObservationEnvelope, ObservationKind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Instruments an embedder without capturing input text or vector contents.
pub struct EmbedderObservationWrapper<E> {
    inner: Arc<E>,
    client: crate::MonitorClient,
    producer_id: String,
    sequence: Arc<AtomicU64>,
}

impl<E> EmbedderObservationWrapper<E> {
    /// Wrap a public semantic-memory embedder with metadata-only observations.
    pub fn new(
        inner: Arc<E>,
        client: crate::MonitorClient,
        producer_id: impl Into<String>,
    ) -> Self {
        Self {
            inner,
            client,
            producer_id: producer_id.into(),
            sequence: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl<E> Embedder for EmbedderObservationWrapper<E>
where
    E: Embedder + 'static,
{
    fn embed<'a>(&'a self, text: &'a str) -> EmbedFuture<'a> {
        let inner = Arc::clone(&self.inner);
        let client = self.client.clone();
        let producer_id = self.producer_id.clone();
        let sequence = Arc::clone(&self.sequence);
        let model = inner.model_name().to_string();
        let dimensions = inner.dimensions();
        Box::pin(async move {
            let started = Instant::now();
            let result = inner.embed(text).await;
            let mut observation = ObservationEnvelope::metadata(
                producer_id,
                "semantic-memory",
                "embedder-observation",
                sequence.fetch_add(1, Ordering::Relaxed),
                ObservationKind::Embedding,
                if result.is_ok() {
                    LifecycleStatus::Completed
                } else {
                    LifecycleStatus::Failed
                },
                "embedding operation completed",
            );
            observation.timing.model = Some(model);
            observation.timing.duration_ms = Some(started.elapsed().as_millis() as u64);
            observation.payload = serde_json::json!({"dimensions": dimensions, "batch_size": 1});
            let _ = client.try_emit(observation);
            result
        })
    }

    fn embed_batch<'a>(&'a self, texts: Vec<String>) -> EmbedBatchFuture<'a> {
        let inner = Arc::clone(&self.inner);
        let client = self.client.clone();
        let producer_id = self.producer_id.clone();
        let sequence = Arc::clone(&self.sequence);
        let model = inner.model_name().to_string();
        let dimensions = inner.dimensions();
        let batch_size = texts.len();
        Box::pin(async move {
            let started = Instant::now();
            let result = inner.embed_batch(texts).await;
            let mut observation = ObservationEnvelope::metadata(
                producer_id,
                "semantic-memory",
                "embedder-observation",
                sequence.fetch_add(1, Ordering::Relaxed),
                ObservationKind::Embedding,
                if result.is_ok() {
                    LifecycleStatus::Completed
                } else {
                    LifecycleStatus::Failed
                },
                "embedding batch completed",
            );
            observation.timing.model = Some(model);
            observation.timing.duration_ms = Some(started.elapsed().as_millis() as u64);
            observation.payload =
                serde_json::json!({"dimensions": dimensions, "batch_size": batch_size});
            let _ = client.try_emit(observation);
            result
        })
    }

    fn model_name(&self) -> &str {
        self.inner.model_name()
    }

    fn dimensions(&self) -> usize {
        self.inner.dimensions()
    }
}

/// Converts public semantic-memory LLM receipt metadata into observations.
pub struct SemanticMemoryReceiptObservationSink {
    client: crate::MonitorClient,
    producer_id: String,
    sequence: AtomicU64,
}

impl SemanticMemoryReceiptObservationSink {
    /// Create a collector-backed read-only receipt metadata adapter.
    pub fn new(client: crate::MonitorClient, producer_id: impl Into<String>) -> Self {
        Self {
            client,
            producer_id: producer_id.into(),
            sequence: AtomicU64::new(0),
        }
    }

    /// Observe structurally valid, caller-reported receipt metadata.
    /// This does not verify receipt integrity or retain raw receipt JSON.
    pub fn observe(&self, metadata: &LlmReceiptMetadataV1) -> Result<(), String> {
        metadata.validate()?;
        let mut observation = ObservationEnvelope::metadata(
            self.producer_id.clone(),
            "semantic-memory",
            "llm-receipt-metadata",
            self.sequence.fetch_add(1, Ordering::Relaxed),
            ObservationKind::Receipt,
            LifecycleStatus::Health,
            format!("LLM receipt metadata {}", metadata.pipeline_id),
        );
        observation.correlation.run_id = Some(metadata.pipeline_id.clone());
        observation.correlation.trace_id = metadata.traceparent.clone();
        observation.timing.model = Some(metadata.model.clone());
        observation.timing.provider = Some(metadata.provider.clone());
        observation.payload = serde_json::json!({
            "receipt_digest": metadata.receipt_digest,
            "integrity_verified": metadata.integrity_verified,
            "integrity_verification_basis": "caller_reported",
        });
        match self.client.try_emit(observation) {
            Ok(EmitStatus::Accepted) => Ok(()),
            Ok(EmitStatus::Dropped) => {
                Err("receipt metadata observation dropped by collector queue".into())
            }
            Ok(EmitStatus::CollectorUnavailable) => {
                Err("receipt metadata collector unavailable".into())
            }
            Err(error) => Err(error.to_string()),
        }
    }
}

#[allow(dead_code)]
fn _memory_error_type_is_public(_: MemoryError) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{start_collector, ActivityStore, ObservationFilter};

    fn metadata(reported: bool) -> LlmReceiptMetadataV1 {
        LlmReceiptMetadataV1::new(
            "sha256:source-reported-only",
            None,
            "pipeline-test",
            "provider-test",
            "model-test",
            reported,
        )
        .unwrap()
    }

    #[test]
    fn caller_reported_integrity_is_not_a_verified_lifecycle_outcome() {
        let _guard = crate::test_support::global_sink_guard();
        let store = ActivityStore::open(":memory:").unwrap();
        let (client, collector) = start_collector(store.clone(), 8);
        let sink = SemanticMemoryReceiptObservationSink::new(client, "receipt-observer");
        sink.observe(&metadata(true)).unwrap();
        sink.observe(&metadata(false)).unwrap();
        collector.shutdown();

        let events = store
            .query_observations(&ObservationFilter {
                producer_id: Some("receipt-observer".into()),
                ..ObservationFilter::default()
            })
            .unwrap();
        assert_eq!(events.len(), 2);
        assert!(events
            .iter()
            .all(|event| event.status == LifecycleStatus::Health));
        assert!(events
            .iter()
            .all(|event| event.provenance == stack_observation::Provenance::Adapted));
        assert!(events
            .iter()
            .any(|event| event.payload["integrity_verified"] == true));
        assert!(events
            .iter()
            .any(|event| event.payload["integrity_verified"] == false));
        assert!(events
            .iter()
            .all(|event| event.payload["integrity_verification_basis"] == "caller_reported"));
    }

    #[test]
    fn unavailable_collector_does_not_claim_observation_delivery() {
        let _guard = crate::test_support::global_sink_guard();
        let store = ActivityStore::open(":memory:").unwrap();
        let (client, collector) = start_collector(store, 8);
        collector.shutdown();
        let sink = SemanticMemoryReceiptObservationSink::new(client, "receipt-observer");
        assert!(sink.observe(&metadata(true)).is_err());
    }

    #[test]
    fn full_queue_is_not_reported_as_successful_observation() {
        let (client, held_receiver) = crate::MonitorClient::test_channel(1);
        let sink = SemanticMemoryReceiptObservationSink::new(client, "receipt-observer");
        assert!(sink.observe(&metadata(true)).is_ok());
        assert!(sink.observe(&metadata(false)).is_err());
        drop(held_receiver);
        assert!(sink.observe(&metadata(true)).is_err());
    }
}
