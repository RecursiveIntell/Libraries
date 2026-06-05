//! Pool-backed vector codec: stores embeddings in a compressed SharedKVPool.
//!
//! This codec adapts poly-kv's multi-agent KV-cache pool for semantic-memory's
//! per-vector VectorCodec interface. The adaptation treats N embedding vectors
//! as N "tokens" in a single-layer, single-head model, so SharedKVPool's two-tier
//! compression (fib-quant cold + turbo-quant hot) applies transparently.
//!
//! ## Two modes
//!
//! 1. **Batch mode** (`PoolCodec::new`): Build a pool from a corpus of embeddings.
//!    Decode individual vectors by index via `decode_at()`.
//!
//! 2. **Per-vector mode** (via `VectorCodec` trait): `encode()` compresses a single
//!    vector using fib-quant through a single-token pool; `decode()` decompresses
//!    from the artifact's own compressed bytes. This is less efficient than batch
//!    mode but satisfies the `VectorCodec` trait contract.

#[cfg(feature = "poly-kv-pool")]
use crate::vector_codec::{VectorArtifactV1, VectorCodec, VectorCodecProfileV1};
#[cfg(feature = "poly-kv-pool")]
use crate::{db, MemoryError};

#[cfg(feature = "poly-kv-pool")]
use poly_kv_core::{AttentionType, KvTensorShape, PoolBuildReceipt, SharedKVPool};

const POOL_CODEC_SCHEMA_V1: &str = "vector_codec_profile_v1";

/// A VectorCodec that stores embeddings in a compressed SharedKVPool.
///
/// The pool is built once from a corpus of embeddings. Individual vectors
/// can then be decoded by index via [`decode_at()`](Self::decode_at), or
/// encoded/decoded one at a time via the `VectorCodec` trait.
#[cfg(feature = "poly-kv-pool")]
#[derive(Debug, Clone)]
pub struct PoolCodec {
    profile: VectorCodecProfileV1,
    pool: SharedKVPool,
    build_receipt: PoolBuildReceipt,
    seed: u64,
}

#[cfg(feature = "poly-kv-pool")]
impl PoolCodec {
    /// Build a PoolCodec from a corpus of embedding vectors.
    ///
    /// All vectors must have the same dimensionality (`dim`). The pool
    /// treats them as N tokens in a single-layer, single-head model.
    pub fn new(dim: usize, corpus: &[(String, Vec<f32>)], seed: u64) -> Result<Self, MemoryError> {
        // Single-layer, single-head model. Both key and value carry the
        // same embedding (fib-quant requires non-zero-norm vectors).
        let shape = KvTensorShape {
            attention_type: AttentionType::MHA,
            num_layers: 1,
            num_heads: 1,
            num_kv_heads: 1,
            head_dim: dim,
            hidden_size: dim,
        };

        let pool_corpus: Vec<(String, Vec<f32>)> = corpus
            .iter()
            .map(|(id, vec)| {
                let mut kv = Vec::with_capacity(dim * 2);
                kv.extend_from_slice(vec);
                kv.extend_from_slice(vec); // value = same embedding (fib-quant needs non-zero norm)
                (id.clone(), kv)
            })
            .collect();

        let (pool, receipt) = SharedKVPool::build(&pool_corpus, &shape, seed)
            .map_err(|e| MemoryError::QuantizationError(format!("pool build: {e}")))?;

        let profile = VectorCodecProfileV1 {
            schema_version: POOL_CODEC_SCHEMA_V1.into(),
            codec: "shared_kv_pool".into(),
            dim: u32::try_from(dim).map_err(|_| MemoryError::InvalidConfig {
                field: "embedding.dimensions",
                reason: format!("dimension {dim} does not fit vector codec profile u32"),
            })?,
            bits: 4, // fib-quant k=4 effective bits
            projections: None,
            seed: Some(seed),
            codec_version: "poly-kv:0.1.0".into(),
            scoring_semantics: "cosine_on_decompressed_f32".into(),
            normalization: "caller_supplied".into(),
        };

        Ok(Self {
            profile,
            pool,
            build_receipt: receipt,
            seed,
        })
    }

    /// Reference to the underlying SharedKVPool.
    pub fn pool(&self) -> &SharedKVPool {
        &self.pool
    }

    /// Reference to the pool build receipt.
    pub fn build_receipt(&self) -> &PoolBuildReceipt {
        &self.build_receipt
    }

    /// Decompress the entire pool and decode a single vector by position index.
    ///
    /// This is the efficient path: decompress the pool once, then index into
    /// the result. Use this for batch recall/precision evaluation where you
    /// need every vector.
    pub fn decode_at(&self, index: usize) -> Result<Vec<f32>, MemoryError> {
        let dim = self.profile.dim as usize;
        let layer = self
            .pool
            .decompress_layer(0)
            .map_err(|e| MemoryError::QuantizationError(format!("pool decode_at: {e}")))?;

        if layer.keys.is_empty() {
            return Err(MemoryError::QuantizationError(
                "pool decode_at: no keys".into(),
            ));
        }

        let all_keys = &layer.keys[0]; // single head: keys[0] = all tokens concatenated
        if index >= layer.num_tokens {
            return Err(MemoryError::QuantizationError(format!(
                "pool decode_at: index {index} >= num_tokens {}",
                layer.num_tokens
            )));
        }

        let start = index * dim;
        let end = start + dim;
        if end > all_keys.len() {
            return Err(MemoryError::QuantizationError(format!(
                "pool decode_at: key buffer too short (need {end}, have {})",
                all_keys.len()
            )));
        }

        let result = all_keys[start..end].to_vec();
        db::validate_embedding(&result, dim)?;
        Ok(result)
    }

    /// Decompress the entire pool and return all vectors.
    ///
    /// Useful for bulk recall@k evaluation.
    pub fn decode_all(&self) -> Result<Vec<Vec<f32>>, MemoryError> {
        let dim = self.profile.dim as usize;
        let layer = self
            .pool
            .decompress_layer(0)
            .map_err(|e| MemoryError::QuantizationError(format!("pool decode_all: {e}")))?;

        if layer.keys.is_empty() {
            return Ok(Vec::new());
        }

        let all_keys = &layer.keys[0];
        let num_tokens = layer.num_tokens;
        let mut result = Vec::with_capacity(num_tokens);
        for i in 0..num_tokens {
            let start = i * dim;
            let end = start + dim;
            if end > all_keys.len() {
                return Err(MemoryError::QuantizationError(format!(
                    "pool decode_all: key buffer too short at token {i}"
                )));
            }
            let vec = all_keys[start..end].to_vec();
            result.push(vec);
        }

        Ok(result)
    }
}

/// Build a single-embedding kv_vector for pool ingestion.
///
/// Duplicates the embedding for both key and value slots to satisfy
/// fib-quant's non-zero-norm requirement.
#[cfg(feature = "poly-kv-pool")]
fn kv_vector_from_embedding(embedding: &[f32]) -> Vec<f32> {
    let mut kv = Vec::with_capacity(embedding.len() * 2);
    kv.extend_from_slice(embedding);
    kv.extend_from_slice(embedding); // value = same (avoids zero-norm rejection)
    kv
}

#[cfg(feature = "poly-kv-pool")]
impl VectorCodec for PoolCodec {
    fn profile(&self) -> &VectorCodecProfileV1 {
        &self.profile
    }

    fn encode(&self, vector: &[f32]) -> Result<VectorArtifactV1, MemoryError> {
        db::validate_embedding(vector, self.profile.dim as usize)?;
        let dim = self.profile.dim as usize;
        let kv = kv_vector_from_embedding(vector);
        let shape = KvTensorShape {
            attention_type: AttentionType::MHA,
            num_layers: 1,
            num_heads: 1,
            num_kv_heads: 1,
            head_dim: dim,
            hidden_size: dim,
        };
        let (mini_pool, _receipt) =
            SharedKVPool::build(&[("single".into(), kv)], &shape, self.seed)
                .map_err(|e| MemoryError::QuantizationError(format!("pool encode: {e}")))?;

        // Extract the compressed bytes from layer 0
        let encoded = if let Some(layer) = mini_pool.layers.first() {
            let mut buf = Vec::new();
            for block in &layer.key_blocks {
                buf.extend_from_slice(&block.encoded_payload);
            }
            buf
        } else {
            return Err(MemoryError::QuantizationError(
                "pool encode: no layers".into(),
            ));
        };

        Ok(VectorArtifactV1::new(self.profile.clone(), encoded))
    }

    fn decode(&self, artifact: &VectorArtifactV1) -> Result<Vec<f32>, MemoryError> {
        // The artifact stores the compressed key payload from a single-token
        // mini-pool. Rebuild a mini-pool from the artifact bytes and decompress.
        let dim = self.profile.dim as usize;
        // Build a minimal CompressedBlock-like structure to feed back through decode.
        // The artifact.encoded is the raw fib-quant key payload from a single head.
        // We need to reconstruct a single-block layer and decompress it.
        //
        // Easier path: build a single-token pool using the same shape/seed, then
        // read from it. But we don't have the original vector — only its compressed form.
        //
        // Simplest correct path: the artifact.encoded bytes ARE the fib-quant compressed
        // payload for one key block. We can decode it directly through the poly-kv codec.
        let codec = poly_kv_core::create_codec(
            &poly_kv_core::CODEC_FIB_K4_N32,
            dim,
            Some(&self.pool.manifest.policy.fib_config),
            None,
        )
        .map_err(|e| MemoryError::QuantizationError(format!("pool decode codec: {e}")))?;

        let decoded = if let Some(mut batch) = codec
            .decode_batch_compact(&artifact.encoded, self.seed)
            .map_err(|e| MemoryError::QuantizationError(format!("pool decode compact: {e}")))?
        {
            if batch.len() != 1 {
                return Err(MemoryError::QuantizationError(format!(
                    "pool decode compact: expected 1 vector, got {}",
                    batch.len()
                )));
            }
            batch.remove(0)
        } else {
            codec
                .decode(&artifact.encoded, self.seed)
                .map_err(|e| MemoryError::QuantizationError(format!("pool decode: {e}")))?
        };

        if decoded.len() != dim {
            return Err(MemoryError::QuantizationError(format!(
                "pool decode: decoded length {} != dim {}",
                decoded.len(),
                dim
            )));
        }

        db::validate_embedding(&decoded, dim)?;
        Ok(decoded)
    }
}
