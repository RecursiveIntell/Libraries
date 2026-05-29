use criterion::{black_box, criterion_group, criterion_main, Criterion};
use poly_kv::*;

fn synthetic_blocks(shape: &KvTensorShape) -> Vec<ExactKvBlock> {
    let mut blocks = Vec::new();
    for layer in 0..shape.layers {
        for role in [KvRole::Key, KvRole::Value] {
            let len = shape.layer_element_count(role).unwrap();
            let data = (0..len)
                .map(|idx| ((idx as i32 % 17) - 8) as f32 / 64.0)
                .collect::<Vec<_>>();
            blocks.push(ExactKvBlock::new(role, LayerId(layer), shape.clone(), data).unwrap());
        }
    }
    blocks
}

fn build_synthetic_pool(c: &mut Criterion) {
    c.bench_function("synthetic_pool_build_mha", |b| {
        b.iter(|| {
            let shape =
                KvTensorShape::gqa(2, 2, 2, 8, 4, KvLayout::LayersHeadsTokensDim, DType::F32)
                    .unwrap();
            let blocks = synthetic_blocks(&shape);
            black_box(
                SharedKvPool::builder()
                    .model_fingerprint(ModelFingerprint::new("synthetic:test-model").unwrap())
                    .tokenizer_fingerprint(
                        TokenizerFingerprint::new("synthetic:test-tokenizer").unwrap(),
                    )
                    .shape(shape)
                    .exact_fallback(ExactFallback::from_blocks(blocks.clone()))
                    .build_from_blocks(blocks)
                    .unwrap(),
            );
        });
    });
}

criterion_group!(benches, build_synthetic_pool);
criterion_main!(benches);
