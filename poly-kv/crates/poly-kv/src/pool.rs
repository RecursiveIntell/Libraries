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
    RawValue(Vec<f32>),
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
    value_codec: RawExactValueCodec,
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
            value_codec: RawExactValueCodec,
        }
    }
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

    pub fn value_codec(mut self, value: RawExactValueCodec) -> Self {
        self.value_codec = value;
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
                    let encoded_bytes = (encoded.len() as u64) * 4;
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
                        encoded: EncodedBlock::RawValue(encoded),
                        exact_len: block.data.len(),
                        encoded_bytes,
                    });
                }
            }
        }

        let mut quality_gate = self.policy.quality_gate.clone();
        quality_gate.observed_key_mse = observed_key_mse;
        quality_gate.passed = observed_key_mse
            .map(|mse| mse <= quality_gate.max_key_mse)
            .unwrap_or(false);
        if !quality_gate.passed {
            return Err(PolyKvError::QualityGateFailed(format!(
                "observed key mse {:?} exceeds max {}",
                observed_key_mse, quality_gate.max_key_mse
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
                quality_gate: quality_gate.clone(),
                ..self.policy
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
        (decode_full_block(encoded)?, encoded.encoded_bytes, None)
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

fn decode_full_block(block: &EncodedPoolBlock) -> Result<Vec<f32>, PolyKvError> {
    match &block.encoded {
        EncodedBlock::Q8Key(encoded) => {
            let codec = Q8KeyCodec::symmetric_per_block();
            let mut out = vec![0.0; block.exact_len];
            codec.decode_block(encoded, &mut out)?;
            Ok(out)
        }
        EncodedBlock::RawValue(encoded) => Ok(encoded.clone()),
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

fn digest_encoded_raw(block: &ExactKvBlock, encoded: &[f32]) -> ArtifactDigest {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(block.artifact_digest().to_string().as_bytes());
    for value in encoded {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    ArtifactDigest::from_canonical_bytes(&bytes)
}

fn max_optional(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}
