use crate::PolyKvError;
use quant_codec_core::{
    ArtifactDigest, DType, KvTensorShape, ModelFingerprint, TokenizerFingerprint,
};

/// Pool-ready, owned representation of a Transformers cache.
#[derive(Debug, Clone, PartialEq)]
pub struct PoolInput {
    pub model_fingerprint: ModelFingerprint,
    pub tokenizer_fingerprint: TokenizerFingerprint,
    pub config_digest: ArtifactDigest,
    pub shape: KvTensorShape,
    pub dtype: DType,
    pub layers: Vec<TransformersCacheLayer>,
    pub token_ids: Vec<u32>,
    pub position_ids: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransformersCacheLayer {
    pub layer_idx: u32,
    pub key_tensor: Vec<f32>,
    pub value_tensor: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransformersCacheBundle {
    pub model_fingerprint: ModelFingerprint,
    pub tokenizer_fingerprint: TokenizerFingerprint,
    pub revision: String,
    pub config_digest: ArtifactDigest,
    pub shape: KvTensorShape,
    pub dtype: DType,
    pub layers: Vec<TransformersCacheLayer>,
    pub token_ids: Vec<u32>,
    pub position_ids: Vec<u32>,
    pub seq_len: u32,
}

impl TransformersCacheBundle {
    pub fn into_pool_input(self) -> PoolInput {
        PoolInput {
            model_fingerprint: self.model_fingerprint,
            tokenizer_fingerprint: self.tokenizer_fingerprint,
            config_digest: self.config_digest,
            shape: self.shape,
            dtype: self.dtype,
            layers: self.layers,
            token_ids: self.token_ids,
            position_ids: self.position_ids,
        }
    }

    pub fn restore_dynamic_cache(&self) -> Vec<(Vec<f32>, Vec<f32>)> {
        let mut layers = self.layers.clone();
        layers.sort_by_key(|layer| layer.layer_idx);
        layers
            .into_iter()
            .map(|l| (l.key_tensor, l.value_tensor))
            .collect()
    }

    pub fn verify_shape_consistency(&self) -> Result<(), PolyKvError> {
        self.shape.validate()?;
        if self.seq_len as u64 != self.shape.seq_len {
            return Err(PolyKvError::ShapeMismatch {
                reason: "seq_len differs from tensor shape".into(),
            });
        }
        if self.layers.len() != self.shape.layers as usize {
            return Err(PolyKvError::ShapeMismatch {
                reason: "layer count differs from tensor shape".into(),
            });
        }
        if self.token_ids.len() != self.seq_len as usize
            || self.position_ids.len() != self.seq_len as usize
        {
            return Err(PolyKvError::ShapeMismatch {
                reason: "token/position ids differ from seq_len".into(),
            });
        }
        let key_len = self
            .shape
            .layer_element_count(quant_codec_core::KvRole::Key)?;
        let value_len = self
            .shape
            .layer_element_count(quant_codec_core::KvRole::Value)?;
        for (expected, layer) in self.layers.iter().enumerate() {
            if layer.layer_idx != expected as u32 {
                return Err(PolyKvError::ShapeMismatch {
                    reason: "layer indices are not contiguous".into(),
                });
            }
            if layer.key_tensor.len() != key_len || layer.value_tensor.len() != value_len {
                return Err(PolyKvError::ShapeMismatch {
                    reason: format!(
                        "layer {} tensor dimensions do not match heads/head_dim/seq_len",
                        layer.layer_idx
                    ),
                });
            }
        }
        Ok(())
    }
}
