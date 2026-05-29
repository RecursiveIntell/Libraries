use crate::PolyKvError;
use quant_codec_core::{ArtifactDigest, KvRole, KvTensorShape, LayerId, VectorCodec};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExactKvBlock {
    pub role: KvRole,
    pub layer: LayerId,
    pub shape: KvTensorShape,
    pub data: Vec<f32>,
}

impl ExactKvBlock {
    pub fn new(
        role: KvRole,
        layer: LayerId,
        shape: KvTensorShape,
        data: Vec<f32>,
    ) -> Result<Self, PolyKvError> {
        shape.validate()?;
        if layer.0 >= shape.layers {
            return Err(PolyKvError::ShapeMismatch {
                reason: format!("layer {} is outside shape layers {}", layer.0, shape.layers),
            });
        }
        let expected = shape.layer_element_count(role)?;
        if data.len() != expected {
            return Err(PolyKvError::ShapeMismatch {
                reason: format!(
                    "block has {} values but expected {} for {:?} layer",
                    data.len(),
                    expected,
                    role
                ),
            });
        }
        if !data.iter().all(|v| v.is_finite()) {
            return Err(PolyKvError::Codec(
                "exact block contains NaN or infinite value".to_string(),
            ));
        }
        Ok(Self {
            role,
            layer,
            shape,
            data,
        })
    }

    pub fn exact_bytes(&self) -> u64 {
        (self.data.len() as u64) * 4
    }

    pub fn artifact_digest(&self) -> ArtifactDigest {
        digest_block(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExactFallback {
    pub blocks: Vec<ExactKvBlock>,
}

pub type ExactFallbackRef = ExactFallback;

impl ExactFallback {
    pub fn from_blocks(blocks: Vec<ExactKvBlock>) -> Self {
        Self {
            blocks: sort_blocks(blocks),
        }
    }

    pub fn exact_bytes(&self) -> u64 {
        self.blocks.iter().map(ExactKvBlock::exact_bytes).sum()
    }

    pub fn find(&self, role: KvRole, layer: LayerId) -> Option<&ExactKvBlock> {
        self.blocks
            .iter()
            .find(|block| block.role == role && block.layer == layer)
    }

    pub fn input_digest(&self) -> ArtifactDigest {
        digest_blocks(&self.blocks)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RawExactCodec;

impl VectorCodec for RawExactCodec {
    type EncodedBlock = Vec<f32>;
    type Error = PolyKvError;

    fn encode_block(&self, input: &[f32]) -> Result<Self::EncodedBlock, Self::Error> {
        if !input.iter().all(|v| v.is_finite()) {
            return Err(PolyKvError::Codec(
                "raw exact input contains NaN or infinite value".to_string(),
            ));
        }
        Ok(input.to_vec())
    }

    fn decode_block(&self, block: &Self::EncodedBlock, out: &mut [f32]) -> Result<(), Self::Error> {
        if out.len() != block.len() {
            return Err(PolyKvError::Codec(format!(
                "raw exact decode length mismatch: out={}, block={}",
                out.len(),
                block.len()
            )));
        }
        out.copy_from_slice(block);
        Ok(())
    }
}

pub(crate) fn sort_blocks(mut blocks: Vec<ExactKvBlock>) -> Vec<ExactKvBlock> {
    blocks.sort_by_key(|block| (block.layer.0, block.role));
    blocks
}

pub(crate) fn digest_blocks(blocks: &[ExactKvBlock]) -> ArtifactDigest {
    let block_digests = blocks
        .iter()
        .map(|block| block.artifact_digest().to_string())
        .collect::<Vec<_>>();
    let refs = block_digests
        .iter()
        .map(String::as_bytes)
        .collect::<Vec<_>>();
    ArtifactDigest::from_parts(&refs)
}

fn digest_block(block: &ExactKvBlock) -> ArtifactDigest {
    let mut bytes = Vec::with_capacity(block.data.len() * 4 + 64);
    bytes.extend_from_slice(format!("{:?}", block.role).as_bytes());
    bytes.extend_from_slice(&block.layer.0.to_le_bytes());
    bytes.extend_from_slice(&block.shape.layers.to_le_bytes());
    bytes.extend_from_slice(&block.shape.key_heads.to_le_bytes());
    bytes.extend_from_slice(&block.shape.value_heads.to_le_bytes());
    bytes.extend_from_slice(&block.shape.seq_len.to_le_bytes());
    bytes.extend_from_slice(&block.shape.head_dim.to_le_bytes());
    bytes.extend_from_slice(format!("{:?}{:?}", block.shape.layout, block.shape.dtype).as_bytes());
    for value in &block.data {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    ArtifactDigest::from_canonical_bytes(&bytes)
}
