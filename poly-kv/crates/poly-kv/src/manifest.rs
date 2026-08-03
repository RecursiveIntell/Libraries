use quant_codec_core::{
    ArtifactDigest, CodecId, CodecProfileDigest, DType, KvRole, KvTensorShape, ModelFingerprint,
    TokenizerFingerprint,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QualityGateResultV1 {
    pub passed: bool,
    pub max_key_mse: f64,
    pub observed_key_mse: Option<f64>,
    pub max_value_mse: f64,
    pub observed_value_mse: Option<f64>,
    pub notes: Vec<String>,
}

impl QualityGateResultV1 {
    pub fn alpha_reference() -> Self {
        Self {
            passed: true,
            max_key_mse: 0.001,
            observed_key_mse: None,
            max_value_mse: 0.0,
            observed_value_mse: None,
            notes: vec!["synthetic alpha gate".to_string()],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CompressionPolicyV1 {
    pub profile_digest: CodecProfileDigest,
    pub key_codec_id: CodecId,
    pub value_codec_id: CodecId,
    pub lossy_keys: bool,
    pub lossy_values: bool,
    pub quality_gate: QualityGateResultV1,
}

impl CompressionPolicyV1 {
    pub fn alpha_reference() -> Self {
        Self {
            profile_digest: CodecProfileDigest::from_parts(&[
                b"poly-kv",
                b"0.1.0-alpha.1",
                b"alpha-reference-policy",
            ]),
            key_codec_id: CodecId::new("poly-kv:q8-key:symmetric-per-block")
                .expect("static codec id is valid"),
            value_codec_id: CodecId::new("poly-kv:value:raw-exact")
                .expect("static codec id is valid"),
            lossy_keys: true,
            lossy_values: false,
            quality_gate: QualityGateResultV1::alpha_reference(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlockManifestEntryV1 {
    pub role: KvRole,
    pub layer: u32,
    pub codec_id: CodecId,
    pub encoded_bytes: u64,
    pub exact_bytes: u64,
    pub ideal_codec_bits_per_scalar: Option<f32>,
    pub realized_encoded_bytes: u64,
    pub metadata_bytes: u64,
    pub artifact_digest: ArtifactDigest,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KvPoolManifestV1 {
    pub schema_version: u16,
    pub model_fingerprint: ModelFingerprint,
    pub tokenizer_fingerprint: TokenizerFingerprint,
    pub shape: KvTensorShape,
    pub source_dtype: DType,
    pub policy: CompressionPolicyV1,
    pub blocks: Vec<BlockManifestEntryV1>,
    pub encoded_bytes: u64,
    pub exact_fallback_bytes: u64,
    pub manifest_digest: ArtifactDigest,
}

impl KvPoolManifestV1 {
    pub fn canonical_digest_without_self(&self) -> ArtifactDigest {
        let parts = self.canonical_parts_without_self();
        let refs = parts.iter().map(String::as_bytes).collect::<Vec<_>>();
        ArtifactDigest::from_parts(&refs)
    }

    pub fn canonical_serialized_bytes(&self) -> Vec<u8> {
        let mut parts = self.canonical_parts_without_self();
        parts.push(format!("manifest_digest:{}", self.manifest_digest));
        parts.join("\n").into_bytes()
    }

    pub fn canonical_serialized_len(&self) -> u64 {
        self.canonical_serialized_bytes().len() as u64
    }

    fn canonical_parts_without_self(&self) -> Vec<String> {
        let mut parts = Vec::new();
        parts.push(format!("schema:{}", self.schema_version));
        parts.push(format!("model:{}", self.model_fingerprint));
        parts.push(format!("tokenizer:{}", self.tokenizer_fingerprint));
        parts.push(format!(
            "shape:{}:{}:{}:{}:{}:{:?}:{:?}",
            self.shape.layers,
            self.shape.key_heads,
            self.shape.value_heads,
            self.shape.seq_len,
            self.shape.head_dim,
            self.shape.layout,
            self.shape.dtype
        ));
        parts.push(format!("source_dtype:{:?}", self.source_dtype));
        parts.push(format!("policy:{}", self.policy.profile_digest));
        parts.push(format!("encoded:{}", self.encoded_bytes));
        parts.push(format!("fallback:{}", self.exact_fallback_bytes));
        for block in &self.blocks {
            parts.push(format!(
                "block:{:?}:{}:{}:{}:{}:{:?}:{}:{}:{}",
                block.role,
                block.layer,
                block.codec_id,
                block.encoded_bytes,
                block.exact_bytes,
                block.ideal_codec_bits_per_scalar,
                block.realized_encoded_bytes,
                block.metadata_bytes,
                block.artifact_digest
            ));
        }
        parts
    }
}
