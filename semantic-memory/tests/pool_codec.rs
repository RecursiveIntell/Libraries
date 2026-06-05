//! Integration tests for the poly-kv-pool VectorCodec adapter.
//!
//! Tests that PoolCodec round-trips embeddings through
//! SharedKVPool's fib-quant compression with acceptable
//! cosine similarity and compression ratio.

#[cfg(feature = "poly-kv-pool")]
use semantic_memory::{MemoryError, PoolCodec, VectorCodec};

#[cfg(feature = "poly-kv-pool")]
fn deterministic_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    let mut vector = Vec::with_capacity(dim);
    for _ in 0..dim {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let value = ((state as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32;
        vector.push(value);
    }
    vector
}

#[cfg(feature = "poly-kv-pool")]
fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

// fib-quant k=4 at low dimensions (64) gives ~0.86 cosine sim.
// At higher dimensions (384+), cosine sim improves toward 0.99+.
// The threshold below reflects measured reality at the test dimension.
const COSINE_SIM_THRESHOLD: f32 = 0.80;

#[cfg(feature = "poly-kv-pool")]
#[test]
fn pool_codec_build_returns_valid_profile() -> Result<(), MemoryError> {
    let dim = 64;
    let corpus: Vec<(String, Vec<f32>)> = (0..8)
        .map(|i| (format!("vec_{i}"), deterministic_vector(dim, i as u64)))
        .collect();

    let codec = PoolCodec::new(dim, &corpus, 42)?;

    assert_eq!(codec.profile().codec, "shared_kv_pool");
    assert_eq!(codec.profile().dim, dim as u32);
    assert_eq!(codec.profile().bits, 4);
    assert_eq!(codec.profile().seed, Some(42));
    assert!(!codec.build_receipt().pool_digest.hex().is_empty());

    Ok(())
}

#[cfg(feature = "poly-kv-pool")]
#[test]
fn pool_codec_decode_at_reproduces_one_embedding() -> Result<(), MemoryError> {
    let dim = 64;
    let originals: Vec<Vec<f32>> = (0..4)
        .map(|i| deterministic_vector(dim, i as u64))
        .collect();

    let corpus: Vec<(String, Vec<f32>)> = originals
        .iter()
        .enumerate()
        .map(|(i, v)| (format!("vec_{i}"), v.clone()))
        .collect();

    let codec = PoolCodec::new(dim, &corpus, 42)?;

    // `decode_at` intentionally decompresses the pool for direct indexed access.
    // Test one index here; bulk quality is covered by `decode_all`, which is the
    // intended evaluation path.
    let index = 2;
    let decoded = codec.decode_at(index)?;
    assert_eq!(decoded.len(), dim);
    let sim = cosine_sim(&originals[index], &decoded);
    assert!(
        sim > COSINE_SIM_THRESHOLD,
        "cosine similarity {sim:.4} < {COSINE_SIM_THRESHOLD} threshold"
    );
    println!("pool_codec_decode_at: dim={dim} cosine_sim={sim:.4}");

    Ok(())
}

#[cfg(feature = "poly-kv-pool")]
#[test]
fn pool_codec_decode_all_returns_all_vectors() -> Result<(), MemoryError> {
    let dim = 64;
    let n = 16;
    let originals: Vec<Vec<f32>> = (0..n)
        .map(|i| deterministic_vector(dim, i as u64))
        .collect();

    let corpus: Vec<(String, Vec<f32>)> = originals
        .iter()
        .enumerate()
        .map(|(i, v)| (format!("vec_{i}"), v.clone()))
        .collect();

    let codec = PoolCodec::new(dim, &corpus, 42)?;
    let decoded = codec.decode_all()?;

    assert_eq!(decoded.len(), n);
    let mut min_sim = 1.0f32;
    for (i, original) in originals.iter().enumerate() {
        let sim = cosine_sim(original, &decoded[i]);
        min_sim = min_sim.min(sim);
    }
    assert!(
        min_sim > COSINE_SIM_THRESHOLD,
        "min cosine similarity {min_sim:.4} < {COSINE_SIM_THRESHOLD}"
    );
    println!("pool_codec_decode_all: dim={dim} min_cosine_sim={min_sim:.4}");

    Ok(())
}

#[cfg(feature = "poly-kv-pool")]
#[test]
fn pool_codec_vector_codec_round_trips() -> Result<(), MemoryError> {
    let dim = 64;
    let corpus: Vec<(String, Vec<f32>)> = (0..4)
        .map(|i| (format!("vec_{i}"), deterministic_vector(dim, i as u64)))
        .collect();

    let codec = PoolCodec::new(dim, &corpus, 42)?;

    let original = deterministic_vector(dim, 99);
    let artifact = codec.encode(&original)?;
    let decoded = codec.decode(&artifact)?;

    assert_eq!(decoded.len(), dim);
    let sim = cosine_sim(&original, &decoded);
    assert!(
        sim > COSINE_SIM_THRESHOLD,
        "per-vector round-trip cosine sim {sim:.4} < {COSINE_SIM_THRESHOLD}"
    );
    println!("pool_codec_roundtrip: dim={dim} cosine_sim={sim:.4}");

    Ok(())
}

#[cfg(feature = "poly-kv-pool")]
#[test]
fn pool_codec_medium_dim_cosine_similarity() -> Result<(), MemoryError> {
    // Keep this shape small enough for a public quick gate. Larger dimensions
    // are covered by proveKV's checked-in PPL receipts rather than by this
    // semantic-memory adapter smoke test.
    let dim = 128;
    let corpus: Vec<(String, Vec<f32>)> = (0..4)
        .map(|i| (format!("vec_{i}"), deterministic_vector(dim, i as u64)))
        .collect();

    let codec = PoolCodec::new(dim, &corpus, 42)?;

    let mut min_sim = 1.0f32;
    for (i, (_, original)) in corpus.iter().enumerate() {
        let decoded = codec.decode_at(i)?;
        let sim = cosine_sim(original, &decoded);
        min_sim = min_sim.min(sim);
    }

    println!("pool_codec_medium_dim: dim={dim} min_cosine_sim={min_sim:.4}");
    // fib-quant k=4 gives ~0.86 cosine sim in this adapter smoke shape.
    // This is a per-vector decoded-cosine sanity check, not the proveKV
    // PPL-neutrality claim.
    assert!(
        min_sim > 0.80,
        "dim=128 min cosine similarity {min_sim:.4} < 0.80"
    );

    Ok(())
}

#[cfg(feature = "poly-kv-pool")]
#[test]
fn pool_codec_compresses_below_raw_size() -> Result<(), MemoryError> {
    let dim = 64;
    let corpus: Vec<(String, Vec<f32>)> = (0..16)
        .map(|i| (format!("vec_{i}"), deterministic_vector(dim, i as u64)))
        .collect();

    let codec = PoolCodec::new(dim, &corpus, 42)?;

    let raw_bytes_per_vec = dim * std::mem::size_of::<f32>();

    let mut total_compressed = 0usize;
    for (_, vec) in &corpus {
        let artifact = codec.encode(vec)?;
        total_compressed += artifact.encoded.len();
    }

    let total_raw = corpus.len() * raw_bytes_per_vec;
    let ratio = total_raw as f64 / total_compressed as f64;
    assert!(ratio > 2.0, "compression ratio {ratio:.1}x < 2.0x minimum");

    println!(
        "pool_codec_compresses: dim={} n_vecs={} raw={} compressed={} ratio={ratio:.1}x",
        dim,
        corpus.len(),
        total_raw,
        total_compressed
    );

    Ok(())
}

#[cfg(feature = "poly-kv-pool")]
#[test]
fn pool_codec_rejects_wrong_dimension() -> Result<(), MemoryError> {
    let dim = 64;
    let corpus: Vec<(String, Vec<f32>)> = (0..4)
        .map(|i| (format!("vec_{i}"), deterministic_vector(dim, i as u64)))
        .collect();

    let codec = PoolCodec::new(dim, &corpus, 42)?;

    let wrong_dim_vec = deterministic_vector(32, 0);
    let result = codec.encode(&wrong_dim_vec);
    assert!(result.is_err(), "should reject wrong-dimension vector");

    Ok(())
}

#[cfg(feature = "poly-kv-pool")]
#[test]
fn pool_governed_builds_from_corpus() {
    use semantic_memory::quantize_governed::pool_governed::encode_pool;

    let dim = 64;
    let corpus: Vec<(String, Vec<f32>)> = (0..8)
        .map(|i| (format!("vec_{i}"), deterministic_vector(dim, i as u64)))
        .collect();

    let codec = encode_pool(&corpus, dim, 42).expect("pool build should succeed");
    assert_eq!(codec.profile().dim, dim as u32);
}

#[cfg(not(feature = "poly-kv-pool"))]
#[test]
fn pool_governed_stub_returns_error_when_disabled() {
    use semantic_memory::quantize_governed::pool_governed::encode_pool;

    let corpus: Vec<(String, Vec<f32>)> = vec![("x".into(), vec![0.0; 64])];
    let result = encode_pool(&corpus, 64, 42);
    assert!(result.is_err());
}
