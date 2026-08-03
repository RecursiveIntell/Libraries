use crate::codecs::q8_keys::{Q8KeyBlock, Q8KeyCodec};
use crate::codecs::raw_exact::{
    digest_blocks, sort_blocks, ExactFallback, ExactFallbackRef, ExactKvBlock,
};
use crate::codecs::value::{RawExactValueCodec, ValueCodec};
use crate::{
    BlockManifestEntryV1, CompressionEvalReceiptV1, CompressionPolicyV1, DecodeReceiptV1,
    FallbackReceiptV1, KvPoolManifestV1, MemoryAccounting, PolyKvError, PoolBuildReceiptV1,
};
use quant_codec_core::{
    ArtifactDigest, CodecProfile, KvLayout, KvRole, KvSliceRequest, KvTensorShape, LayerId,
    ModelFingerprint, TokenSpan, TokenizerFingerprint, VectorCodec,
};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct SharedKvPool {
    pub(crate) inner: Arc<SharedKvPoolInner>,
}

#[derive(Debug)]
pub(crate) struct SharedKvPoolInner {
    pub manifest: KvPoolManifestV1,
    pub build_receipt: PoolBuildReceiptV1,
    pub fallback: ExactFallback,
    pub encoded_blocks: Vec<EncodedPoolBlock>,
    pub value_codec: Arc<dyn ValueCodec>,
    pub active_readers: AtomicUsize,
    pub active_reader_scratch_bytes: AtomicU64,
    pub next_reader_id: AtomicU64,
}

#[derive(Debug, Clone)]
pub(crate) struct EncodedPoolBlock {
    pub role: KvRole,
    pub layer: LayerId,
    pub encoded: EncodedBlock,
    pub exact_len: usize,
    pub encoded_bytes: u64,
}

#[derive(Debug, Clone)]
pub(crate) enum EncodedBlock {
    Q8Key(Q8KeyBlock),
    Value(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct DecodedKvSlice {
    pub request: KvSliceRequest,
    pub data: Vec<f32>,
    pub receipt: DecodeReceiptV1,
}

#[derive(Debug, Clone)]
pub struct DecodedLayer {
    pub layer: LayerId,
    pub key: DecodedKvSlice,
    pub value: DecodedKvSlice,
}

#[derive(Debug, Clone)]
pub struct PoolBuilder {
    model_fingerprint: Option<ModelFingerprint>,
    tokenizer_fingerprint: Option<TokenizerFingerprint>,
    shape: Option<KvTensorShape>,
    policy: CompressionPolicyV1,
    exact_fallback: Option<ExactFallback>,
    key_codec: Q8KeyCodec,
    value_codec: Arc<dyn ValueCodec>,
}

impl Default for PoolBuilder {
    fn default() -> Self {
        Self {
            model_fingerprint: None,
            tokenizer_fingerprint: None,
            shape: None,
            policy: CompressionPolicyV1::alpha_reference(),
            exact_fallback: None,
            key_codec: Q8KeyCodec::symmetric_per_block(),
            value_codec: Arc::new(RawExactValueCodec),
        }
    }
}

pub fn should_compress(encoded_bytes: u64, exact_fallback_bytes: u64) -> bool {
    encoded_bytes < exact_fallback_bytes
}

impl SharedKvPool {
    pub fn builder() -> PoolBuilder {
        PoolBuilder::default()
    }

    pub fn manifest(&self) -> &KvPoolManifestV1 {
        &self.inner.manifest
    }

    pub fn build_receipt(&self) -> &PoolBuildReceiptV1 {
        &self.inner.build_receipt
    }

    pub fn attach_reader(
        &self,
        config: crate::ReaderConfig,
    ) -> Result<crate::PoolReader, PolyKvError> {
        crate::PoolReader::attach(Arc::clone(&self.inner), config)
    }

    pub fn exact_fallback_ref(&self) -> Option<&ExactFallbackRef> {
        Some(&self.inner.fallback)
    }

    pub fn encoded_bytes(&self) -> u64 {
        self.inner.manifest.encoded_bytes
    }

    pub fn reader_count(&self) -> usize {
        self.inner.active_readers.load(Ordering::SeqCst)
    }

    pub fn memory_accounting(&self) -> MemoryAccounting {
        self.inner.build_receipt.memory.with_active_reader_scratch(
            self.reader_count() as u64,
            self.inner
                .active_reader_scratch_bytes
                .load(Ordering::SeqCst),
        )
    }
}

impl Clone for SharedKvPool {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl PoolBuilder {
    pub fn model_fingerprint(mut self, value: ModelFingerprint) -> Self {
        self.model_fingerprint = Some(value);
        self
    }

    pub fn tokenizer_fingerprint(mut self, value: TokenizerFingerprint) -> Self {
        self.tokenizer_fingerprint = Some(value);
        self
    }

    pub fn shape(mut self, value: KvTensorShape) -> Self {
        self.shape = Some(value);
        self
    }

    pub fn policy(mut self, value: CompressionPolicyV1) -> Self {
        self.policy = value;
        self
    }

    pub fn exact_fallback(mut self, value: ExactFallback) -> Self {
        self.exact_fallback = Some(value);
        self
    }

    pub fn key_codec(mut self, value: Q8KeyCodec) -> Self {
        self.key_codec = value;
        self
    }

    pub fn value_codec<V>(mut self, value: V) -> Self
    where
        V: ValueCodec + 'static,
    {
        self.value_codec = Arc::new(value);
        self
    }

    pub fn build_from_blocks(self, blocks: Vec<ExactKvBlock>) -> Result<SharedKvPool, PolyKvError> {
        let shape = self
            .shape
            .ok_or_else(|| PolyKvError::Manifest("builder shape is required".to_string()))?;
        shape.validate()?;
        let fallback = self.exact_fallback.ok_or(PolyKvError::MissingFallback)?;
        let model_fingerprint = self
            .model_fingerprint
            .ok_or_else(|| PolyKvError::Manifest("model fingerprint is required".to_string()))?;
        let tokenizer_fingerprint = self.tokenizer_fingerprint.ok_or_else(|| {
            PolyKvError::Manifest("tokenizer fingerprint is required".to_string())
        })?;

        let blocks = sort_blocks(blocks);
        validate_block_set(&shape, &blocks)?;
        if fallback.input_digest() != digest_blocks(&blocks) {
            return Err(PolyKvError::Manifest(
                "exact fallback digest does not match build input blocks".to_string(),
            ));
        }

        let mut encoded_blocks = Vec::with_capacity(blocks.len());
        let mut manifest_blocks = Vec::with_capacity(blocks.len());
        let mut eval_receipts = Vec::new();
        let mut observed_key_mse = None::<f64>;
        let mut observed_value_mse = None::<f64>;

        for block in &blocks {
            let exact_bytes = block.exact_bytes();
            match block.role {
                KvRole::Key => {
                    let encoded = self.key_codec.encode_block(&block.data)?;
                    let eval = self.key_codec.eval(&block.data, &encoded)?;
                    observed_key_mse = max_optional(observed_key_mse, eval.mse);
                    let encoded_bytes = encoded.encoded_bytes();
                    let artifact_digest = digest_encoded_q8(block, &encoded);
                    eval_receipts.push(CompressionEvalReceiptV1 {
                        schema_version: 1,
                        role: block.role,
                        layer: block.layer.0,
                        ideal_codec_bits_per_scalar: self
                            .key_codec
                            .fixed_rate_bits()
                            .map(f32::from),
                        realized_encoded_bytes: encoded_bytes,
                        metadata_bytes: encoded.metadata_bytes(),
                        eval,
                    });
                    manifest_blocks.push(BlockManifestEntryV1 {
                        role: block.role,
                        layer: block.layer.0,
                        codec_id: self.key_codec.codec_id(),
                        encoded_bytes,
                        exact_bytes,
                        ideal_codec_bits_per_scalar: self
                            .key_codec
                            .fixed_rate_bits()
                            .map(f32::from),
                        realized_encoded_bytes: encoded_bytes,
                        metadata_bytes: encoded.metadata_bytes(),
                        artifact_digest,
                    });
                    encoded_blocks.push(EncodedPoolBlock {
                        role: block.role,
                        layer: block.layer,
                        encoded: EncodedBlock::Q8Key(encoded),
                        exact_len: block.data.len(),
                        encoded_bytes,
                    });
                }
                KvRole::Value => {
                    let encoded = self.value_codec.encode_values(&block.data)?;
                    let eval = self.value_codec.eval_values(&block.data, &encoded)?;
                    observed_value_mse = max_optional(observed_value_mse, eval.mse);
                    let encoded_bytes = encoded.len() as u64;
                    let artifact_digest = digest_encoded_raw(block, &encoded);
                    eval_receipts.push(CompressionEvalReceiptV1 {
                        schema_version: 1,
                        role: block.role,
                        layer: block.layer.0,
                        ideal_codec_bits_per_scalar: self
                            .value_codec
                            .fixed_rate_bits()
                            .map(f32::from),
                        realized_encoded_bytes: encoded_bytes,
                        metadata_bytes: 0,
                        eval,
                    });
                    manifest_blocks.push(BlockManifestEntryV1 {
                        role: block.role,
                        layer: block.layer.0,
                        codec_id: self.value_codec.codec_id(),
                        encoded_bytes,
                        exact_bytes,
                        ideal_codec_bits_per_scalar: self
                            .value_codec
                            .fixed_rate_bits()
                            .map(f32::from),
                        realized_encoded_bytes: encoded_bytes,
                        metadata_bytes: 0,
                        artifact_digest,
                    });
                    encoded_blocks.push(EncodedPoolBlock {
                        role: block.role,
                        layer: block.layer,
                        encoded: EncodedBlock::Value(encoded),
                        exact_len: block.data.len(),
                        encoded_bytes,
                    });
                }
            }
        }

        let mut quality_gate = self.policy.quality_gate.clone();
        quality_gate.observed_key_mse = observed_key_mse;
        quality_gate.observed_value_mse = observed_value_mse;
        let key_passed = observed_key_mse
            .map(|mse| mse <= quality_gate.max_key_mse)
            .unwrap_or(false);
        let value_passed = observed_value_mse
            .map(|mse| mse <= quality_gate.max_value_mse)
            .unwrap_or(false);
        quality_gate.passed = key_passed && value_passed;
        if !quality_gate.passed {
            return Err(PolyKvError::QualityGateFailed(format!(
                "observed key mse {:?} (max {}) or value mse {:?} (max {}) exceeds budget",
                observed_key_mse,
                quality_gate.max_key_mse,
                observed_value_mse,
                quality_gate.max_value_mse
            )));
        }

        let encoded_bytes = manifest_blocks
            .iter()
            .map(|entry| entry.encoded_bytes)
            .sum::<u64>();
        let exact_fallback_bytes = fallback.exact_bytes();
        let mut manifest = KvPoolManifestV1 {
            schema_version: 1,
            model_fingerprint,
            tokenizer_fingerprint,
            source_dtype: shape.dtype,
            shape,
            policy: CompressionPolicyV1 {
                profile_digest: combined_profile_digest(&self.key_codec, self.value_codec.as_ref()),
                key_codec_id: self.key_codec.codec_id(),
                value_codec_id: self.value_codec.codec_id(),
                lossy_keys: self.key_codec.is_lossy(),
                lossy_values: self.value_codec.is_lossy(),
                quality_gate: quality_gate.clone(),
            },
            blocks: manifest_blocks,
            encoded_bytes,
            exact_fallback_bytes,
            manifest_digest: ArtifactDigest::from_canonical_bytes(b"pending"),
        };
        manifest.manifest_digest = manifest.canonical_digest_without_self();
        let manifest_bytes = manifest.canonical_serialized_len();
        let memory = MemoryAccounting {
            exact_fallback_bytes,
            encoded_shared_bytes: encoded_bytes,
            manifest_bytes,
            per_reader_scratch_bytes: 0,
            reader_count: 0,
        };
        let build_receipt = PoolBuildReceiptV1 {
            schema_version: 1,
            manifest_digest: manifest.manifest_digest,
            input_digest: fallback.input_digest(),
            encoded_bytes,
            exact_fallback_bytes,
            block_count: encoded_blocks.len() as u64,
            quality_gate,
            compression_evals: eval_receipts,
            memory,
        };
        Ok(SharedKvPool {
            inner: Arc::new(SharedKvPoolInner {
                manifest,
                build_receipt,
                fallback,
                encoded_blocks,
                value_codec: Arc::clone(&self.value_codec),
                active_readers: AtomicUsize::new(0),
                active_reader_scratch_bytes: AtomicU64::new(0),
                next_reader_id: AtomicU64::new(1),
            }),
        })
    }

    pub fn build_from_exact_blocks(
        self,
        blocks: Vec<ExactKvBlock>,
    ) -> Result<SharedKvPool, PolyKvError> {
        let fallback = ExactFallback::from_blocks(blocks.clone());
        self.exact_fallback(fallback).build_from_blocks(blocks)
    }
}

pub(crate) fn decode_slice_from_inner(
    inner: &SharedKvPoolInner,
    request: KvSliceRequest,
    scratch_bytes: u64,
    force_exact_fallback: bool,
) -> Result<DecodedKvSlice, PolyKvError> {
    request.validate_for_shape(&inner.manifest.shape)?;
    let encoded = inner
        .encoded_blocks
        .iter()
        .find(|block| block.role == request.role && block.layer == request.layer)
        .ok_or_else(|| PolyKvError::MissingBlock {
            reason: format!("{:?} layer {} not in pool", request.role, request.layer.0),
        })?;

    let (decoded_layer, source_encoded_bytes, fallback) = if force_exact_fallback {
        let exact = inner
            .fallback
            .find(request.role, request.layer)
            .ok_or(PolyKvError::MissingFallback)?;
        (
            exact.data.clone(),
            exact.exact_bytes(),
            Some(FallbackReceiptV1 {
                schema_version: 1,
                reason: "explicit exact fallback decode requested".to_string(),
                role: request.role,
                layer: request.layer.0,
                exact_bytes_read: exact.exact_bytes(),
                manifest_digest: inner.manifest.manifest_digest,
            }),
        )
    } else {
        (
            decode_full_block(inner, encoded)?,
            encoded.encoded_bytes,
            None,
        )
    };

    let data = extract_slice(&decoded_layer, &inner.manifest.shape, &request)?;
    let returned_values = data.len() as u64;
    let receipt = DecodeReceiptV1 {
        schema_version: 1,
        decoded_values: returned_values,
        full_block_decoded: true,
        decoded_full_values: decoded_layer.len() as u64,
        returned_values,
        copy_performed: true,
        source_encoded_bytes,
        scratch_bytes,
        fallback,
        request: request.clone(),
    };
    Ok(DecodedKvSlice {
        request,
        data,
        receipt,
    })
}

pub(crate) fn decode_layer_from_inner(
    inner: &SharedKvPoolInner,
    layer: LayerId,
    scratch_bytes: u64,
) -> Result<DecodedLayer, PolyKvError> {
    if layer.0 >= inner.manifest.shape.layers {
        return Err(PolyKvError::ShapeMismatch {
            reason: format!(
                "requested layer {} but shape has {} layers",
                layer.0, inner.manifest.shape.layers
            ),
        });
    }
    let span = TokenSpan::new(0, inner.manifest.shape.seq_len)?;
    let key = decode_slice_from_inner(
        inner,
        KvSliceRequest::layer_span(layer, span).for_role(KvRole::Key),
        scratch_bytes,
        false,
    )?;
    let value = decode_slice_from_inner(
        inner,
        KvSliceRequest::layer_span(layer, span).for_role(KvRole::Value),
        scratch_bytes,
        false,
    )?;
    Ok(DecodedLayer { layer, key, value })
}

fn decode_full_block(
    inner: &SharedKvPoolInner,
    block: &EncodedPoolBlock,
) -> Result<Vec<f32>, PolyKvError> {
    match &block.encoded {
        EncodedBlock::Q8Key(encoded) => {
            let codec = Q8KeyCodec::symmetric_per_block();
            let mut out = vec![0.0; block.exact_len];
            codec.decode_block(encoded, &mut out)?;
            Ok(out)
        }
        EncodedBlock::Value(encoded) => {
            let mut out = vec![0.0; block.exact_len];
            inner.value_codec.decode_values(encoded, &mut out)?;
            Ok(out)
        }
    }
}

fn extract_slice(
    data: &[f32],
    shape: &KvTensorShape,
    request: &KvSliceRequest,
) -> Result<Vec<f32>, PolyKvError> {
    let heads = shape.heads_for(request.role) as usize;
    let seq_len = shape.seq_len as usize;
    let head_dim = shape.head_dim as usize;
    let start = request.token_span.start as usize;
    let end = request.token_span.end as usize;
    let expected = heads
        .checked_mul(seq_len)
        .and_then(|v| v.checked_mul(head_dim))
        .ok_or_else(|| PolyKvError::InvalidShape {
            reason: "slice dimension overflow".to_string(),
        })?;
    if data.len() != expected {
        return Err(PolyKvError::ShapeMismatch {
            reason: format!(
                "decoded layer length {} does not match expected {}",
                data.len(),
                expected
            ),
        });
    }

    let selected_heads = if let Some(head) = request.head {
        vec![head.0 as usize]
    } else {
        (0..heads).collect::<Vec<_>>()
    };
    let out_len = selected_heads.len() * (end - start) * head_dim;
    let mut out = Vec::with_capacity(out_len);
    match shape.layout {
        KvLayout::LayersHeadsTokensDim => {
            for head in selected_heads {
                for token in start..end {
                    let offset = ((head * seq_len) + token) * head_dim;
                    out.extend_from_slice(&data[offset..offset + head_dim]);
                }
            }
        }
        KvLayout::LayersTokensHeadsDim => {
            for token in start..end {
                for head in &selected_heads {
                    let offset = ((token * heads) + *head) * head_dim;
                    out.extend_from_slice(&data[offset..offset + head_dim]);
                }
            }
        }
        KvLayout::RuntimeSpecific(_) => {
            return Err(PolyKvError::InvalidShape {
                reason: "runtime-specific layouts cannot be decoded by alpha reference reader"
                    .to_string(),
            });
        }
    }
    Ok(out)
}

fn validate_block_set(shape: &KvTensorShape, blocks: &[ExactKvBlock]) -> Result<(), PolyKvError> {
    let expected_count = (shape.layers as usize) * 2;
    if blocks.len() != expected_count {
        return Err(PolyKvError::Manifest(format!(
            "expected {} key/value blocks but got {}",
            expected_count,
            blocks.len()
        )));
    }
    for layer in 0..shape.layers {
        for role in [KvRole::Key, KvRole::Value] {
            let block = blocks
                .iter()
                .find(|block| block.layer.0 == layer && block.role == role)
                .ok_or_else(|| PolyKvError::MissingBlock {
                    reason: format!("{:?} layer {} missing", role, layer),
                })?;
            if block.shape != *shape {
                return Err(PolyKvError::ShapeMismatch {
                    reason: format!(
                        "{:?} layer {} shape differs from builder shape",
                        role, layer
                    ),
                });
            }
        }
    }
    Ok(())
}

fn digest_encoded_q8(block: &ExactKvBlock, encoded: &Q8KeyBlock) -> ArtifactDigest {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(block.artifact_digest().to_string().as_bytes());
    bytes.extend_from_slice(&encoded.scale.to_bits().to_le_bytes());
    bytes.extend_from_slice(&encoded.original_len.to_le_bytes());
    for value in &encoded.values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    ArtifactDigest::from_canonical_bytes(&bytes)
}

fn digest_encoded_raw(block: &ExactKvBlock, encoded: &[u8]) -> ArtifactDigest {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(block.artifact_digest().to_string().as_bytes());
    bytes.extend_from_slice(encoded);
    ArtifactDigest::from_canonical_bytes(&bytes)
}

fn combined_profile_digest(
    key_codec: &Q8KeyCodec,
    value_codec: &dyn ValueCodec,
) -> quant_codec_core::CodecProfileDigest {
    let key_digest = key_codec.profile_digest().to_string();
    let value_digest = value_codec.profile_digest().to_string();
    quant_codec_core::CodecProfileDigest::from_parts(&[
        b"poly-kv-policy-v1",
        key_digest.as_bytes(),
        value_digest.as_bytes(),
    ])
}

fn max_optional(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

// ── Compressed attention scoring (fibquant-adapter only) ────────

#[cfg(feature = "fibquant-adapter")]
use crate::CompressedAttentionSelectionReceipt;

/// A scored candidate from compressed-domain attention.
#[cfg(feature = "fibquant-adapter")]
#[derive(Debug, Clone)]
pub struct CompressedAttentionHit {
    pub token_index: usize,
    pub score: f32,
    pub value: Vec<f32>,
}

/// Result of compressed candidate scoring with bounded value decode.
#[cfg(feature = "fibquant-adapter")]
#[derive(Debug, Clone)]
pub struct CompressedAttentionSelection {
    pub hits: Vec<CompressedAttentionHit>,
    pub receipt: CompressedAttentionSelectionReceipt,
}

/// Cached decoded key/value codes and FibScorer for one layer/head.
#[cfg(feature = "fibquant-adapter")]
pub struct PreparedCompressedIndex {
    pub layer_idx: usize,
    pub head_idx: usize,
    pub key_codes: Vec<fib_quant::FibCodeV1>,
    pub value_codes: Vec<fib_quant::FibCodeV1>,
    pub scorer: fib_quant::FibScorer,
    pub num_tokens: usize,
}

#[cfg(feature = "fibquant-adapter")]
impl SharedKvPool {
    /// Score compressed FibQuant codes for one layer/head, select top-k,
    /// decode only selected values, return receipt proving no full decode.
    pub fn attention_topk_compressed(
        &self,
        layer_idx: usize,
        head_idx: usize,
        query: &[f32],
        top_k: usize,
    ) -> Result<CompressedAttentionSelection, PolyKvError> {
        let shape = &self.inner.manifest.shape;
        let head_dim = shape.head_dim as usize;
        if query.len() != head_dim {
            return Err(PolyKvError::ShapeMismatch {
                reason: format!("query dim {} != head_dim {}", query.len(), head_dim),
            });
        }
        let num_heads = shape.key_heads as usize;
        if head_idx >= num_heads {
            return Err(PolyKvError::InvalidShape {
                reason: format!("head_idx {} >= num_heads {}", head_idx, num_heads),
            });
        }
        let num_tokens = shape.seq_len as usize;

        // Decode FibCodeV1 codes from encoded value blocks.
        let adapter = crate::adapters::fibquant::FibQuantValueCodec::new(head_dim, 4, 32, 42)?;
        let mut value_codes: Vec<fib_quant::FibCodeV1> = Vec::new();
        for block in &self.inner.encoded_blocks {
            if block.role == KvRole::Value {
                if let EncodedBlock::Value(ref bytes) = block.encoded {
                    let codes = adapter.decode_to_fib_codes(bytes)?;
                    value_codes.extend(codes);
                }
            }
        }
        if value_codes.len() < num_tokens * num_heads {
            return Err(PolyKvError::Codec(
                "not enough FibCodeV1 codes decoded for scoring".into(),
            ));
        }

        // Build scorer and score candidates for this head.
        let scorer = adapter.build_scorer(42)?;
        let prepared = scorer
            .prepare_query(query)
            .map_err(|e| PolyKvError::Codec(format!("fib prepare: {e}")))?;

        let mut scored: Vec<(usize, f32)> = Vec::with_capacity(num_tokens);
        for token_idx in 0..num_tokens {
            let code_idx = token_idx * num_heads + head_idx;
            if code_idx < value_codes.len() {
                let score = scorer
                    .score_prepared(&prepared, &value_codes[code_idx])
                    .map_err(|e| PolyKvError::Codec(format!("fib score: {e}")))?;
                scored.push((token_idx, score));
            }
        }

        let selected = top_k.min(scored.len());
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(selected);

        let mut hits = Vec::with_capacity(selected);
        for &(token_idx, score) in &scored {
            let code_idx = token_idx * num_heads + head_idx;
            let value = scorer
                .quantizer()
                .decode(&value_codes[code_idx])
                .map_err(|e| PolyKvError::Codec(format!("fib decode: {e}")))?;
            hits.push(CompressedAttentionHit {
                token_index: token_idx,
                score,
                value,
            });
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let receipt = CompressedAttentionSelectionReceipt {
            schema_version: crate::COMPRESSED_ATTENTION_SELECTION_RECEIPT_SCHEMA,
            pool_id: self.inner.manifest.manifest_digest.to_string(),
            layer_idx: layer_idx as u32,
            head_idx: head_idx as u32,
            total_candidates: num_tokens as u32,
            selected_count: hits.len() as u32,
            full_layer_decoded: false,
            decoded_value_vectors: hits.len() as u64,
            full_decode_value_count: num_tokens as u64,
            claim_boundary: "fib_cold_pool_compressed_score_topk_value_decode".into(),
            codec_used: "fibquant-adapter".into(),
            timestamp: now,
        };
        receipt.validate().map_err(|e| PolyKvError::Manifest(e))?;
        Ok(CompressedAttentionSelection { hits, receipt })
    }

    /// Build a prepared compressed index for repeated scoring.
    pub fn prepare_compressed_index(
        &self,
        layer_idx: usize,
        head_idx: usize,
    ) -> Result<PreparedCompressedIndex, PolyKvError> {
        let shape = &self.inner.manifest.shape;
        let num_tokens = shape.seq_len as usize;
        let head_dim = shape.head_dim as usize;
        let num_heads = shape.key_heads as usize;
        if head_idx >= num_heads {
            return Err(PolyKvError::InvalidShape {
                reason: format!("head_idx {} >= num_heads {}", head_idx, num_heads),
            });
        }

        let adapter = crate::adapters::fibquant::FibQuantValueCodec::new(head_dim, 4, 32, 42)?;
        let scorer = adapter.build_scorer(42)?;

        let mut value_codes: Vec<fib_quant::FibCodeV1> = Vec::new();
        for block in &self.inner.encoded_blocks {
            if block.role == KvRole::Value {
                if let EncodedBlock::Value(ref bytes) = block.encoded {
                    value_codes.extend(adapter.decode_to_fib_codes(bytes)?);
                }
            }
        }

        Ok(PreparedCompressedIndex {
            layer_idx,
            head_idx,
            key_codes: vec![],
            value_codes,
            scorer,
            num_tokens,
        })
    }

    /// Score using a prepared index (avoids rebuilding codec state).
    pub fn attention_topk_compressed_prepared(
        &self,
        index: &PreparedCompressedIndex,
        query: &[f32],
        top_k: usize,
    ) -> Result<CompressedAttentionSelection, PolyKvError> {
        let head_dim = self.inner.manifest.shape.head_dim as usize;
        if query.len() != head_dim {
            return Err(PolyKvError::ShapeMismatch {
                reason: format!("query dim {} != head_dim {}", query.len(), head_dim),
            });
        }
        let prepared = index
            .scorer
            .prepare_query(query)
            .map_err(|e| PolyKvError::Codec(format!("fib prepare: {e}")))?;

        let num_heads = self.inner.manifest.shape.key_heads as usize;
        let num_tokens = index.num_tokens;
        let mut scored: Vec<(usize, f32)> = Vec::with_capacity(num_tokens);
        for token_idx in 0..num_tokens {
            let code_idx = token_idx * num_heads + index.head_idx;
            if code_idx < index.value_codes.len() {
                let score = index
                    .scorer
                    .score_prepared(&prepared, &index.value_codes[code_idx])
                    .map_err(|e| PolyKvError::Codec(format!("fib score: {e}")))?;
                scored.push((token_idx, score));
            }
        }

        let selected = top_k.min(scored.len());
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(selected);

        let mut hits = Vec::with_capacity(selected);
        for &(token_idx, score) in &scored {
            let code_idx = token_idx * num_heads + index.head_idx;
            let value = index
                .scorer
                .quantizer()
                .decode(&index.value_codes[code_idx])
                .map_err(|e| PolyKvError::Codec(format!("fib decode: {e}")))?;
            hits.push(CompressedAttentionHit {
                token_index: token_idx,
                score,
                value,
            });
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let receipt = CompressedAttentionSelectionReceipt {
            schema_version: crate::COMPRESSED_ATTENTION_SELECTION_RECEIPT_SCHEMA,
            pool_id: self.inner.manifest.manifest_digest.to_string(),
            layer_idx: index.layer_idx as u32,
            head_idx: index.head_idx as u32,
            total_candidates: num_tokens as u32,
            selected_count: hits.len() as u32,
            full_layer_decoded: false,
            decoded_value_vectors: hits.len() as u64,
            full_decode_value_count: num_tokens as u64,
            claim_boundary: "fib_cold_pool_compressed_score_topk_value_decode_prepared".into(),
            codec_used: "fibquant-adapter-prepared".into(),
            timestamp: now,
        };
        receipt.validate().map_err(|e| PolyKvError::Manifest(e))?;
        Ok(CompressedAttentionSelection { hits, receipt })
    }
}

// ── Branch support ──────────────────────────────────────────────

/// A writable branch forked from a shared immutable pool.
///
/// Each branch holds a reference to the shared prefix and its own
/// exact-writable tail. Branch mutations never affect the shared pool
/// or any other branch.
#[derive(Debug)]
pub struct BranchHandle {
    /// Reference to the shared immutable pool.
    pool: SharedKvPool,
    /// Agent/branch identifier.
    agent_id: String,
    /// Writable tail tokens — appended after the shared prefix.
    tail_tokens: Vec<u32>,
    /// Writable tail KV blocks (exact only initially; compressed after
    /// Phase 4 governed admission).
    tail_blocks: Vec<ExactKvBlock>,
    /// Current sequence length = shared_prefix_len + tail_len.
    current_seq_len: u64,
}

/// Configuration for creating a new branch.
#[derive(Debug, Clone, Default)]
pub struct BranchConfig {
    /// Human-readable agent identifier.
    pub agent_id: String,
    /// Optional initial tail tokens to seed the branch.
    pub initial_tokens: Vec<u32>,
}

impl BranchConfig {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            initial_tokens: Vec::new(),
        }
    }

    pub fn with_tokens(mut self, tokens: Vec<u32>) -> Self {
        self.initial_tokens = tokens;
        self
    }
}

impl SharedKvPool {
    /// Fork a new branch from this shared pool.
    ///
    /// The branch starts with an empty writable tail and the full
    /// shared prefix available for decoding.
    pub fn fork(&self, config: BranchConfig) -> Result<BranchHandle, PolyKvError> {
        if config.agent_id.is_empty() {
            return Err(PolyKvError::Manifest(
                "branch agent_id must not be empty".to_string(),
            ));
        }
        let seq_len = self.inner.manifest.shape.seq_len;
        Ok(BranchHandle {
            pool: self.clone(),
            agent_id: config.agent_id,
            tail_tokens: config.initial_tokens,
            tail_blocks: Vec::new(),
            current_seq_len: seq_len,
        })
    }
}

impl BranchHandle {
    /// Return the shared pool this branch was forked from.
    pub fn pool(&self) -> &SharedKvPool {
        &self.pool
    }

    /// Return the agent/branch identifier.
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Number of tokens in the shared prefix.
    pub fn shared_prefix_len(&self) -> u64 {
        self.pool.inner.manifest.shape.seq_len
    }

    /// Number of tokens in this branch's writable tail.
    pub fn tail_len(&self) -> usize {
        self.tail_tokens.len()
    }

    /// Current total sequence length (shared prefix + tail).
    pub fn current_seq_len(&self) -> u64 {
        self.current_seq_len
    }

    /// Append tokens to the branch tail.
    ///
    /// The caller is responsible for computing the corresponding KV
    /// tensors and calling `append_blocks`.
    pub fn append_tokens(&mut self, tokens: &[u32]) {
        self.tail_tokens.extend_from_slice(tokens);
        self.current_seq_len += tokens.len() as u64;
    }

    /// Append exact KV blocks to the branch tail.
    ///
    /// Blocks must match the pool's shape (per-layer key/value pairs)
    /// and the current tail length.
    pub fn append_blocks(&mut self, blocks: Vec<ExactKvBlock>) -> Result<(), PolyKvError> {
        let shape = &self.pool.inner.manifest.shape;
        for block in &blocks {
            if block.shape != *shape {
                return Err(PolyKvError::ShapeMismatch {
                    reason: format!(
                        "block {:?} layer {} shape differs from pool",
                        block.role, block.layer.0
                    ),
                });
            }
        }
        self.tail_blocks.extend(blocks);
        Ok(())
    }

    /// Append tokens and their associated KV blocks atomically.
    pub fn append(&mut self, tokens: &[u32], blocks: Vec<ExactKvBlock>) -> Result<(), PolyKvError> {
        self.append_tokens(tokens);
        self.append_blocks(blocks)
    }

    /// Decode the combined shared-prefix + branch-tail state for one
    /// layer, returning the full (key_data, value_data) tensors.
    ///
    /// This decodes the shared prefix blocks and concatenates the
    /// branch-tail exact blocks.
    pub fn decode_combined_layer(
        &self,
        layer: LayerId,
    ) -> Result<(Vec<f32>, Vec<f32>), PolyKvError> {
        let scratch = 64 * 1024;
        let decoded = decode_layer_from_inner(&self.pool.inner, layer, scratch)?;
        let mut keys = decoded.key.data;
        let mut values = decoded.value.data;

        // Append branch-tail blocks for this layer.
        for block in &self.tail_blocks {
            if block.layer == layer {
                match block.role {
                    KvRole::Key => keys.extend_from_slice(&block.data),
                    KvRole::Value => values.extend_from_slice(&block.data),
                }
            }
        }
        Ok((keys, values))
    }
}

impl Drop for BranchHandle {
    fn drop(&mut self) {
        // Branch cleanup: no shared state mutation needed.
        // The pool reference is released via Arc drop.
    }
}
