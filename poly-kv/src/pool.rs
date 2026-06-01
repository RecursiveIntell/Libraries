use std::time::Instant;

use crate::codec::{create_codec, CompressedBlock};
use crate::error::{PolyKvError, Result};
use crate::manifest::PoolManifest;
use crate::policy::{CompressionPolicy, CODEC_FIB_K4_N32};
use crate::receipt::{now_unix, PoolBuildReceipt};
use crate::shape::KvTensorShape;

/// One layer's worth of compressed KV blocks in the shared pool.
#[derive(Debug, Clone)]
pub struct PoolLayer {
    /// Zero-based layer index.
    pub layer_index: u32,
    /// Key blocks — one per token, fib-quant compressed.
    pub key_blocks: Vec<CompressedBlock>,
    /// Value blocks — one per token, fib-quant compressed.
    pub value_blocks: Vec<CompressedBlock>,
    /// Blake3 digest of all blocks in this layer (canonical JSON).
    pub block_digest: String,
}

impl PoolLayer {
    /// Compute a content digest over the blocks in this layer.
    fn compute_digest(&self) -> Result<String> {
        // Serialize key + value payloads to compute a deterministic digest
        let key_digests: Vec<&str> = self
            .key_blocks
            .iter()
            .map(|b| b.payload_digest.as_str())
            .collect();
        let value_digests: Vec<&str> = self
            .value_blocks
            .iter()
            .map(|b| b.payload_digest.as_str())
            .collect();
        let payload = serde_json::json!({
            "layer_index": self.layer_index,
            "key_digests": key_digests,
            "value_digests": value_digests,
        });
        let json = serde_json::to_string(&payload)
            .map_err(|e| PolyKvError::Internal(format!("layer digest serialization: {}", e)))?;
        Ok(blake3::hash(json.as_bytes()).to_hex().to_string())
    }
}

/// A shared, compressed KV cache pool.
///
/// The pool holds fib-quant compressed KV blocks for tokens shared across
/// agents. It is immutable after construction. Agent shells can be materialized
/// from this pool by adding agent-specific tokens compressed with turbo-quant.
#[derive(Debug, Clone)]
pub struct SharedKVPool {
    /// Pool manifest with shape, policy, timestamps.
    pub manifest: PoolManifest,
    /// One PoolLayer per transformer layer.
    pub layers: Vec<PoolLayer>,
    /// The compression policy used.
    pub policy: CompressionPolicy,
}

impl SharedKVPool {
    /// Build a shared KV pool from a corpus of token vectors.
    ///
    /// # Arguments
    /// * `corpus` - List of (token_id, kv_vector) pairs. Each kv_vector must be
    ///   the concatenated keys and values for all layers and heads: `[layer0_head0_key,
    ///   layer0_head0_value, layer0_head1_key, ...]`.
    /// * `shape` - The tensor shape describing the model architecture.
    /// * `seed` - Deterministic seed for codec operations.
    ///
    /// # Returns
    /// The built SharedKVPool and a PoolBuildReceipt.
    pub fn build(
        corpus: &[(String, Vec<f32>)],
        shape: &KvTensorShape,
        seed: u64,
    ) -> Result<(Self, PoolBuildReceipt)> {
        let start = Instant::now();

        if corpus.is_empty() {
            return Err(PolyKvError::EmptyCorpus);
        }

        shape.validate()?;
        let policy = CompressionPolicy::default_two_tier();
        policy.validate()?;

        let num_tokens = corpus.len();
        let num_layers = shape.num_layers as usize;
        let num_kv_heads = shape.num_kv_heads as usize;
        let head_dim = shape.head_dim;

        // Validate that each corpus vector has the correct length
        let expected_len = num_layers * num_kv_heads * head_dim * 2; // key + value per head per layer
        for (token_id, vec) in corpus {
            if vec.len() != expected_len {
                return Err(PolyKvError::DimensionMismatch {
                    expected: expected_len,
                    got: vec.len(),
                });
            }
            if vec.iter().any(|v| !v.is_finite()) {
                return Err(PolyKvError::CorruptPayload(format!(
                    "token {} contains non-finite values",
                    token_id
                )));
            }
        }

        // Create the fib-quant codec for shared pool compression.
        // We compress per-head key and value vectors individually.
        let codec = create_codec(CODEC_FIB_K4_N32, head_dim, Some(&policy.fib_config), None)?;

        let mut layers: Vec<PoolLayer> = Vec::with_capacity(num_layers);
        let mut total_compressed_bytes: u64 = 0;

        for layer_idx in 0..num_layers {
            let mut key_blocks: Vec<CompressedBlock> =
                Vec::with_capacity(num_tokens * num_kv_heads);
            let mut value_blocks: Vec<CompressedBlock> =
                Vec::with_capacity(num_tokens * num_kv_heads);

            for (_token_id, vec) in corpus.iter() {
                for head_idx in 0..num_kv_heads {
                    // Each head's key and value vectors are contiguous head_dim floats
                    let base_offset =
                        layer_idx * num_kv_heads * head_dim * 2 + head_idx * head_dim * 2;

                    let key_start = base_offset;
                    let key_end = key_start + head_dim;
                    let key: Vec<f32> = vec[key_start..key_end].to_vec();

                    let value_start = key_end;
                    let value_end = value_start + head_dim;
                    let value: Vec<f32> = vec[value_start..value_end].to_vec();

                    let encoded_key = codec.encode(&key, seed)?;
                    let encoded_value = codec.encode(&value, seed)?;

                    key_blocks.push(CompressedBlock::new(
                        codec.codec_id(),
                        encoded_key,
                        head_dim,
                    ));
                    value_blocks.push(CompressedBlock::new(
                        codec.codec_id(),
                        encoded_value,
                        head_dim,
                    ));

                    total_compressed_bytes += key_blocks.last().unwrap().compressed_bytes as u64;
                    total_compressed_bytes += value_blocks.last().unwrap().compressed_bytes as u64;
                }
            }

            let mut layer = PoolLayer {
                layer_index: layer_idx as u32,
                key_blocks,
                value_blocks,
                block_digest: String::new(),
            };
            layer.block_digest = layer.compute_digest()?;
            layers.push(layer);
        }

        let raw_size_bytes = shape.total_kv_bytes(num_tokens) as u64;
        let fib_build_ms = start.elapsed().as_millis() as u64;
        let built_at_unix = now_unix();

        // Compute pool digest
        let layer_digests: Vec<String> = layers.iter().map(|l| l.block_digest.clone()).collect();
        let pool_id = blake3::hash(
            serde_json::to_string(&layer_digests)
                .map_err(|e| PolyKvError::Internal(format!("pool_id hash: {}", e)))?
                .as_bytes(),
        )
        .to_hex()
        .to_string();

        let manifest = PoolManifest::new(
            pool_id.clone(),
            shape.clone(),
            policy.clone(),
            num_tokens as u32,
            shape.num_layers,
            total_compressed_bytes,
            raw_size_bytes,
            seed,
            built_at_unix,
        )?;

        let backend = if cfg!(feature = "gpu") { "gpu" } else { "cpu" };
        let receipt = PoolBuildReceipt::new(
            pool_id,
            layer_digests,
            String::new(), // codebook_digest — not exposed at this level
            String::new(), // rotation_digest — not exposed at this level
            num_tokens as u32,
            fib_build_ms,
            total_compressed_bytes,
            raw_size_bytes,
            policy.clone(),
            seed,
            built_at_unix,
        ).with_backend(backend);

        Ok((
            Self {
                manifest,
                layers,
                policy,
            },
            receipt,
        ))
    }
    /// Materialize an agent shell from this pool.
    ///
    /// Agent-specific tokens (not in the shared corpus) are compressed with
    /// turbo-quant and appended as shell layers. Tokens already in the pool
    /// are referenced by digest only.
    ///
    /// # Arguments
    /// * `agent_id` - Identifier for this agent.
    /// * `agent_tokens` - Token vectors specific to this agent.
    /// * `seed` - Deterministic seed for turbo-quant operations.
    ///
    /// # Returns
    /// An AgentShell and a ShellMaterializeReceipt.
    pub fn materialize_shell(
        &self,
        agent_id: &str,
        agent_tokens: &[(String, Vec<f32>)],
        seed: u64,
    ) -> Result<(
        crate::shell::AgentShell,
        crate::receipt::ShellMaterializeReceipt,
    )> {
        crate::shell::materialize_shell(self, agent_id, agent_tokens, seed)
    }

    /// Inject a shell into a KV cache.
    ///
    /// The injection receipt traces every block from its source (pool or shell)
    /// to its target position in the cache.
    pub fn inject_into_cache(
        _shell: &crate::shell::AgentShell,
        _base_cache: &mut dyn CacheTarget,
    ) -> Result<crate::receipt::InjectionReceipt> {
        // The CacheTarget trait allows injection without knowing the concrete cache type.
        // This is a generic injection path — concrete adapters (e.g., for HF DynamicCache)
        // live in downstream crates.
        Err(PolyKvError::Internal(
            "inject_into_cache requires a concrete cache adapter; use inject_into_cache_with_adaptor"
                .into(),
        ))
    }
}

/// Trait for KV cache targets that can receive injected blocks.
pub trait CacheTarget: std::fmt::Debug {
    /// Get the number of layers in this cache.
    fn num_layers(&self) -> u32;

    /// Append a key block at a specific layer and position.
    fn append_key(&mut self, layer: u32, position: u32, key: &[f32]) -> Result<()>;

    /// Append a value block at a specific layer and position.
    fn append_value(&mut self, layer: u32, position: u32, value: &[f32]) -> Result<()>;

    /// Get the current sequence length (tokens in cache).
    fn seq_len(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::AttentionType;

    fn make_test_shape() -> KvTensorShape {
        KvTensorShape {
            attention_type: AttentionType::MHA,
            num_layers: 2,
            num_heads: 4,
            num_kv_heads: 4,
            head_dim: 8, // must be divisible by k=4 for fib-quant
            hidden_size: 32,
        }
    }

    fn make_test_corpus(n: usize) -> Vec<(String, Vec<f32>)> {
        use rand::Rng;
        use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let shape = make_test_shape();
        let vec_len = shape.num_layers as usize * shape.num_kv_heads as usize * shape.head_dim * 2;

        (0..n)
            .map(|i| {
                let vec: Vec<f32> = (0..vec_len).map(|_| rng.gen_range(-1.0..1.0)).collect();
                (format!("token_{}", i), vec)
            })
            .collect()
    }

    #[test]
    fn test_pool_build_empty() {
        let shape = make_test_shape();
        let corpus: Vec<(String, Vec<f32>)> = vec![];
        let result = SharedKVPool::build(&corpus, &shape, 42);
        assert!(result.is_err());
    }

    #[test]
    fn test_pool_build_basic() {
        let shape = make_test_shape();
        let corpus = make_test_corpus(4);
        let result = SharedKVPool::build(&corpus, &shape, 42);
        assert!(result.is_ok(), "build failed: {:?}", result.err());

        let (pool, receipt) = result.unwrap();
        assert_eq!(pool.layers.len(), 2);
        assert_eq!(pool.manifest.num_shared_tokens, 4);
        assert_eq!(receipt.total_tokens, 4);
        assert!(
            receipt.compression_ratio > 0.0,
            "compression ratio: {}",
            receipt.compression_ratio
        );
        // Note: ratio < 1.0 is normal for tiny test corpora where JSON
        // serialization overhead dominates the encoded payload.
    }

    #[test]
    fn test_pool_build_deterministic() {
        let shape = make_test_shape();
        let corpus = make_test_corpus(4);

        let (pool1, receipt1) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
        let (pool2, receipt2) = SharedKVPool::build(&corpus, &shape, 42).unwrap();

        assert_eq!(receipt1.pool_digest, receipt2.pool_digest);
        assert_eq!(receipt1.layer_digests, receipt2.layer_digests);
        assert_eq!(pool1.layers[0].block_digest, pool2.layers[0].block_digest);
    }

    #[test]
    fn test_pool_build_different_seeds() {
        let shape = make_test_shape();
        let corpus = make_test_corpus(4);

        let (_pool1, receipt1) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
        let (_pool2, receipt2) = SharedKVPool::build(&corpus, &shape, 12345).unwrap();

        assert_ne!(receipt1.pool_digest, receipt2.pool_digest);
    }

    #[test]
    fn test_mismatched_shape_rejected() {
        let shape = make_test_shape();
        let mut bad_corpus = make_test_corpus(1);
        // Truncate the vector
        bad_corpus[0].1.truncate(10);
        let result = SharedKVPool::build(&bad_corpus, &shape, 42);
        assert!(result.is_err());
    }
}
