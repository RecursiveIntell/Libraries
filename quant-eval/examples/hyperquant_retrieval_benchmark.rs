use hyperquant::LatticeKind;
use quant_eval::{run_hyperquant_retrieval_benchmark, HyperQuantRetrievalBenchmarkConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = run_hyperquant_retrieval_benchmark(&HyperQuantRetrievalBenchmarkConfig {
        dim: 32,
        docs: 128,
        queries: 12,
        clusters: 8,
        top_k: 5,
        seed: 11,
        scale: 16.0,
        lattice: LatticeKind::D4,
    })?;

    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}
