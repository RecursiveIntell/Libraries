use poly_kv::{
    run_model_replay, AttentionType, KvTensorShape, ModelReplayConfig, ModelReplayQuery,
    SharedKVPool,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn candidate_ks() -> Vec<usize> {
    std::env::var("PKV_CANDIDATE_KS")
        .ok()
        .map(|raw| {
            raw.split(',')
                .filter_map(|part| part.trim().parse::<usize>().ok())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| vec![32, 64, 128, 256])
}

fn make_shape(num_layers: u32, num_heads: u32, head_dim: usize) -> KvTensorShape {
    KvTensorShape {
        attention_type: AttentionType::MHA,
        num_layers,
        num_heads,
        num_kv_heads: num_heads,
        head_dim,
        hidden_size: num_heads as usize * head_dim,
    }
}

fn make_corpus(n: usize, shape: &KvTensorShape, seed: u64) -> Vec<(String, Vec<f32>)> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let len = shape.num_layers as usize * shape.num_kv_heads as usize * shape.head_dim * 2;
    (0..n)
        .map(|i| {
            let values = (0..len)
                .map(|j| {
                    let harmonic = ((i as f32 * 0.017) + (j as f32 * 0.013)).sin() * 0.5;
                    harmonic + rng.gen_range(-0.20f32..0.20)
                })
                .collect();
            (format!("token_{i}"), values)
        })
        .collect()
}

fn make_queries(n: usize, head_dim: usize, vocab_size: usize, seed: u64) -> Vec<ModelReplayQuery> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..n)
        .map(|i| ModelReplayQuery {
            query: (0..head_dim)
                .map(|j| ((i as f32 * 0.031) + (j as f32 * 0.07)).cos() + rng.gen_range(-0.1..0.1))
                .collect(),
            label_token: i % vocab_size,
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shared_tokens = env_usize("PKV_SHARED_TOKENS", 512);
    let shell_tokens = env_usize("PKV_SHELL_TOKENS", 64);
    let num_queries = env_usize("PKV_QUERIES", 8);
    let head_dim = env_usize("PKV_HEAD_DIM", 64);
    let num_heads = env_usize("PKV_HEADS", 4) as u32;
    let num_layers = env_usize("PKV_LAYERS", 2) as u32;
    let vocab_size = env_usize("PKV_VOCAB", 128);
    let seed = 4242u64;

    let shape = make_shape(num_layers, num_heads, head_dim);
    let shared = make_corpus(shared_tokens, &shape, seed);
    let shell_corpus = make_corpus(shell_tokens, &shape, seed + 1);
    let (pool, _pool_receipt) = SharedKVPool::build(&shared, &shape, seed + 2)?;
    let (shell, _shell_receipt) =
        pool.materialize_shell("model_replay_agent", &shell_corpus, seed + 3)?;
    let queries = make_queries(num_queries, head_dim, vocab_size, seed + 4);

    let receipt = run_model_replay(
        &pool,
        &shell,
        &queries,
        ModelReplayConfig {
            layer: 0,
            head: 0,
            candidate_ks: candidate_ks(),
            vocab_size,
            projection_seed: seed + 5,
            min_output_cosine: 0.10,
            max_output_mse: 1.25,
            max_kl_divergence: 1.25,
            max_ppl_delta: 250.0,
            min_top1_agreement: 0.0,
        },
    )?;

    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}
