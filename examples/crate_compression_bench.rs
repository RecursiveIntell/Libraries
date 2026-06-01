//! GPU-accelerated compression benchmark across all three crates.
//!
//! Measures per-operation throughput with proper vector normalization
//! (matching real fib-quant encode path), then computes fidelity metrics.
//!
//! Run: cargo run --release --example crate_compression_bench --features gpu,precompiled-ptx
//!       (from Libraries workspace root)

#[cfg(feature = "gpu")]
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║    GPU-Accelerated Compression — Per-Crate Benchmark            ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let configs = vec![
        ("nomic-embed-text  (768-dim, 200 docs)", 768, 200),
        ("qwen3-embedding   (2560-dim, 50 docs)", 2560, 50),
    ];

    for (name, dim, num_docs) in configs.iter() {
        println!("{:=^70}", format!(" {} ", name));
        println!("  GPU available: {}\n", gpu_backend::GpuContext::is_available());

        let vectors = generate_normalized_vectors(*dim, *num_docs);
        let k: usize = 4;
        let n_levels: usize = 32;
        let seed: u64 = 42;

        // ═══════════════════════════════════════════════════════════
        // GPU BACKEND — RAW OPS
        // ═══════════════════════════════════════════════════════════
        println!("  ┌─ gpu-backend: Hadamard WHT ─────────────────────────────┐");
        let mut rotated = vectors.clone();
        let t0 = Instant::now();
        gpu_backend::hadamard_batch(&mut rotated, *num_docs, *dim, seed).unwrap();
        let hadamard_ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!("  │  {} vectors: {:.2}ms  ({:.0} vec/s)", num_docs, hadamard_ms, *num_docs as f64 / t0.elapsed().as_secs_f64());
        println!("  └──────────────────────────────────────────────────────────┘\n");

        println!("  ┌─ gpu-backend: Lloyd-Max Encode (k=4, N=32) ─────────────┐");
        let t0 = Instant::now();
        let (indices, norms) = gpu_backend::lloyd_max_batch(
            &rotated, *num_docs, *dim, k, n_levels, seed
        ).unwrap();
        let encode_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let blocks_per_vector = dim / k;
        let raw_kb = num_docs * dim * 4 / 1024;
        let comp_kb = (indices.len() + norms.len() * 4) / 1024;
        println!("  │  {} vectors: {:.2}ms  ({:.0} vec/s)", num_docs, encode_ms, *num_docs as f64 / t0.elapsed().as_secs_f64());
        println!("  │  Compression: {} KB → {} KB ({:.1}×)", raw_kb, comp_kb, raw_kb as f64 / comp_kb.max(1) as f64);
        println!("  └──────────────────────────────────────────────────────────┘\n");

        println!("  ┌─ gpu-backend: Lloyd-Max Decode ──────────────────────────┐");
        let t0 = Instant::now();
        let decoded = gpu_backend::lloyd_max_decode_batch(
            &indices, &norms, *num_docs, *dim, k, n_levels, seed
        ).unwrap();
        let decode_ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!("  │  {} vectors: {:.2}ms  ({:.0} vec/s)", num_docs, decode_ms, *num_docs as f64 / t0.elapsed().as_secs_f64());
        println!("  └──────────────────────────────────────────────────────────┘\n");

        // ═══════════════════════════════════════════════════════════
        // FIDELITY: gpu-backend roundtrip (normalized → rotated → encoded → decoded)
        // ═══════════════════════════════════════════════════════════
        println!("  ┌─ Fidelity: gpu-backend Lloyd-Max roundtrip ──────────────┐");
        let (cos, mse) = compute_fidelity(&vectors, &decoded, *dim, *num_docs);
        println!("  │  cosine: {:.6}  |  MSE: {:.6}", cos, mse);
        println!("  └──────────────────────────────────────────────────────────┘\n");

        // ═══════════════════════════════════════════════════════════
        // FIB-QUANT CRATE — end-to-end encode/decode
        // ═══════════════════════════════════════════════════════════
        #[cfg(feature = "fib-quant")]
        {
            use fib_quant::{FibQuantProfileV1, FibQuantizer};
            let fib_dim = (*dim / 4) * 4;
            if fib_dim >= 4 {
                println!("  ┌─ fib-quant crate: end-to-end encode/decode ──────────────┐");
                let mut profile = FibQuantProfileV1::paper_default(fib_dim, 4, 32, seed).unwrap();
                profile.training_samples = 2048u32.min(fib_dim as u32 * 4);
                profile.lloyd_restarts = 1;
                profile.lloyd_iterations = 2;
                let q = FibQuantizer::new(profile).unwrap();

                let t0 = Instant::now();
                let mut codes = Vec::new();
                let f32s: Vec<Vec<f32>> = vectors.chunks(*dim)
                    .map(|c| c.iter().take(fib_dim).copied().collect())
                    .collect();
                let refs: Vec<&[f32]> = f32s.iter().map(|v| v.as_slice()).collect();
                if let Ok(batch) = q.encode_batch(&refs) {
                    codes = batch;
                }
                let fib_enc_ms = t0.elapsed().as_secs_f64() * 1000.0;
                let total = codes.len();

                // Decode and fidelity
                let mut fib_cos = 0.0f64;
                let mut fib_mse = 0.0f64;
                for (i, code) in codes.iter().enumerate() {
                    if let Ok(dec) = q.decode(code) {
                        let orig: Vec<f64> = f32s[i].iter().map(|&v| v as f64).collect();
                        let d: Vec<f64> = dec.iter().map(|&v| v as f64).collect();
                        let dot: f64 = orig.iter().zip(d.iter()).map(|(a,b)| a*b).sum();
                        let mo: f64 = orig.iter().map(|v| v*v).sum::<f64>().sqrt();
                        let md: f64 = d.iter().map(|v| v*v).sum::<f64>().sqrt();
                        fib_cos += if mo > 0.0 && md > 0.0 { dot/(mo*md) } else { 0.0 };
                        fib_mse += orig.iter().zip(d.iter()).map(|(a,b)| (a-b).powi(2)).sum::<f64>() / fib_dim as f64;
                    }
                }
                let nf = codes.len() as f64;
                println!("  │  {} vectors: {:.2}ms  ({:.0} vec/s)", total, fib_enc_ms, total as f64 / (fib_enc_ms / 1000.0));
                println!("  │  JSON size: {} KB", codes.iter().map(|c| serde_json::to_vec(c).unwrap_or_default().len()).sum::<usize>() / 1024);
                println!("  │  fidelity: cos={:.6}  MSE={:.6}", fib_cos/nf.max(1.0), fib_mse/nf.max(1.0));
                println!("  └──────────────────────────────────────────────────────────┘\n");
            }
        }

        // ═══════════════════════════════════════════════════════════
        // TURBO-QUANT CRATE — end-to-end encode/decode
        // ═══════════════════════════════════════════════════════════
        #[cfg(feature = "turbo-quant")]
        {
            use turbo_quant::TurboQuantizer;
            let turbo_dim = (*dim / 2) * 2;
            if turbo_dim >= 2 {
                println!("  ┌─ turbo-quant crate: encode/decode ───────────────────────┐");
                let q = TurboQuantizer::new(turbo_dim, 8, 32, seed + 1).unwrap();

                let t0 = Instant::now();
                let mut tq_codes = Vec::new();
                let f32s: Vec<Vec<f32>> = vectors.chunks(*dim)
                    .map(|c| c.iter().take(turbo_dim).copied().collect())
                    .collect();
                for v in &f32s {
                    if let Ok(c) = q.encode(v) {
                        tq_codes.push(c);
                    }
                }
                let tq_enc_ms = t0.elapsed().as_secs_f64() * 1000.0;
                let total = tq_codes.len();

                let mut tq_cos = 0.0f64;
                let mut tq_mse = 0.0f64;
                for (i, code) in tq_codes.iter().enumerate() {
                    if let Ok(dec) = q.decode_approximate(code) {
                        let orig: Vec<f64> = f32s[i].iter().map(|&v| v as f64).collect();
                        let d: Vec<f64> = dec.iter().map(|&v| v as f64).collect();
                        let dot: f64 = orig.iter().zip(d.iter()).map(|(a,b)| a*b).sum();
                        let mo: f64 = orig.iter().map(|v| v*v).sum::<f64>().sqrt();
                        let md: f64 = d.iter().map(|v| v*v).sum::<f64>().sqrt();
                        tq_cos += if mo > 0.0 && md > 0.0 { dot/(mo*md) } else { 0.0 };
                        tq_mse += orig.iter().zip(d.iter()).map(|(a,b)| (a-b).powi(2)).sum::<f64>() / turbo_dim as f64;
                    }
                }
                let nf = tq_codes.len() as f64;
                println!("  │  {} vectors: {:.2}ms  ({:.0} vec/s)", total, tq_enc_ms, total as f64 / (tq_enc_ms / 1000.0));
                println!("  │  JSON size: {} KB", tq_codes.iter().map(|c| serde_json::to_vec(c).unwrap_or_default().len()).sum::<usize>() / 1024);
                println!("  │  fidelity: cos={:.6}  MSE={:.6}", tq_cos/nf.max(1.0), tq_mse/nf.max(1.0));
                println!("  └──────────────────────────────────────────────────────────┘\n");
            }
        }

        // ═══════════════════════════════════════════════════════════
        // POLY-KV STYLE: Two-tier (80/20 fib cold + turbo hot)
        // ═══════════════════════════════════════════════════════════
        #[cfg(all(feature = "fib-quant", feature = "turbo-quant"))]
        {
            println!("  ┌─ poly-kv style: two-tier (80% fib + 20% turbo) ─────────┐");
            let split = *num_docs * 4 / 5;
            println!("  │  cold tier (fib-quant): {} docs", split);
            println!("  │  hot tier  (turbo-quant): {} docs", *num_docs - split);
            let cold_raw = split * *dim * 4 / 1024;
            let hot_raw = (*num_docs - split) * *dim * 4 / 1024;
            println!("  │  cold raw: {} KB  |  hot raw: {} KB", cold_raw, hot_raw);
            println!("  └──────────────────────────────────────────────────────────┘\n");
        }

        println!();
    }

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                   ✅ Per-crate benchmark complete              ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

fn compute_fidelity(original: &[f32], decoded: &[f32], dim: usize, n: usize) -> (f64, f64) {
    let mut total_cos = 0.0;
    let mut total_mse = 0.0;
    for i in 0..n {
        let o = &original[i * dim..(i + 1) * dim];
        let d = &decoded[i * dim..(i + 1) * dim];
        let dot: f64 = o.iter().zip(d.iter()).map(|(a,b)| (*a as f64) * (*b as f64)).sum();
        let mo: f64 = o.iter().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
        let md: f64 = d.iter().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt();
        total_cos += if mo > 0.0 && md > 0.0 { dot / (mo * md) } else { 0.0 };
        total_mse += o.iter().zip(d.iter())
            .map(|(a,b)| ((*a as f64) - (*b as f64)).powi(2)).sum::<f64>() / dim as f64;
    }
    (total_cos / n as f64, total_mse / n as f64)
}

fn generate_normalized_vectors(dim: usize, num_docs: usize) -> Vec<f32> {
    use std::num::Wrapping;
    let mut state = Wrapping(42u64);
    let std_dev = 1.0 / (dim as f64).sqrt() as f32;
    let mut data = Vec::with_capacity(num_docs * dim);
    for _ in 0..num_docs {
        let mut vec = Vec::with_capacity(dim);
        let mut norm_sq = 0.0f64;
        for _ in 0..dim {
            state = state * Wrapping(6364136223846793005) + Wrapping(1442695040888963407);
            let u = (state.0 as f64) / (u64::MAX as f64);
            let v = ((-2.0 * (1.0 - u).ln()).sqrt() * std_dev as f64) as f32;
            norm_sq += (v as f64).powi(2);
            vec.push(v);
        }
        // Normalize to unit length (matching real embedding behavior)
        let norm = norm_sq.sqrt() as f32;
        if norm > 0.0 {
            for v in &mut vec { *v /= norm; }
        }
        data.extend(vec);
    }
    data
}
