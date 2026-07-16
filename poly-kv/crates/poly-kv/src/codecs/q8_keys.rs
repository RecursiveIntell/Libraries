use crate::{metrics, PolyKvError};
use quant_codec_core::{CodecId, CodecProfile, CodecProfileDigest, EvalReport, VectorCodec};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Q8KeyBlock {
    pub scale: f32,
    pub original_len: u64,
    pub values: Vec<i8>,
}

impl Q8KeyBlock {
    pub fn encoded_bytes(&self) -> u64 {
        self.metadata_bytes() + self.values.len() as u64
    }

    pub fn metadata_bytes(&self) -> u64 {
        4 + 8
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Q8KeyCodec {
    max_mse: f64,
}

impl Q8KeyCodec {
    pub fn symmetric_per_block() -> Self {
        Self { max_mse: 0.001 }
    }

    pub fn eval(&self, exact: &[f32], block: &Q8KeyBlock) -> Result<EvalReport, PolyKvError> {
        let mut decoded = vec![0.0; exact.len()];
        self.decode_block(block, &mut decoded)?;
        Ok(metrics::eval_report(
            exact,
            &decoded,
            (exact.len() as u64) * 4,
            block.encoded_bytes(),
            self.max_mse,
            "q8 symmetric per-block key reference path",
        ))
    }
}

impl CodecProfile for Q8KeyCodec {
    fn codec_id(&self) -> CodecId {
        CodecId::new("poly-kv:q8-key:symmetric-per-block").expect("static codec id is valid")
    }

    fn codec_version(&self) -> &str {
        "0.1.0-alpha.1"
    }

    fn profile_digest(&self) -> CodecProfileDigest {
        CodecProfileDigest::from_parts(&[
            self.codec_id().as_str().as_bytes(),
            self.codec_version().as_bytes(),
            b"scale=max_abs/127",
        ])
    }

    fn fixed_rate_bits(&self) -> Option<u16> {
        Some(8)
    }

    fn block_dim(&self) -> Option<u16> {
        None
    }

    fn is_lossy(&self) -> bool {
        true
    }
}

impl VectorCodec for Q8KeyCodec {
    type EncodedBlock = Q8KeyBlock;
    type Error = PolyKvError;

    fn profile(&self) -> &dyn quant_codec_core::CodecProfile {
        unreachable!("poly-kv codecs do not implement CodecProfile")
    }

    fn capabilities(&self) -> quant_codec_core::CodecCapabilities {
        quant_codec_core::CodecCapabilities {
            can_encode: true,
            can_decode: true,
            can_score_inner_product: false,
            can_score_l2: false,
            is_lossless: false,
        }
    }

    fn resource_limits(&self) -> quant_codec_core::CodecResourceLimits {
        quant_codec_core::CodecResourceLimits::default()
    }

    fn encode_block(&self, input: &[f32]) -> Result<Self::EncodedBlock, Self::Error> {
        if input.is_empty() {
            return Err(PolyKvError::Codec("q8 input block is empty".to_string()));
        }
        if !input.iter().all(|v| v.is_finite()) {
            return Err(PolyKvError::Codec(
                "q8 input contains NaN or infinite value".to_string(),
            ));
        }
        let max_abs = input.iter().map(|v| v.abs()).fold(0.0_f32, f32::max);
        let scale = if max_abs == 0.0 { 1.0 } else { max_abs / 127.0 };
        let values = input
            .iter()
            .map(|value| (value / scale).round().clamp(-127.0, 127.0) as i8)
            .collect();
        Ok(Q8KeyBlock {
            scale,
            original_len: input.len() as u64,
            values,
        })
    }

    fn decode_block(&self, block: &Self::EncodedBlock, out: &mut [f32]) -> Result<(), Self::Error> {
        if out.len() != block.values.len() || block.original_len != block.values.len() as u64 {
            return Err(PolyKvError::Codec(format!(
                "q8 decode length mismatch: out={}, block={}",
                out.len(),
                block.values.len()
            )));
        }
        if !block.scale.is_finite() || block.scale <= 0.0 {
            return Err(PolyKvError::Codec("q8 block scale is invalid".to_string()));
        }
        for (dst, src) in out.iter_mut().zip(&block.values) {
            *dst = f32::from(*src) * block.scale;
        }
        Ok(())
    }
    fn score_semantics(&self) -> quant_codec_core::ScoreSemantics {
        quant_codec_core::ScoreSemantics::CosineOnDecodedF32
    }
}
