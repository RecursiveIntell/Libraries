use semantic_memory::MemoryStore;
use std::path::Path;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let r = MemoryStore::rebuild_fib_quant_artifacts_at(
        Path::new("/home/sikmindz/.hermes/semantic-memory.db/memory.db"),
        768,
        12,
    )?;
    println!("source_rows={} artifacts_generated={} skipped_rows={} elapsed_ms={} generation_id={:?} manifest={:?}",r.source_row_count,r.artifact_count,r.skipped_row_count,r.elapsed_ms,r.generation_id,r.artifact_manifest_digest);
    Ok(())
}
