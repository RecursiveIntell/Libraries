#![allow(dead_code)]

use poly_kv::*;

pub fn shape_mha() -> KvTensorShape {
    KvTensorShape::gqa(2, 2, 2, 8, 4, KvLayout::LayersHeadsTokensDim, DType::F32).unwrap()
}

pub fn shape_mqa() -> KvTensorShape {
    KvTensorShape::gqa(2, 1, 4, 8, 4, KvLayout::LayersHeadsTokensDim, DType::F32).unwrap()
}

pub fn shape_gqa() -> KvTensorShape {
    KvTensorShape::gqa(2, 2, 8, 8, 4, KvLayout::LayersHeadsTokensDim, DType::F32).unwrap()
}

pub fn blocks_for(shape: &KvTensorShape) -> Vec<ExactKvBlock> {
    let mut blocks = Vec::new();
    for layer in 0..shape.layers {
        for role in [KvRole::Key, KvRole::Value] {
            let len = shape.layer_element_count(role).unwrap();
            let data = (0..len)
                .map(|idx| {
                    let centered = (idx as i32 % 17) - 8;
                    let role_offset = if role == KvRole::Key { 0.0 } else { 0.125 };
                    centered as f32 / 64.0 + role_offset + layer as f32 / 100.0
                })
                .collect::<Vec<_>>();
            blocks.push(ExactKvBlock::new(role, LayerId(layer), shape.clone(), data).unwrap());
        }
    }
    blocks
}

pub fn build_pool(shape: KvTensorShape) -> SharedKvPool {
    let blocks = blocks_for(&shape);
    SharedKvPool::builder()
        .model_fingerprint(ModelFingerprint::new("synthetic:test-model").unwrap())
        .tokenizer_fingerprint(TokenizerFingerprint::new("synthetic:test-tokenizer").unwrap())
        .shape(shape)
        .policy(CompressionPolicyV1::alpha_reference())
        .exact_fallback(ExactFallback::from_blocks(blocks.clone()))
        .key_codec(Q8KeyCodec::symmetric_per_block())
        .value_codec(RawExactValueCodec)
        .build_from_blocks(blocks)
        .unwrap()
}
