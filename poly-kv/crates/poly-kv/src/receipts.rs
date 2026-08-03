use crate::{MemoryAccounting, QualityGateResultV1};
use quant_codec_core::{ArtifactDigest, EvalReport, KvRole, KvSliceRequest};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PoolBuildReceiptV1 {
    pub schema_version: u16,
    pub manifest_digest: ArtifactDigest,
    pub input_digest: ArtifactDigest,
    pub encoded_bytes: u64,
    pub exact_fallback_bytes: u64,
    pub block_count: u64,
    pub quality_gate: QualityGateResultV1,
    pub compression_evals: Vec<CompressionEvalReceiptV1>,
    pub memory: MemoryAccounting,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ReaderInjectionReceiptV1 {
    pub schema_version: u16,
    pub reader_id: u64,
    pub manifest_digest: ArtifactDigest,
    pub encoded_shared_bytes: u64,
    pub per_reader_scratch_bytes: u64,
    pub reader_count_after_attach: u64,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FallbackReceiptV1 {
    pub schema_version: u16,
    pub reason: String,
    pub role: KvRole,
    pub layer: u32,
    pub exact_bytes_read: u64,
    pub manifest_digest: ArtifactDigest,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DecodeReceiptV1 {
    pub schema_version: u16,
    pub request: KvSliceRequest,
    pub decoded_values: u64,
    pub full_block_decoded: bool,
    pub decoded_full_values: u64,
    pub returned_values: u64,
    pub copy_performed: bool,
    pub source_encoded_bytes: u64,
    pub scratch_bytes: u64,
    pub fallback: Option<FallbackReceiptV1>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CompressionEvalReceiptV1 {
    pub schema_version: u16,
    pub role: KvRole,
    pub layer: u32,
    pub ideal_codec_bits_per_scalar: Option<f32>,
    pub realized_encoded_bytes: u64,
    pub metadata_bytes: u64,
    pub eval: EvalReport,
}

pub const COMPRESSED_ATTENTION_SELECTION_RECEIPT_SCHEMA: u16 = 1;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CompressedAttentionSelectionReceipt {
    pub schema_version: u16,
    pub pool_id: String,
    pub layer_idx: u32,
    pub head_idx: u32,
    pub total_candidates: u32,
    pub selected_count: u32,
    pub full_layer_decoded: bool,
    pub decoded_value_vectors: u64,
    pub full_decode_value_count: u64,
    pub claim_boundary: String,
    pub codec_used: String,
    pub timestamp: u64,
}

impl CompressedAttentionSelectionReceipt {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != COMPRESSED_ATTENTION_SELECTION_RECEIPT_SCHEMA {
            return Err("unsupported schema version".into());
        }
        if self.selected_count > self.total_candidates {
            return Err("selected count exceeds candidates".into());
        }
        if self.full_layer_decoded && self.decoded_value_vectors < self.full_decode_value_count {
            return Err("full decode count is inconsistent".into());
        }
        if self.claim_boundary.trim().is_empty() || self.codec_used.trim().is_empty() {
            return Err("claim boundary and codec are required".into());
        }
        Ok(())
    }
}
