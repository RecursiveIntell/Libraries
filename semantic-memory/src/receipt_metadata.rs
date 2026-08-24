//! Read-only metadata contract for LLM execution receipts.
//!
//! This type deliberately excludes raw receipt JSON and does not perform any
//! semantic-memory write. It is safe for observation adapters to consume.

use serde::{Deserialize, Serialize};

/// Stable schema label for read-only LLM receipt metadata.
pub const LLM_RECEIPT_METADATA_V1: &str = "semantic-memory.llm-receipt-metadata.v1";

/// Public, read-only metadata derived from an externally retained LLM receipt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmReceiptMetadataV1 {
    pub schema_version: String,
    pub receipt_digest: String,
    pub traceparent: Option<String>,
    pub pipeline_id: String,
    pub provider: String,
    pub model: String,
    pub integrity_verified: bool,
}

impl LlmReceiptMetadataV1 {
    /// Construct metadata without retaining the raw receipt body.
    pub fn new(
        receipt_digest: impl Into<String>,
        traceparent: Option<String>,
        pipeline_id: impl Into<String>,
        provider: impl Into<String>,
        model: impl Into<String>,
        integrity_verified: bool,
    ) -> Result<Self, String> {
        let metadata = Self {
            schema_version: LLM_RECEIPT_METADATA_V1.into(),
            receipt_digest: receipt_digest.into(),
            traceparent,
            pipeline_id: pipeline_id.into(),
            provider: provider.into(),
            model: model.into(),
            integrity_verified,
        };
        metadata.validate()?;
        Ok(metadata)
    }

    /// Validate bounded identity fields before adaptation or persistence.
    pub fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("receipt_digest", &self.receipt_digest),
            ("pipeline_id", &self.pipeline_id),
            ("provider", &self.provider),
            ("model", &self.model),
        ] {
            if value.trim().is_empty() {
                return Err(format!("{name} must not be empty"));
            }
            if value.len() > 512 {
                return Err(format!("{name} exceeds 512 bytes"));
            }
        }
        if self.schema_version != LLM_RECEIPT_METADATA_V1 {
            return Err("unsupported receipt metadata schema version".into());
        }
        if self
            .traceparent
            .as_ref()
            .is_some_and(|value| value.len() > 512)
        {
            return Err("traceparent exceeds 512 bytes".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_excludes_raw_receipt_body() {
        let metadata = LlmReceiptMetadataV1::new(
            "sha256:receipt",
            Some("00-trace-span-01".into()),
            "pipeline-1",
            "provider",
            "model",
            true,
        )
        .unwrap();
        let encoded = serde_json::to_value(&metadata).unwrap();
        assert!(encoded.get("receipt_json").is_none());
        assert!(metadata.integrity_verified);
    }

    #[test]
    fn metadata_rejects_empty_identity() {
        assert!(
            LlmReceiptMetadataV1::new("", None, "pipeline", "provider", "model", true).is_err()
        );
    }
}
