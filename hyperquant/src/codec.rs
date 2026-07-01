use crate::{HyperQuantConfig, HyperQuantError, HyperQuantResult, LatticeKind, Result};
use quant_codec_core::{CodecId, CodecProfile, CodecProfileDigest, VectorCodec};

/// Feature-gated adapter exposing HyperQuant through quant-codec-core traits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HyperQuantCodec {
    config: HyperQuantConfig,
}

/// Encoded lossy block produced by the compat adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct HyperQuantEncodedBlock {
    pub result: HyperQuantResult,
}

impl HyperQuantCodec {
    pub fn new(kind: LatticeKind, scale: f32) -> Self {
        Self {
            config: HyperQuantConfig::new(kind, scale),
        }
    }

    pub fn config(&self) -> HyperQuantConfig {
        self.config
    }
}

impl CodecProfile for HyperQuantCodec {
    fn codec_id(&self) -> CodecId {
        match CodecId::new("hyperquant") {
            Ok(id) => id,
            Err(err) => unreachable!("static valid codec id rejected: {err}"),
        }
    }

    fn codec_version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn profile_digest(&self) -> CodecProfileDigest {
        let kind = [self.config.kind as u8];
        let scale = self.config.effective_scale().to_bits().to_le_bytes();
        CodecProfileDigest::from_parts(&[b"hyperquant-compat-v1", &kind, &scale])
    }

    fn fixed_rate_bits(&self) -> Option<u16> {
        Some(16)
    }

    fn block_dim(&self) -> Option<u16> {
        match self.config.kind {
            LatticeKind::Z1 => Some(1),
            LatticeKind::A2 => Some(2),
            LatticeKind::D4 => Some(4),
            LatticeKind::E8 => Some(8),
        }
    }

    fn is_lossy(&self) -> bool {
        true
    }
}

impl VectorCodec for HyperQuantCodec {
    type EncodedBlock = HyperQuantEncodedBlock;
    type Error = HyperQuantError;

    fn encode_block(&self, input: &[f32]) -> Result<Self::EncodedBlock> {
        Ok(HyperQuantEncodedBlock {
            result: self.config.quantize(input)?,
        })
    }

    fn decode_block(&self, block: &Self::EncodedBlock, out: &mut [f32]) -> Result<()> {
        if out.len() != block.result.reconstructed.len() {
            return Err(HyperQuantError::DecodeLengthMismatch {
                expected: block.result.reconstructed.len(),
                actual: out.len(),
            });
        }
        out.copy_from_slice(&block.result.reconstructed);
        Ok(())
    }
}
