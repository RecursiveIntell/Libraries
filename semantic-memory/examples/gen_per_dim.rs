use semantic_memory::MemoryStore;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "/home/sikmindz/.hermes/semantic-memory.db/memory.db";
    let receipt = MemoryStore::rebuild_per_dim_artifacts_at(Path::new(db_path), 768, 8)?;
    println!("source_rows={}", receipt.source_row_count);
    println!("artifact_count={}", receipt.artifact_count);
    println!("elapsed_ms={}", receipt.elapsed_ms);
    println!("receipt: {:?}", receipt);
    Ok(())
}
