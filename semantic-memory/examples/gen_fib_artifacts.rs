use semantic_memory::MemoryStore;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "/home/sikmindz/.hermes/semantic-memory.db/memory.db";
    let receipt = MemoryStore::rebuild_fib_quant_artifacts_at(Path::new(db_path), 768, 12)?;
    println!("source_rows={}", receipt.source_row_count);
    println!("artifact_count={}", receipt.artifact_count);
    println!("receipt: {:?}", receipt);
    Ok(())
}
