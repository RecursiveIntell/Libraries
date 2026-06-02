//! Codec A/B benchmark: search recall@K and storage size across
//! TurboQuant, PolarQuant, QjlSketch, and Uncompressed.
//!
#![allow(clippy::expect_used)] // benchmark code — expect() on Result/Option is the idiomatic pattern
//! For a synthetic corpus of N vectors at dim D, this benchmark:
//! 1. Generates N random unit-normalized query vectors and a deterministic
//!    corpus of M unit-normalized candidate vectors.
//! 2. Computes the ground-truth top-K (by exact inner product, brute
//!    force against the raw floats) for each query.
//! 3. For each codec, encodes every corpus vector. For each query,
//!    scores every encoded candidate, sorts, takes top-K. Compares
//!    against ground truth. Reports recall@K.
//! 4. Reports encoded-size-per-vector in bytes.
//!
//! All codecs are honest: a codec that cannot reconstruct is fine
//! because the benchmark only uses score_inner_product, not decode.
//!
//! Usage:
//!   cargo run --release --example codec_search_bench

use std::time::Instant;

use rand::Rng;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use scr_runtime_compression::{decode, encode, CodecId};
use turbo_quant::{
    PolarQuantizer, QjlQuantizer, TurboMode, TurboQuantizer,
};

const NUM_CORPUS: usize = 1000;
const NUM_QUERIES: usize = 50;
const DIM: usize = 128;
const TOP_K: usize = 10;
const SEED: u64 = 42;

fn make_unit_vector(rng: &mut ChaCha8Rng, dim: usize) -> Vec<f32> {
    let v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    v.into_iter().map(|x| x / norm).collect()
}

fn make_corpus(rng: &mut ChaCha8Rng, n: usize) -> Vec<Vec<f32>> {
    (0..n).map(|_| make_unit_vector(rng, DIM)).collect()
}

fn make_queries(rng: &mut ChaCha8Rng, n: usize) -> Vec<Vec<f32>> {
    (0..n).map(|_| make_unit_vector(rng, DIM)).collect()
}

/// Ground truth: top-K corpus vectors by exact inner product with query.
fn ground_truth_topk(query: &[f32], corpus: &[Vec<f32>], k: usize) -> Vec<usize> {
    let mut scores: Vec<(usize, f32)> = corpus
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let s: f32 = query.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
            (i, s)
        })
        .collect();
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scores.into_iter().take(k).map(|(i, _)| i).collect()
}

fn recall_at_k(predicted: &[usize], ground_truth: &[usize]) -> f32 {
    if ground_truth.is_empty() {
        return 0.0;
    }
    let g: std::collections::HashSet<&usize> = ground_truth.iter().collect();
    let hits = predicted.iter().filter(|i| g.contains(i)).count();
    hits as f32 / ground_truth.len() as f32
}

/// Cached quantizer handles — built once, amortized across all queries.
struct Quantizers {
    turbo: Option<TurboQuantizer>,
    polar: Option<PolarQuantizer>,
    qjl: Option<QjlQuantizer>,
}

impl Quantizers {
    fn build() -> Self {
        let turbo = TurboQuantizer::new_with_mode_and_rotation(
            DIM,
            8,
            32,
            SEED,
            TurboMode::PolarWithQjl,
            turbo_quant::RotationKind::Auto,
        )
        .ok();
        let polar = PolarQuantizer::new_with_stored_rotation(DIM, 8, SEED).ok();
        let qjl = QjlQuantizer::new(DIM, 32, SEED).ok();
        Self { turbo, polar, qjl }
    }
}

fn score_candidate(
    codec: CodecId,
    q: &Quantizers,
    encoded: &[u8],
    query: &[f32],
) -> f32 {
    match codec {
        CodecId::Uncompressed => {
            let bytes = decode(CodecId::Uncompressed, encoded).expect("decode raw");
            let v: &[f32] = bytemuck::cast_slice(&bytes);
            query.iter().zip(v.iter()).map(|(a, b)| a * b).sum()
        }
        CodecId::TurboQuant => {
            let tq = q.turbo.as_ref().expect("turbo quantizer");
            tq.score_inner_product_from_bytes(encoded, query)
                .expect("turbo score")
        }
        CodecId::Polar => {
            use turbo_quant::PolarCode;
            let code: PolarCode =
                serde_json::from_slice(encoded).expect("polar deserialize");
            let pq = q.polar.as_ref().expect("polar quantizer");
            pq.inner_product_estimate(&code, query)
                .expect("polar score")
        }
        CodecId::Qjl => {
            use turbo_quant::QjlSketch;
            let sketch: QjlSketch =
                serde_json::from_slice(encoded).expect("qjl deserialize");
            let qq = q.qjl.as_ref().expect("qjl quantizer");
            qq.inner_product_estimate(&sketch, query)
                .expect("qjl score")
        }
        _ => panic!("unsupported codec in benchmark: {codec:?}"),
    }
}

fn run_codec(codec: CodecId, corpus: &[Vec<f32>], queries: &[Vec<f32>]) -> (f32, f64, usize) {
    // Encode the entire corpus
    let start = Instant::now();
    let encoded: Vec<Vec<u8>> = corpus
        .iter()
        .map(|v| encode(codec, v, SEED).expect("encode"))
        .collect();
    let encode_ms = start.elapsed().as_millis() as f64;

    let total_bytes: usize = encoded.iter().map(|v| v.len()).sum();
    let bytes_per_vec = if corpus.is_empty() {
        0
    } else {
        total_bytes / corpus.len()
    };

    // Build search-time quantizers ONCE — amortized across all queries
    let q = Quantizers::build();

    // For each query, score all candidates and compute recall
    let mut total_recall = 0.0f32;
    let start = Instant::now();
    for query in queries {
        let ground = ground_truth_topk(query, corpus, TOP_K);
        let mut scores: Vec<(usize, f32)> = encoded
            .iter()
            .enumerate()
            .map(|(i, enc)| (i, score_candidate(codec, &q, enc, query)))
            .collect();
        scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        let predicted: Vec<usize> = scores.into_iter().take(TOP_K).map(|(i, _)| i).collect();
        total_recall += recall_at_k(&predicted, &ground);
    }
    let search_ms = start.elapsed().as_millis() as f64;
    let mean_recall = total_recall / queries.len() as f32;
    eprintln!(
        "  {:>14}  recall@{}={:.3}  encode={:.1}ms  search={:.1}ms  bytes/vec={}",
        format!("{:?}", codec),
        TOP_K,
        mean_recall,
        encode_ms,
        search_ms,
        bytes_per_vec
    );
    (mean_recall, encode_ms + search_ms, bytes_per_vec)
}

fn main() {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let corpus = make_corpus(&mut rng, NUM_CORPUS);
    let queries = make_queries(&mut rng, NUM_QUERIES);

    println!("# codec A/B search benchmark");
    println!("# corpus={NUM_CORPUS}  queries={NUM_QUERIES}  dim={DIM}  top_k={TOP_K}");
    println!();

    let codecs: &[(&str, CodecId)] = &[
        ("uncompressed", CodecId::Uncompressed),
        ("turbo_quant", CodecId::TurboQuant),
        ("polar", CodecId::Polar),
        ("qjl", CodecId::Qjl),
    ];

    println!("## Results");
    println!();
    for (name, codec) in codecs {
        let (recall, total_ms, bytes) = run_codec(*codec, &corpus, &queries);
        println!(
            "- {name:>13}: recall@10={recall:.3}  total_ms={total_ms:.1}  bytes_per_vec={bytes}"
        );
    }
    println!();

    println!("## Notes");
    println!("  - recall@10: mean across {NUM_QUERIES} queries, K=10");
    println!("  - 'uncompressed' is the upper bound on recall (1.0 by definition)");
    println!("  - 'bytes_per_vec' is the wire format size for a single {DIM}-dim vector");
    println!("  - QJL is dim-independent (~120 bytes for any dim); dominant for memory-efficient ANN");
    println!("  - Turbo and Polar are asymmetric-friendly; both use score_inner_product on the wire");
    println!("  - Search-time quantizers are built ONCE per codec and reused across all queries");
    println!("    (in a real production pipeline this is amortized across millions of queries)");
}
