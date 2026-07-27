//! Vector codec profile and artifact boundary.
//!
//! Codecs are derived acceleration/transport helpers only. SQLite memory rows
//! and raw f32 embeddings remain authoritative and every artifact must carry a
//! profile digest that can be checked before decoding.

use crate::{db, quantize, MemoryError};
use serde::{Deserialize, Serialize};
use stack_ids::{ContentDigest, DigestBuilder};

const PROFILE_SCHEMA_V1: &str = "vector_codec_profile_v1";
const ARTIFACT_SCHEMA_V1: &str = "vector_artifact_v1";

fn b3_digest(bytes: &[u8]) -> String {
    format!("blake3:{}", ContentDigest::compute(bytes).hex())
}

fn b3_json_digest<T: Serialize>(domain: &str, value: &T) -> String {
    let mut builder = DigestBuilder::new();
    builder.update_str(domain).separator();
    match builder.update_json(value) {
        Ok(_) => format!("blake3:{}", builder.finalize().hex()),
        Err(_) => b3_digest(format!("{domain}:digest-fallback").as_bytes()),
    }
}

fn dim_u32(dim: usize) -> Result<u32, MemoryError> {
    u32::try_from(dim).map_err(|_| MemoryError::InvalidConfig {
        field: "embedding.dimensions",
        reason: format!("dimension {dim} does not fit vector codec profile u32"),
    })
}

fn dim_usize(dim: u32) -> usize {
    dim as usize
}

/// Stable identity for a vector codec configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorCodecProfileV1 {
    /// Stable schema marker for this profile format.
    pub schema_version: String,
    /// Codec implementation identifier.
    pub codec: String,
    /// Vector dimensionality. Persisted as fixed-width `u32`, never `usize`.
    pub dim: u32,
    /// Effective encoded bits per scalar, when applicable.
    pub bits: u8,
    /// Number of projection vectors, reserved for future derived codecs.
    pub projections: Option<u32>,
    /// Codec seed, reserved for randomized derived codecs.
    pub seed: Option<u64>,
    /// Codec wire/algorithm version.
    pub codec_version: String,
    /// Declared scoring semantics for decoded/reference comparison.
    pub scoring_semantics: String,
    /// Declared vector normalization contract.
    pub normalization: String,
}

impl VectorCodecProfileV1 {
    /// Build the reference raw f32 profile.
    pub fn raw_f32(dim: usize) -> Result<Self, MemoryError> {
        Ok(Self {
            schema_version: PROFILE_SCHEMA_V1.into(),
            codec: "raw_f32".into(),
            dim: dim_u32(dim)?,
            bits: 32,
            projections: None,
            seed: None,
            codec_version: "1".into(),
            scoring_semantics: "cosine_on_decoded_f32".into(),
            normalization: "caller_supplied".into(),
        })
    }

    /// Build the existing per-vector scalar quantization profile.
    pub fn sq8(dim: usize) -> Result<Self, MemoryError> {
        Ok(Self {
            schema_version: PROFILE_SCHEMA_V1.into(),
            codec: "sq8".into(),
            dim: dim_u32(dim)?,
            bits: 8,
            projections: None,
            seed: None,
            codec_version: "1".into(),
            scoring_semantics: "cosine_on_dequantized_f32".into(),
            normalization: "caller_supplied".into(),
        })
    }

    /// Build a TurboQuant profile.
    #[cfg(feature = "turbo-quant-codec")]
    pub fn turbo_quant(
        dim: usize,
        bits: u8,
        projections: usize,
        seed: u64,
    ) -> Result<Self, MemoryError> {
        Ok(Self {
            schema_version: PROFILE_SCHEMA_V1.into(),
            codec: "turbo_quant".into(),
            dim: dim_u32(dim)?,
            bits,
            projections: Some(dim_u32(projections)?),
            seed: Some(seed),
            codec_version: "turbo-quant:0.2.0-alpha.1".into(),
            scoring_semantics: "inner_product_estimate".into(),
            normalization: "caller_supplied".into(),
        })
    }

    /// Build a FibQuant profile.
    #[cfg(feature = "fib-quant-codec")]
    pub fn fib_quant(dim: usize) -> Result<Self, MemoryError> {
        Ok(Self {
            schema_version: PROFILE_SCHEMA_V1.into(),
            codec: "fib_quant".into(),
            dim: dim_u32(dim)?,
            bits: 8,
            projections: None,
            seed: None,
            codec_version: "fib-quant:0.1.0-beta.3".into(),
            scoring_semantics: "inner_product_estimate".into(),
            normalization: "caller_supplied".into(),
        })
    }

    /// Stable digest over the explicit profile fields.
    pub fn digest(&self) -> String {
        b3_json_digest(PROFILE_SCHEMA_V1, self)
    }
}

/// Persistable encoded vector plus the profile identity required to decode it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorArtifactV1 {
    /// Stable schema marker for this artifact format.
    pub schema_version: String,
    /// Full codec profile used to produce `encoded`.
    pub profile: VectorCodecProfileV1,
    /// Digest of `profile`; checked before decode.
    pub profile_digest: String,
    /// Digest of `encoded`; checked before decode when present.
    #[serde(default)]
    pub artifact_digest: String,
    /// Codec-owned encoded bytes.
    pub encoded: Vec<u8>,
}

impl VectorArtifactV1 {
    /// Construct a new artifact and stamp the profile digest.
    pub fn new(profile: VectorCodecProfileV1, encoded: Vec<u8>) -> Self {
        let profile_digest = profile.digest();
        let artifact_digest = b3_digest(&encoded);
        Self {
            schema_version: ARTIFACT_SCHEMA_V1.into(),
            profile,
            profile_digest,
            artifact_digest,
            encoded,
        }
    }

    /// Stable digest over encoded artifact bytes.
    pub fn encoded_digest(&self) -> String {
        b3_digest(&self.encoded)
    }
}

/// INT-001: Object-safe vector codec boundary for derived vector artifacts.
///
/// This trait is the semantic-memory consumer-facing boundary. It should
/// eventually be replaced by direct consumption of the canonical
/// `quant_codec_core::VectorCodec` trait. Until then, implementations of
/// this trait should also implement the canonical trait where practical.
pub trait VectorCodec: Send + Sync {
    /// Codec profile identity.
    fn profile(&self) -> &VectorCodecProfileV1;

    /// INT-001: Codec capabilities — what operations this codec supports.
    fn capabilities(&self) -> CodecCapabilityInfo {
        CodecCapabilityInfo::default()
    }

    /// Encode a raw f32 vector into a byte artifact.
    fn encode(&self, vector: &[f32]) -> Result<VectorArtifactV1, MemoryError>;

    /// Decode an artifact back to f32 for reference scoring or differential tests.
    fn decode(&self, artifact: &VectorArtifactV1) -> Result<Vec<f32>, MemoryError>;
}

/// INT-001: Codec capability information for semantic-memory consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CodecCapabilityInfo {
    /// Codec can estimate inner product from compressed form.
    pub can_score_inner_product: bool,
    /// Codec is lossless (raw f32 representation).
    pub is_lossless: bool,
}

fn validate_artifact_profile(
    expected: &VectorCodecProfileV1,
    artifact: &VectorArtifactV1,
) -> Result<(), MemoryError> {
    let artifact_profile_digest = artifact.profile.digest();
    if artifact.profile_digest != artifact_profile_digest {
        return Err(MemoryError::VectorCodecProfileMismatch {
            expected_digest: artifact_profile_digest,
            actual_digest: artifact.profile_digest.clone(),
        });
    }

    let expected_digest = expected.digest();
    if artifact.profile_digest != expected_digest {
        return Err(MemoryError::VectorCodecProfileMismatch {
            expected_digest,
            actual_digest: artifact.profile_digest.clone(),
        });
    }

    let encoded_digest = artifact.encoded_digest();
    if !artifact.artifact_digest.is_empty() && artifact.artifact_digest != encoded_digest {
        return Err(MemoryError::CorruptData {
            table: "vector_artifacts",
            row_id: artifact.profile_digest.clone(),
            detail: format!(
                "encoded artifact digest mismatch: expected {}, got {}",
                artifact.artifact_digest, encoded_digest
            ),
        });
    }

    Ok(())
}

/// Reference codec that stores raw little-endian f32 bytes.
#[derive(Debug, Clone)]
pub struct RawF32Codec {
    profile: VectorCodecProfileV1,
}

impl RawF32Codec {
    /// Create a raw f32 codec for `dim` dimensions.
    pub fn new(dim: usize) -> Result<Self, MemoryError> {
        Ok(Self {
            profile: VectorCodecProfileV1::raw_f32(dim)?,
        })
    }
}

impl VectorCodec for RawF32Codec {
    fn profile(&self) -> &VectorCodecProfileV1 {
        &self.profile
    }

    fn capabilities(&self) -> CodecCapabilityInfo {
        CodecCapabilityInfo {
            can_score_inner_product: false,
            is_lossless: true,
        }
    }

    fn encode(&self, vector: &[f32]) -> Result<VectorArtifactV1, MemoryError> {
        db::validate_embedding(vector, dim_usize(self.profile.dim))?;
        Ok(VectorArtifactV1::new(
            self.profile.clone(),
            db::encode_f32_le(vector),
        ))
    }

    fn decode(&self, artifact: &VectorArtifactV1) -> Result<Vec<f32>, MemoryError> {
        validate_artifact_profile(&self.profile, artifact)?;
        db::decode_f32_le(&artifact.encoded, dim_usize(self.profile.dim))
    }
}

/// Codec wrapper around the existing per-vector SQ8 quantization path.
#[derive(Debug, Clone)]
pub struct Sq8Codec {
    profile: VectorCodecProfileV1,
}

impl Sq8Codec {
    /// Create an SQ8 codec for `dim` dimensions.
    pub fn new(dim: usize) -> Result<Self, MemoryError> {
        Ok(Self {
            profile: VectorCodecProfileV1::sq8(dim)?,
        })
    }
}

impl VectorCodec for Sq8Codec {
    fn profile(&self) -> &VectorCodecProfileV1 {
        &self.profile
    }

    fn capabilities(&self) -> CodecCapabilityInfo {
        CodecCapabilityInfo {
            can_score_inner_product: false,
            is_lossless: false,
        }
    }

    fn encode(&self, vector: &[f32]) -> Result<VectorArtifactV1, MemoryError> {
        db::validate_embedding(vector, dim_usize(self.profile.dim))?;
        let quantized = quantize::Quantizer::new(dim_usize(self.profile.dim)).quantize(vector)?;
        Ok(VectorArtifactV1::new(
            self.profile.clone(),
            quantize::pack_quantized(&quantized),
        ))
    }

    fn decode(&self, artifact: &VectorArtifactV1) -> Result<Vec<f32>, MemoryError> {
        validate_artifact_profile(&self.profile, artifact)?;
        let quantized = quantize::unpack_quantized(&artifact.encoded, dim_usize(self.profile.dim))?;
        let decoded = quantize::Quantizer::new(dim_usize(self.profile.dim)).dequantize(&quantized);
        db::validate_embedding(&decoded, dim_usize(self.profile.dim))?;
        Ok(decoded)
    }
}

#[cfg(feature = "per-dim-codec")]
#[derive(Debug, Clone)]
pub struct PerDimCodec {
    profile: VectorCodecProfileV1,
    scorer: compressed_scorer::PerDimScorer,
}

#[cfg(feature = "per-dim-codec")]
impl PerDimCodec {
    pub fn new(dim: usize, bits: u8) -> Result<Self, MemoryError> {
        let scorer = compressed_scorer::PerDimScorer::new(dim, bits as u32)
            .map_err(|e| MemoryError::QuantizationError(format!("per-dim: {e}")))?;
        Ok(Self {
            profile: VectorCodecProfileV1 {
                schema_version: PROFILE_SCHEMA_V1.into(),
                codec: "per_dim".into(),
                dim: dim_u32(dim)?,
                bits,
                projections: None,
                seed: None,
                codec_version: "compressed-scorer:0.1.0".into(),
                scoring_semantics: "inner_product_estimate".into(),
                normalization: "caller_supplied".into(),
            },
            scorer,
        })
    }
    pub fn profile(&self) -> &VectorCodecProfileV1 {
        &self.profile
    }
    pub fn config_json(&self) -> Result<String, MemoryError> {
        serde_json::to_string(
            &serde_json::json!({"dim": self.profile.dim, "bits": self.profile.bits}),
        )
        .map_err(|e| MemoryError::Other(format!("serialize per-dim config: {e}")))
    }

    pub fn prepare_query(
        &self,
        query: &[f32],
    ) -> Result<compressed_scorer::PerDimPrepared, MemoryError> {
        use compressed_scorer::CompressedScorer;
        self.scorer
            .prepare_query(query)
            .map_err(|e| MemoryError::QuantizationError(format!("per-dim: {e}")))
    }
    pub fn score_inner_product_prepared(
        &self,
        artifact: &VectorArtifactV1,
        prepared: &compressed_scorer::PerDimPrepared,
    ) -> Result<f32, MemoryError> {
        use compressed_scorer::CompressedScorer;
        validate_artifact_profile(&self.profile, artifact)?;
        let c = self.decode_compressed(artifact)?;
        self.scorer
            .score_prepared(prepared, &c)
            .map_err(|e| MemoryError::QuantizationError(format!("per-dim: {e}")))
    }
    fn decode_compressed(
        &self,
        artifact: &VectorArtifactV1,
    ) -> Result<compressed_scorer::PerDimCompressed, MemoryError> {
        validate_artifact_profile(&self.profile, artifact)?;
        if artifact.encoded.len() < 4 {
            return Err(MemoryError::CorruptData {
                table: "vector_artifacts",
                row_id: artifact.profile_digest.clone(),
                detail: "per-dim artifact too short".into(),
            });
        }
        Ok(compressed_scorer::PerDimCompressed {
            norm: f32::from_le_bytes(artifact.encoded[..4].try_into().unwrap()),
            codes: artifact.encoded[4..].to_vec(),
        })
    }
}

#[cfg(feature = "per-dim-codec")]
impl VectorCodec for PerDimCodec {
    fn profile(&self) -> &VectorCodecProfileV1 {
        &self.profile
    }
    fn capabilities(&self) -> CodecCapabilityInfo {
        CodecCapabilityInfo {
            can_score_inner_product: true,
            is_lossless: false,
        }
    }
    fn encode(&self, vector: &[f32]) -> Result<VectorArtifactV1, MemoryError> {
        use compressed_scorer::CompressedScorer;
        db::validate_embedding(vector, dim_usize(self.profile.dim))?;
        let c = self
            .scorer
            .compress(vector)
            .map_err(|e| MemoryError::QuantizationError(format!("per-dim: {e}")))?;
        let mut bytes = c.norm.to_le_bytes().to_vec();
        bytes.extend_from_slice(&c.codes);
        Ok(VectorArtifactV1::new(self.profile.clone(), bytes))
    }
    fn decode(&self, artifact: &VectorArtifactV1) -> Result<Vec<f32>, MemoryError> {
        use compressed_scorer::CompressedScorer;
        self.scorer
            .decode(&self.decode_compressed(artifact)?)
            .map_err(|e| MemoryError::QuantizationError(format!("per-dim: {e}")))
    }
}

#[cfg(feature = "turbo-quant-codec")]
fn map_turbo_error(err: turbo_quant::TurboQuantError) -> MemoryError {
    MemoryError::QuantizationError(format!("turbo-quant: {err}"))
}

/// Optional TurboQuant codec backend.
#[cfg(feature = "turbo-quant-codec")]
#[derive(Debug, Clone)]
pub struct TurboQuantCodec {
    profile: VectorCodecProfileV1,
    quantizer: turbo_quant::TurboQuantizer,
}

#[cfg(feature = "turbo-quant-codec")]
impl TurboQuantCodec {
    /// Create a TurboQuant codec profile.
    pub fn new(dim: usize, bits: u8, projections: usize, seed: u64) -> Result<Self, MemoryError> {
        let quantizer = turbo_quant::TurboQuantizer::new(dim, bits, projections, seed)
            .map_err(map_turbo_error)?;
        Ok(Self {
            profile: VectorCodecProfileV1::turbo_quant(dim, bits, projections, seed)?,
            quantizer,
        })
    }

    fn decode_code(
        &self,
        artifact: &VectorArtifactV1,
    ) -> Result<turbo_quant::TurboCode, MemoryError> {
        validate_artifact_profile(&self.profile, artifact)?;
        self.quantizer
            .decode_code_from_bytes(&artifact.encoded)
            .map_err(map_turbo_error)
    }

    /// Estimate inner product between a raw query vector and a TurboQuant artifact.
    pub fn score_inner_product(
        &self,
        artifact: &VectorArtifactV1,
        query: &[f32],
    ) -> Result<f32, MemoryError> {
        db::validate_embedding(query, dim_usize(self.profile.dim))?;
        validate_artifact_profile(&self.profile, artifact)?;
        self.quantizer
            .score_inner_product_from_bytes(&artifact.encoded, query)
            .map_err(map_turbo_error)
    }

    /// Prepare a query once for scoring many TurboQuant artifacts.
    pub fn prepare_query(
        &self,
        query: &[f32],
    ) -> Result<turbo_quant::TurboProjectedQuery, MemoryError> {
        db::validate_embedding(query, dim_usize(self.profile.dim))?;
        self.quantizer.prepare_query(query).map_err(map_turbo_error)
    }

    /// Estimate inner product using a pre-projected query.
    pub fn score_inner_product_prepared(
        &self,
        artifact: &VectorArtifactV1,
        prepared: &turbo_quant::TurboProjectedQuery,
    ) -> Result<f32, MemoryError> {
        validate_artifact_profile(&self.profile, artifact)?;
        let code = self.decode_code(artifact)?;
        self.quantizer
            .inner_product_estimate_prepared(&code, prepared)
            .map_err(map_turbo_error)
    }

    /// Estimate squared L2 distance between a raw query vector and a TurboQuant artifact.
    pub fn score_l2(&self, artifact: &VectorArtifactV1, query: &[f32]) -> Result<f32, MemoryError> {
        db::validate_embedding(query, dim_usize(self.profile.dim))?;
        validate_artifact_profile(&self.profile, artifact)?;
        let code = self.decode_code(artifact)?;
        self.quantizer
            .l2_distance_estimate(&code, query)
            .map_err(map_turbo_error)
    }
}

#[cfg(feature = "turbo-quant-codec")]
impl VectorCodec for TurboQuantCodec {
    fn profile(&self) -> &VectorCodecProfileV1 {
        &self.profile
    }

    fn capabilities(&self) -> CodecCapabilityInfo {
        CodecCapabilityInfo {
            can_score_inner_product: true,
            is_lossless: false,
        }
    }

    fn encode(&self, vector: &[f32]) -> Result<VectorArtifactV1, MemoryError> {
        db::validate_embedding(vector, dim_usize(self.profile.dim))?;
        let encoded = self
            .quantizer
            .encode_to_bytes(vector)
            .map_err(map_turbo_error)?;
        Ok(VectorArtifactV1::new(self.profile.clone(), encoded))
    }

    fn decode(&self, artifact: &VectorArtifactV1) -> Result<Vec<f32>, MemoryError> {
        let code = self.decode_code(artifact)?;
        let decoded = self
            .quantizer
            .decode_approximate(&code)
            .map_err(map_turbo_error)?;
        db::validate_embedding(&decoded, dim_usize(self.profile.dim))?;
        Ok(decoded)
    }
}

/// A query prepared for repeated FibQuant scoring.
#[cfg(feature = "fib-quant-codec")]
#[derive(Debug, Clone)]
pub struct FibQuantPreparedQuery {
    vector: Vec<f32>,
}

/// Optional FibQuant codec backend.
#[cfg(feature = "fib-quant-codec")]
#[derive(Debug, Clone)]
pub struct FibQuantCodec {
    profile: VectorCodecProfileV1,
    quantizer: fib_quant::FibQuantizer,
}

#[cfg(feature = "fib-quant-codec")]
fn map_fib_error(err: fib_quant::FibQuantError) -> MemoryError {
    MemoryError::QuantizationError(format!("fib-quant: {err}"))
}

#[cfg(feature = "fib-quant-codec")]
impl FibQuantCodec {
    pub fn codebook(&self) -> &fib_quant::FibCodebookV1 {
        self.quantizer.codebook()
    }

    pub fn gram_scorer(&self) -> Result<crate::scoring::fib_scorer::FibGramScorer, MemoryError> {
        crate::scoring::fib_scorer::FibGramScorer::from_quantizer(&self.quantizer)
    }

    pub fn encode_code(&self, vector: &[f32]) -> Result<fib_quant::FibCodeV1, MemoryError> {
        db::validate_embedding(vector, dim_usize(self.profile.dim))?;
        self.quantizer.encode(vector).map_err(map_fib_error)
    }

    pub fn code_from_artifact(
        &self,
        artifact: &VectorArtifactV1,
    ) -> Result<fib_quant::FibCodeV1, MemoryError> {
        validate_artifact_profile(&self.profile, artifact)?;
        serde_json::from_slice(&artifact.encoded)
            .map_err(|e| MemoryError::QuantizationError(format!("fib-quant decode: {e}")))
    }

    pub fn from_codebook(codebook: fib_quant::FibCodebookV1) -> Result<Self, MemoryError> {
        let quantizer = fib_quant::FibQuantizer::from_codebook(codebook).map_err(map_fib_error)?;
        let dim = quantizer.profile().ambient_dim as usize;
        Ok(Self {
            profile: VectorCodecProfileV1::fib_quant(dim)?,
            quantizer,
        })
    }

    pub fn new(
        dim: usize,
        block_count: usize,
        gram_table_size: usize,
    ) -> Result<Self, MemoryError> {
        let fib_profile =
            fib_quant::FibQuantProfileV1::paper_default(dim, block_count, gram_table_size, 0)
                .map_err(map_fib_error)?;
        let quantizer = fib_quant::FibQuantizer::new(fib_profile).map_err(map_fib_error)?;
        Ok(Self {
            profile: VectorCodecProfileV1::fib_quant(dim)?,
            quantizer,
        })
    }

    pub fn prepare_query(&self, query: &[f32]) -> Result<FibQuantPreparedQuery, MemoryError> {
        db::validate_embedding(query, dim_usize(self.profile.dim))?;
        Ok(FibQuantPreparedQuery {
            vector: query.to_vec(),
        })
    }

    pub fn score_inner_product_prepared(
        &self,
        artifact: &VectorArtifactV1,
        prepared: &FibQuantPreparedQuery,
    ) -> Result<f32, MemoryError> {
        let decoded = self.decode(artifact)?;
        let score = fib_quant::metrics::cosine_similarity(&prepared.vector, &decoded)
            .map_err(map_fib_error)?;
        Ok(score as f32)
    }

    pub fn score_inner_product(
        &self,
        artifact: &VectorArtifactV1,
        query: &[f32],
    ) -> Result<f32, MemoryError> {
        let prepared = self.prepare_query(query)?;
        self.score_inner_product_prepared(artifact, &prepared)
    }
}

#[cfg(feature = "fib-quant-codec")]
impl VectorCodec for FibQuantCodec {
    fn profile(&self) -> &VectorCodecProfileV1 {
        &self.profile
    }
    fn capabilities(&self) -> CodecCapabilityInfo {
        CodecCapabilityInfo {
            can_score_inner_product: true,
            is_lossless: false,
        }
    }

    fn encode(&self, vector: &[f32]) -> Result<VectorArtifactV1, MemoryError> {
        db::validate_embedding(vector, dim_usize(self.profile.dim))?;
        let code = self.encode_code(vector)?;
        let encoded = serde_json::to_vec(&code)
            .map_err(|e| MemoryError::QuantizationError(format!("fib-quant encode: {e}")))?;
        Ok(VectorArtifactV1::new(self.profile.clone(), encoded))
    }

    fn decode(&self, artifact: &VectorArtifactV1) -> Result<Vec<f32>, MemoryError> {
        validate_artifact_profile(&self.profile, artifact)?;
        let code = self.code_from_artifact(artifact)?;
        let decoded = self.quantizer.decode(&code).map_err(map_fib_error)?;
        db::validate_embedding(&decoded, dim_usize(self.profile.dim))?;
        Ok(decoded)
    }
}
