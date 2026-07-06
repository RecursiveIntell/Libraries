use std::fs;

use poly_kv::{run_captured_model_replay, CapturedReplayConfig, CapturedReplayFixture};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fixture_path = std::env::args().nth(1).ok_or_else(|| {
        "usage: cargo run --example poly_kv_captured_model_replay -- <fixture.json>".to_string()
    })?;
    let fixture: CapturedReplayFixture = serde_json::from_str(&fs::read_to_string(fixture_path)?)?;
    let receipt = run_captured_model_replay(
        &fixture,
        CapturedReplayConfig {
            candidate_ks: vec![8, 16, 32, 48, 64, 72],
            min_output_cosine: 0.10,
            max_output_mse: 4.0,
            max_kl_divergence: 4.0,
            max_ppl_delta: 10.0,
            min_top1_agreement: 0.25,
        },
    )?;
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}
