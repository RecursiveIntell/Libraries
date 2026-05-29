use crate::PolyKvError;
use quant_codec_core::{CodecId, CodecProfile, CodecProfileDigest, EvalReport, VectorCodec};

pub trait ValueCodec: CodecProfile {
    type EncodedValueBlock;

    fn encode_values(&self, input: &[f32]) -> Result<Self::EncodedValueBlock, PolyKvError>;
    fn decode_values(
        &self,
        block: &Self::EncodedValueBlock,
        out: &mut [f32],
    ) -> Result<(), PolyKvError>;
    fn eval_values(
        &self,
        exact: &[f32],
        block: &Self::EncodedValueBlock,
    ) -> Result<EvalReport, PolyKvError>;
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
            b"raw-f32",
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
    type EncodedBlock = Vec<f32>;
    type Error = PolyKvError;

    fn encode_block(&self, input: &[f32]) -> Result<Self::EncodedBlock, Self::Error> {
        self.encode_values(input)
    }

    fn decode_block(&self, block: &Self::EncodedBlock, out: &mut [f32]) -> Result<(), Self::Error> {
        self.decode_values(block, out)
    }
}

impl ValueCodec for RawExactValueCodec {
    type EncodedValueBlock = Vec<f32>;

    fn encode_values(&self, input: &[f32]) -> Result<Self::EncodedValueBlock, PolyKvError> {
        if !input.iter().all(|v| v.is_finite()) {
            return Err(PolyKvError::Codec(
                "raw exact value input contains NaN or infinite value".to_string(),
            ));
        }
        Ok(input.to_vec())
    }

    fn decode_values(
        &self,
        block: &Self::EncodedValueBlock,
        out: &mut [f32],
    ) -> Result<(), PolyKvError> {
        if out.len() != block.len() {
            return Err(PolyKvError::Codec(format!(
                "raw exact value decode length mismatch: out={}, block={}",
                out.len(),
                block.len()
            )));
        }
        out.copy_from_slice(block);
        Ok(())
    }

    fn eval_values(
        &self,
        exact: &[f32],
        block: &Self::EncodedValueBlock,
    ) -> Result<EvalReport, PolyKvError> {
        if exact.len() != block.len() {
            return Err(PolyKvError::Codec(
                "raw exact eval length mismatch".to_string(),
            ));
        }
        Ok(EvalReport::exact((exact.len() as u64) * 4))
    }
}
