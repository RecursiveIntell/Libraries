use crate::PolyKvError;
use quant_codec_core::{CodecId, CodecProfile, CodecProfileDigest, EvalReport, VectorCodec};

/// Object-safe value-codec boundary used by [`crate::PoolBuilder`].
///
/// Encoded blocks are canonical byte strings so a non-generic shared pool can
/// retain one codec instance and dispatch decode without backend-specific
/// storage variants.
pub trait ValueCodec: CodecProfile + std::fmt::Debug + Send + Sync {
    fn encode_values(&self, input: &[f32]) -> Result<Vec<u8>, PolyKvError>;
    fn decode_values(&self, block: &[u8], out: &mut [f32]) -> Result<(), PolyKvError>;
    fn eval_values(&self, exact: &[f32], block: &[u8]) -> Result<EvalReport, PolyKvError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RawExactValueCodec;

impl CodecProfile for RawExactValueCodec {
    fn codec_id(&self) -> CodecId {
        CodecId::new("poly-kv:value:raw-exact").expect("static codec id is valid")
    }

    fn codec_version(&self) -> &str {
        "0.1.0-alpha.1"
    }

    fn profile_digest(&self) -> CodecProfileDigest {
        CodecProfileDigest::from_parts(&[
            self.codec_id().as_str().as_bytes(),
            self.codec_version().as_bytes(),
            b"raw-f32-le-v1",
        ])
    }

    fn fixed_rate_bits(&self) -> Option<u16> {
        Some(32)
    }

    fn block_dim(&self) -> Option<u16> {
        None
    }

    fn is_lossy(&self) -> bool {
        false
    }
}

impl VectorCodec for RawExactValueCodec {
    type EncodedBlock = Vec<u8>;
    type Error = PolyKvError;

    fn profile(&self) -> &dyn quant_codec_core::CodecProfile {
        self
    }

    fn capabilities(&self) -> quant_codec_core::CodecCapabilities {
        quant_codec_core::CodecCapabilities {
            can_encode: true,
            can_decode: true,
            can_score_inner_product: false,
            can_score_l2: false,
            is_lossless: true,
        }
    }

    fn resource_limits(&self) -> quant_codec_core::CodecResourceLimits {
        quant_codec_core::CodecResourceLimits::default()
    }

    fn encode_block(&self, input: &[f32]) -> Result<Self::EncodedBlock, Self::Error> {
        self.encode_values(input)
    }

    fn decode_block(&self, block: &Self::EncodedBlock, out: &mut [f32]) -> Result<(), Self::Error> {
        self.decode_values(block, out)
    }

    fn score_semantics(&self) -> quant_codec_core::ScoreSemantics {
        quant_codec_core::ScoreSemantics::CosineOnDecodedF32
    }
}

impl ValueCodec for RawExactValueCodec {
    fn encode_values(&self, input: &[f32]) -> Result<Vec<u8>, PolyKvError> {
        if !input.iter().all(|v| v.is_finite()) {
            return Err(PolyKvError::Codec(
                "raw exact value input contains NaN or infinite value".to_string(),
            ));
        }
        let capacity = input.len().checked_mul(4).ok_or_else(|| {
            PolyKvError::Codec("raw exact value encoded length overflow".to_string())
        })?;
        let mut encoded = Vec::with_capacity(capacity);
        for value in input {
            encoded.extend_from_slice(&value.to_le_bytes());
        }
        Ok(encoded)
    }

    fn decode_values(&self, block: &[u8], out: &mut [f32]) -> Result<(), PolyKvError> {
        let expected_bytes = out.len().checked_mul(4).ok_or_else(|| {
            PolyKvError::Codec("raw exact value decode length overflow".to_string())
        })?;
        if block.len() != expected_bytes {
            return Err(PolyKvError::Codec(format!(
                "raw exact value decode length mismatch: out={}, block_bytes={}",
                out.len(),
                block.len()
            )));
        }
        for (destination, bytes) in out.iter_mut().zip(block.chunks_exact(4)) {
            *destination = f32::from_le_bytes(bytes.try_into().expect("chunk length is four"));
        }
        Ok(())
    }

    fn eval_values(&self, exact: &[f32], block: &[u8]) -> Result<EvalReport, PolyKvError> {
        let mut decoded = vec![0.0; exact.len()];
        self.decode_values(block, &mut decoded)?;
        if decoded != exact {
            return Err(PolyKvError::Codec("raw exact eval mismatch".to_string()));
        }
        Ok(EvalReport::exact((exact.len() as u64) * 4))
    }
}
