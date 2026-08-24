//! Public semantic-memory `Embedder` observation wrapper.

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

    /// Emit validated receipt metadata without retaining raw receipt JSON.
    pub fn observe(&self, metadata: &LlmReceiptMetadataV1) -> Result<(), String> {
        metadata.validate()?;
        let mut observation = ObservationEnvelope::metadata(
            self.producer_id.clone(),
            "semantic-memory",
            "llm-receipt-metadata",
            self.sequence.fetch_add(1, Ordering::Relaxed),
            ObservationKind::Receipt,
            if metadata.integrity_verified {
                LifecycleStatus::Completed
            } else {
                LifecycleStatus::Failed
            },
            format!("LLM receipt metadata {}", metadata.pipeline_id),
        );
        observation.correlation.run_id = Some(metadata.pipeline_id.clone());
        observation.correlation.trace_id = metadata.traceparent.clone();
        observation.timing.model = Some(metadata.model.clone());
        observation.timing.provider = Some(metadata.provider.clone());
        observation.payload = serde_json::json!({
            "receipt_digest": metadata.receipt_digest,
            "integrity_verified": metadata.integrity_verified,
        });
        self.client
            .try_emit(observation)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

#[allow(dead_code)]
fn _memory_error_type_is_public(_: MemoryError) {}
