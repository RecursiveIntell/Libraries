use std::time::Instant;

use gpu_backend::{score_fib_gram_pages, topk_indices_desc, FibGramPageScoreInput, GpuContext};

fn next_u32(seed: &mut u64) -> u32 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*seed >> 32) as u32
}

fn main() {
    let n_candidates = std::env::var("N_CANDIDATES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4096usize);
    let block_count = std::env::var("BLOCK_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16usize);
    let n_codewords = 32usize;
    let mut seed = 42u64;

    let query_indices: Vec<u32> = (0..block_count)
        .map(|_| next_u32(&mut seed) % n_codewords as u32)
        .collect();
    let stored_indices: Vec<u32> = (0..n_candidates * block_count)
        .map(|_| next_u32(&mut seed) % n_codewords as u32)
        .collect();
    let stored_norms: Vec<f32> = (0..n_candidates)
        .map(|_| 0.5 + (next_u32(&mut seed) as f32 / u32::MAX as f32) * 2.0)
        .collect();
    let gram: Vec<f32> = (0..n_codewords * n_codewords)
        .map(|_| (next_u32(&mut seed) as f32 / u32::MAX as f32) * 2.0 - 1.0)
        .collect();

    let input = FibGramPageScoreInput {
        query_indices: &query_indices,
        stored_indices: &stored_indices,
        stored_norms: &stored_norms,
        gram: &gram,
        query_norm: 1.25,
        n_candidates,
        block_count,
        n_codewords,
    };

    let started = Instant::now();
    let scores = score_fib_gram_pages(input).expect("page scorer failed");
    let elapsed = started.elapsed();
    let top = topk_indices_desc(&scores, 8);
    let candidate_block_scores = n_candidates * block_count;
    let scores_per_sec = candidate_block_scores as f64 / elapsed.as_secs_f64();
    let gpu_available = GpuContext::is_available();
    let backend = if gpu_available && n_candidates >= GpuContext::GPU_MIN_BATCH_SIZE {
        "cuda_or_cuda_fallback"
    } else {
        "cpu"
    };

    println!(
        "{{\"schema_version\":\"fib_gram_page_scorer_bench_v1\",\"backend\":\"{}\",\"gpu_available\":{},\"n_candidates\":{},\"block_count\":{},\"n_codewords\":{},\"elapsed_us\":{},\"candidate_block_scores\":{},\"candidate_block_scores_per_sec\":{},\"top8\":{:?}}}",
        backend,
        gpu_available,
        n_candidates,
        block_count,
        n_codewords,
        elapsed.as_micros(),
        candidate_block_scores,
        scores_per_sec,
        top
    );
}
