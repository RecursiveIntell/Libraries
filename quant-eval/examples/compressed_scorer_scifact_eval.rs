use std::env;
use std::fs;
use std::path::PathBuf;

use quant_eval::{
    run_compressed_scorer_real_corpus_eval, CompressedScorerRealCorpusConfig, HyperQuantRealCorpus,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .ok_or("usage: compressed_scorer_scifact_eval <corpus.json> [receipt.json]")?;
    let output = args.next().map(PathBuf::from);
    let data = fs::read_to_string(&input)?;
    let corpus: HyperQuantRealCorpus = serde_json::from_str(&data)?;
    let config = CompressedScorerRealCorpusConfig {
        top_k: env_usize("CS_TOP_K", 10),
        candidate_k: env_usize("CS_CANDIDATE_K", 40),
        bits: env_u32("CS_BITS", 8),
        min_top_k_overlap: env_f32("CS_MIN_TOP_K_OVERLAP", 0.30),
        min_exact_rerank_recovery_at_1: env_f32("CS_MIN_EXACT_RERANK_RECOVERY_AT_1", 0.80),
    };
    let receipt = run_compressed_scorer_real_corpus_eval(&corpus, &config)?;
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
