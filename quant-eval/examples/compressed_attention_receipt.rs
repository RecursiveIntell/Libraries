use std::env;
use std::fs;
use std::path::PathBuf;

use quant_eval::{run_compressed_attention_eval, CompressedAttentionConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = env::args().nth(1).map(PathBuf::from);
    let keys = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.9, 0.1, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0],
    ];
    let values = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.8, 0.2, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0],
    ];
    let queries = vec![vec![1.0, 0.05, 0.0, 0.0], vec![0.0, 0.95, 0.05, 0.0]];
    let receipt = run_compressed_attention_eval(
        &keys,
        &values,
        &queries,
        &CompressedAttentionConfig {
            bits: env_u32("CA_BITS", 8),
            top_k: env_usize("CA_TOP_K", 2),
            min_mean_output_cosine: env_f32("CA_MIN_MEAN_OUTPUT_COSINE", 0.80),
            max_mean_output_mse: env_f32("CA_MAX_MEAN_OUTPUT_MSE", 0.10),
            min_top_k_overlap: env_f32("CA_MIN_TOP_K_OVERLAP", 0.50),
        },
    )?;
    let json = serde_json::to_string_pretty(&receipt)?;
    if let Some(output) = output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output, json)?;
    } else {
        println!("{json}");
    }
    Ok(())
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_u32(name: &str, default: u32) -> u32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_f32(name: &str, default: f32) -> f32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
