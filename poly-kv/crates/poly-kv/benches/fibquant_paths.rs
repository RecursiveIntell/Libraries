#![cfg(feature = "fibquant-adapter")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fib_quant::kv::{decode_kv_pages, decode_kv_wire, KvEncodedTensorV1};
use poly_kv::adapters::fibquant::FibQuantValueCodec;
use poly_kv::*;
use quant_codec_core::{HeadId, KvRole, KvSliceRequest, LayerId, TokenSpan};

fn fixture() -> (
    Vec<f32>,
    Vec<u8>,
    Vec<u8>,
    Vec<u8>,
    SharedKvPool,
    SharedKvPool,
) {
    let shape =
        KvTensorShape::gqa(1, 2, 2, 8, 4, KvLayout::LayersHeadsTokensDim, DType::F32).unwrap();
    let values = (0..shape.layer_element_count(KvRole::Value).unwrap())
        .map(|idx| ((idx as i32 % 17) - 8) as f32 / 64.0)
        .collect::<Vec<_>>();
    let blocks = [KvRole::Key, KvRole::Value]
        .into_iter()
        .map(|role| {
            let len = shape.layer_element_count(role).unwrap();
            let data = (0..len)
                .map(|idx| ((idx as i32 % 17) - 8) as f32 / 64.0)
                .collect::<Vec<_>>();
            ExactKvBlock::new(role, LayerId(0), shape.clone(), data).unwrap()
        })
        .collect::<Vec<_>>();

    let raw_codec = RawExactValueCodec;
    let raw_payload = raw_codec.encode_values(&values).unwrap();
    let fib_codec = FibQuantValueCodec::new(4, 2, 4, 7)
        .unwrap()
        .with_max_mse(1.0)
        .unwrap();
    let wire_payload = fib_codec.encode_values(&values).unwrap();
    let tensor: KvEncodedTensorV1 = decode_kv_wire(&wire_payload).unwrap();
    let json_payload = serde_json::to_vec(&tensor).unwrap();

    let base = || {
        SharedKvPool::builder()
            .model_fingerprint(ModelFingerprint::new("bench:model").unwrap())
            .tokenizer_fingerprint(TokenizerFingerprint::new("bench:tokenizer").unwrap())
            .shape(shape.clone())
            .exact_fallback(ExactFallback::from_blocks(blocks.clone()))
            .key_codec(Q8KeyCodec::symmetric_per_block())
    };
    let raw_pool = base()
        .value_codec(RawExactValueCodec)
        .build_from_blocks(blocks.clone())
        .unwrap();
    let mut policy = CompressionPolicyV1::alpha_reference();
    policy.quality_gate.max_value_mse = 1.0;
    let hybrid_pool = base()
        .policy(policy)
        .value_codec(fib_codec)
        .build_from_blocks(blocks)
        .unwrap();

    (
        values,
        raw_payload,
        json_payload,
        wire_payload,
        raw_pool,
        hybrid_pool,
    )
}

fn benchmark_paths(c: &mut Criterion) {
    let (values, raw_payload, json_payload, wire_payload, raw_pool, hybrid_pool) = fixture();
    println!(
        "fibquant-path-accounting raw_payload={} json_candidate={} framed_wire={} exact_fallback_payload={} hybrid_resident={} hybrid_manifest={} ",
        raw_payload.len(),
        json_payload.len(),
        wire_payload.len(),
        values.len() * std::mem::size_of::<f32>(),
        hybrid_pool.memory_accounting().total_bytes(),
        hybrid_pool.manifest().encoded_bytes,
    );

    c.bench_function("fibquant_raw_encode", |b| {
        b.iter(|| {
            black_box(
                RawExactValueCodec
                    .encode_values(black_box(&values))
                    .unwrap(),
            )
        })
    });
    c.bench_function("fibquant_raw_decode", |b| {
        b.iter(|| {
            let mut out = vec![0.0; values.len()];
            RawExactValueCodec
                .decode_values(black_box(&raw_payload), &mut out)
                .unwrap();
            black_box(out);
        })
    });
    c.bench_function("fibquant_json_decode_candidate", |b| {
        b.iter(|| {
            let tensor: KvEncodedTensorV1 =
                serde_json::from_slice(black_box(&json_payload)).unwrap();
            black_box(decode_kv_pages(&tensor).unwrap());
        })
    });
    c.bench_function("fibquant_wire_decode", |b| {
        b.iter(|| {
            let tensor = decode_kv_wire(black_box(&wire_payload)).unwrap();
            black_box(decode_kv_pages(&tensor).unwrap());
        })
    });

    let request = KvSliceRequest {
        layer: LayerId(0),
        role: KvRole::Value,
        token_span: TokenSpan::new(0, 8).unwrap(),
        head: Some(HeadId(0)),
    };
    let raw_reader = raw_pool.attach_reader(ReaderConfig::default()).unwrap();
    let hybrid_reader = hybrid_pool.attach_reader(ReaderConfig::default()).unwrap();
    c.bench_function("pool_raw_reader_decode", |b| {
        b.iter(|| black_box(raw_reader.decode_slice(black_box(request.clone())).unwrap()))
    });
    c.bench_function("pool_hybrid_reader_decode", |b| {
        b.iter(|| {
            black_box(
                hybrid_reader
                    .decode_slice(black_box(request.clone()))
                    .unwrap(),
            )
        })
    });
    c.bench_function("pool_hybrid_exact_fallback_decode", |b| {
        b.iter(|| {
            black_box(
                hybrid_reader
                    .decode_slice_exact_fallback(black_box(request.clone()))
                    .unwrap(),
            )
        })
    });
}

criterion_group!(benches, benchmark_paths);
criterion_main!(benches);
