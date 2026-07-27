use rusqlite::Connection;
use semantic_memory::config::SearchConfig;
use semantic_memory::search::{brute_force_vector_outcome, per_dim_vector_outcome};
use semantic_memory::types::SearchSourceType;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("/home/sikmindz/.hermes/semantic-memory.db/memory.db")?;
    let mut stmt = conn.prepare(
        "SELECT embedding FROM facts WHERE embedding IS NOT NULL ORDER BY RANDOM() LIMIT 20",
    )?;
    let queries: Vec<Vec<f32>> = stmt
        .query_map([], |row| {
            let blob: Vec<u8> = row.get(0)?;
            Ok(blob
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect())
        })?
        .collect::<Result<_, _>>()?;

    let mut config = SearchConfig::default();
    config.per_dim_bits = 8;
    let st = [SearchSourceType::Facts];
    let warmup = 3;
    let measured = queries.len() - warmup;

    // Warmup both paths
    for q in queries.iter().take(warmup) {
        let _ = brute_force_vector_outcome(&conn, q, 10, -1.0, None, Some(&st), None);
        let _ = per_dim_vector_outcome(&conn, q, 10, -1.0, &config, None, Some(&st), None);
    }

    // Time brute force
    let t0 = Instant::now();
    for q in queries.iter().skip(warmup) {
        let _ = brute_force_vector_outcome(&conn, q, 10, -1.0, None, Some(&st), None);
    }
    let brute_ms = t0.elapsed().as_micros() as f64 / 1000.0 / measured as f64;

    // Time PerDim
    let t0 = Instant::now();
    for q in queries.iter().skip(warmup) {
        let _ = per_dim_vector_outcome(&conn, q, 10, -1.0, &config, None, Some(&st), None);
    }
    let per_dim_ms = t0.elapsed().as_micros() as f64 / 1000.0 / measured as f64;

    println!("brute force: {brute_ms:.2} ms/query");
    println!("per_dim:     {per_dim_ms:.2} ms/query");
    println!("speedup:     {:.2}x", brute_ms / per_dim_ms);
    Ok(())
}
