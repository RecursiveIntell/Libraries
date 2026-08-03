use crate::{codecs::value::ValueCodec, metrics, PolyKvError};
use fib_quant::{
    kv::{
        decode_kv_pages, decode_kv_wire, encode_kv_tensor, encode_kv_wire, KvAttentionKind,
        KvAxisPolicyV1, KvCacheLayoutV1, KvCompressionProfileV1, KvDType, KvEncodedTensorV1,
        KvPageGeometryV1, KvRole, KvRopeState, KvTensorShapeV1,
    },
    profile::FibQuantProfileV1,
    FibQuantizer,
};
use quant_codec_core::{
    CodecCapabilities, CodecId, CodecProfile, CodecProfileDigest, CodecResourceLimits, EvalReport,
    ScoreSemantics, VectorCodec,
};

/// Feature-gated PolyKV value adapter backed by FibQuant's canonical CPU KV API.
#[derive(Debug, Clone)]
pub struct FibQuantValueCodec {
    head_dim: usize,
    shape: KvTensorShapeV1,
    profile: KvCompressionProfileV1,
    profile_digest: String,
    max_mse: Option<f64>,
}

const MAX_ADAPTER_ENCODED_BYTES: u64 = 1 << 28;

impl FibQuantValueCodec {
    /// Build a deterministic one-vector value profile.
    pub fn new(
        head_dim: usize,
        block_dim: usize,
        codebook_size: usize,
        seed: u64,
    ) -> Result<Self, PolyKvError> {
        let shape = KvTensorShapeV1::new(
            KvRole::Value,
            KvAttentionKind::Mha,
            1,
            1,
            1,
            1,
            1,
            u32::try_from(head_dim).map_err(|err| PolyKvError::InvalidShape {
                reason: err.to_string(),
            })?,
            KvDType::F32,
            KvRopeState::NotApplicable,
        );
        let fib_profile =
            FibQuantProfileV1::paper_default(head_dim, block_dim, codebook_size, seed)
                .map_err(fib_error)?;
        let quantizer = FibQuantizer::new(fib_profile.clone()).map_err(fib_error)?;
        let raw_bytes = head_dim
            .checked_mul(4)
            .and_then(|bytes| bytes.checked_add(1024))
            .and_then(|bytes| u32::try_from(bytes).ok())
            .ok_or_else(|| PolyKvError::InvalidShape {
                reason: "encoded block reservation overflow".to_string(),
            })?;
        let page_geometry = KvPageGeometryV1::new(1, shape.head_dim, raw_bytes);
        let profile = KvCompressionProfileV1::from_parts(
            format!("poly-kv:fibquant:value:{head_dim}:{block_dim}:{codebook_size}:{seed}"),
            &shape,
            fib_profile,
            quantizer.codebook().codebook_digest.clone(),
            KvAxisPolicyV1::PerToken,
            page_geometry,
        )
        .map_err(fib_error)?;
        let profile_digest = profile.digest(&shape).map_err(fib_error)?;
        Ok(Self {
            head_dim,
            shape,
            profile,
            profile_digest,
            max_mse: None,
        })
    }

    /// Attach the finite per-block MSE budget used by [`ValueCodec::eval_values`].
    ///
    /// Without this budget, encoding and decoding remain available but quality
    /// evaluation fails closed rather than silently admitting a lossy block.
    pub fn with_max_mse(mut self, max_mse: f64) -> Result<Self, PolyKvError> {
        if !max_mse.is_finite() || max_mse < 0.0 {
            return Err(PolyKvError::Codec(
                "fibquant max_mse must be finite and nonnegative".to_string(),
            ));
        }
        self.max_mse = Some(max_mse);
        Ok(self)
    }

    /// Return the canonical FibQuant KV profile digest.
    pub fn fib_profile_digest(&self) -> &str {
        &self.profile_digest
    }

    /// Build a FibScorer for compressed-domain attention scoring.
    #[cfg(feature = "fibquant-adapter")]
    pub fn build_scorer(&self, seed: u64) -> Result<fib_quant::FibScorer, PolyKvError> {
        let fib_profile =
            FibQuantProfileV1::paper_default(self.head_dim, 4, 32, seed).map_err(fib_error)?;
        let mut fp = fib_profile;
        fp.training_samples = 128;
        fp.lloyd_restarts = 1;
        fp.lloyd_iterations = 2;
        let quantizer = FibQuantizer::new(fp).map_err(fib_error)?;
        fib_quant::FibScorer::new(quantizer)
            .map_err(|e| PolyKvError::Codec(format!("fib scorer build: {e}")))
    }

    /// Decode FQKV wire bytes into FibCodeV1 codes for compressed scoring.
    /// Returns codes in (layer, head, token) order matching the tensor layout.
    #[cfg(feature = "fibquant-adapter")]
    pub fn decode_to_fib_codes(
        &self,
        wire_bytes: &[u8],
    ) -> Result<Vec<fib_quant::FibCodeV1>, PolyKvError> {
        let encoded = decode_kv_wire(wire_bytes).map_err(fib_error)?;
        let mut codes = Vec::new();
        for page in &encoded.pages {
            for block in &page.encoded_blocks {
                if let fib_quant::kv::KvBlockEncodingV1::FibQuant { ref code } = block.encoding {
                    codes.push((**code).clone());
                }
            }
        }
        Ok(codes)
    }

    fn encode_envelope(&self, input: &[f32]) -> Result<KvEncodedTensorV1, PolyKvError> {
        self.validate_input(input)?;
        let shape = self.shape_for_len(input.len())?;
        let profile = self.profile_for_shape(&shape)?;
        let layout = KvCacheLayoutV1::canonical(&shape).map_err(fib_error)?;
        encode_kv_tensor(shape, layout, profile, input).map_err(fib_error)
    }

    fn validate_input(&self, input: &[f32]) -> Result<(), PolyKvError> {
        if input.is_empty() || input.len() % self.head_dim != 0 {
            return Err(PolyKvError::Codec(format!(
                "fibquant value dimension must be a nonzero multiple of {}: input={}",
                self.head_dim,
                input.len()
            )));
        }
        if input.iter().any(|value| !value.is_finite()) {
            return Err(PolyKvError::Codec(
                "fibquant value input contains NaN or infinite value".to_string(),
            ));
        }
        Ok(())
    }

    fn shape_for_len(&self, len: usize) -> Result<KvTensorShapeV1, PolyKvError> {
        let tokens =
            u32::try_from(len / self.head_dim).map_err(|err| PolyKvError::InvalidShape {
                reason: err.to_string(),
            })?;
        Ok(KvTensorShapeV1::new(
            KvRole::Value,
            KvAttentionKind::Mha,
            1,
            1,
            1,
            1,
            tokens,
            self.shape.head_dim,
            KvDType::F32,
            KvRopeState::NotApplicable,
        ))
    }

    fn profile_for_shape(
        &self,
        shape: &KvTensorShapeV1,
    ) -> Result<KvCompressionProfileV1, PolyKvError> {
        let raw_bytes = shape
            .head_dim
            .checked_mul(4)
            .and_then(|bytes| bytes.checked_add(1024))
            .ok_or_else(|| PolyKvError::InvalidShape {
                reason: "encoded block reservation overflow".to_string(),
            })?;
        let page_geometry = KvPageGeometryV1::new(1, shape.head_dim, raw_bytes);
        KvCompressionProfileV1::from_parts(
            self.profile.profile_id.clone(),
            shape,
            self.profile.fib_profile.clone(),
            self.profile.codebook_digest.clone(),
            self.profile.axis_policy,
            page_geometry,
        )
        .map_err(fib_error)
    }

    fn decode_envelope(&self, block: &[u8]) -> Result<Vec<f32>, PolyKvError> {
        if block.len() as u64 > MAX_ADAPTER_ENCODED_BYTES {
            return Err(PolyKvError::Codec(format!(
                "fibquant payload exceeds {MAX_ADAPTER_ENCODED_BYTES} byte limit"
            )));
        }
        let encoded: KvEncodedTensorV1 = decode_kv_wire(block).map_err(fib_error)?;
        if encoded.shape.role != KvRole::Value
            || encoded.shape.head_dim != self.shape.head_dim
            || encoded.shape.attention_kind != KvAttentionKind::Mha
            || encoded.shape.batch != 1
            || encoded.shape.layers != 1
            || encoded.shape.kv_heads != 1
            || encoded.shape.query_heads != 1
        {
            return Err(PolyKvError::ShapeMismatch {
                reason: "fibquant encoded shape does not match codec profile".to_string(),
            });
        }
        let expected_profile = self.profile_for_shape(&encoded.shape)?;
        let expected_profile_digest = expected_profile.digest(&encoded.shape).map_err(fib_error)?;
        let actual_profile_digest = encoded.profile.digest(&encoded.shape).map_err(fib_error)?;
        if actual_profile_digest != expected_profile_digest {
            return Err(PolyKvError::Codec(format!(
                "fibquant profile mismatch: expected {}, got {}",
                expected_profile_digest, actual_profile_digest
            )));
        }
        decode_kv_pages(&encoded)
            .map(|decoded| decoded.values)
            .map_err(fib_error)
    }
}

impl CodecProfile for FibQuantValueCodec {
    fn codec_id(&self) -> CodecId {
        CodecId::new("poly-kv:value:fibquant").expect("static codec id is valid")
    }

    fn codec_version(&self) -> &str {
        "0.1.0-alpha.1"
    }

    fn profile_digest(&self) -> CodecProfileDigest {
        let quality_budget = self
            .max_mse
            .map(|value| format!("mse:{:016x}", value.to_bits()))
            .unwrap_or_else(|| "mse:unconfigured".to_string());
        CodecProfileDigest::from_parts(&[
            self.codec_id().as_str().as_bytes(),
            self.codec_version().as_bytes(),
            self.profile_digest.as_bytes(),
            quality_budget.as_bytes(),
        ])
    }

    fn fixed_rate_bits(&self) -> Option<u16> {
        let index_bits = u16::from(self.profile.fib_profile.wire_index_bits);
        let block_dim = u16::try_from(self.profile.fib_profile.block_dim).ok()?;
        (index_bits % block_dim == 0).then_some(index_bits / block_dim)
    }

    fn block_dim(&self) -> Option<u16> {
        u16::try_from(self.profile.fib_profile.block_dim).ok()
    }

    fn is_lossy(&self) -> bool {
        true
    }
}

impl VectorCodec for FibQuantValueCodec {
    type EncodedBlock = Vec<u8>;
    type Error = PolyKvError;

    fn profile(&self) -> &dyn CodecProfile {
        self
    }

    fn capabilities(&self) -> CodecCapabilities {
        CodecCapabilities {
            can_encode: true,
            can_decode: true,
            can_score_inner_product: false,
            can_score_l2: false,
            is_lossless: false,
        }
    }

    fn resource_limits(&self) -> CodecResourceLimits {
        CodecResourceLimits {
            max_dim: self.head_dim as u32,
            max_encoded_bytes: MAX_ADAPTER_ENCODED_BYTES,
            max_batch_size: 1,
        }
    }

    fn encode_block(&self, input: &[f32]) -> Result<Self::EncodedBlock, Self::Error> {
        self.encode_values(input)
    }

    fn decode_block(&self, block: &Self::EncodedBlock, out: &mut [f32]) -> Result<(), Self::Error> {
        self.decode_values(block, out)
    }

    fn score_semantics(&self) -> ScoreSemantics {
        ScoreSemantics::CosineOnDecodedF32
    }
}

impl crate::codecs::value::ValueCodec for FibQuantValueCodec {
    fn encode_values(&self, input: &[f32]) -> Result<Vec<u8>, PolyKvError> {
        let encoded = self.encode_envelope(input)?;
        let bytes = encode_kv_wire(&encoded).map_err(fib_error)?;
        if bytes.len() as u64 > MAX_ADAPTER_ENCODED_BYTES {
            return Err(PolyKvError::Codec(format!(
                "fibquant payload exceeds {MAX_ADAPTER_ENCODED_BYTES} byte limit"
            )));
        }
        Ok(bytes)
    }

    fn decode_values(&self, block: &[u8], out: &mut [f32]) -> Result<(), PolyKvError> {
        let decoded = self.decode_envelope(block)?;
        if out.len() != decoded.len() {
            return Err(PolyKvError::Codec(format!(
                "fibquant value decode length mismatch: out={}, decoded={}",
                out.len(),
                decoded.len()
            )));
        }
        out.copy_from_slice(&decoded);
        Ok(())
    }

    fn eval_values(&self, exact: &[f32], block: &[u8]) -> Result<EvalReport, PolyKvError> {
        self.validate_input(exact)?;
        let mut decoded = vec![0.0; exact.len()];
        self.decode_values(block, &mut decoded)?;
        let mut report = metrics::eval_report(
            exact,
            &decoded,
            (exact.len() as u64) * 4,
            block.len() as u64,
            self.max_mse.unwrap_or(0.0),
            "fibquant canonical KV envelope; fallback status remains inside the FibQuant receipt",
        );
        if self.max_mse.is_none() {
            report.passed = false;
            report
                .notes
                .push("fibquant quality budget is not configured".to_string());
        }
        Ok(report)
    }
}

fn fib_error(error: impl std::fmt::Display) -> PolyKvError {
    PolyKvError::Codec(error.to_string())
}
