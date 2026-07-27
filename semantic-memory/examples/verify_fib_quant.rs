use rusqlite::Connection;
use semantic_memory::config::SearchConfig;
use semantic_memory::search::{brute_force_vector_outcome, fib_quant_vector_outcome};
use semantic_memory::types::SearchSourceType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("/home/sikmindz/.hermes/semantic-memory.db/memory.db")?;
    let mut stmt = conn.prepare(
        "SELECT embedding FROM facts WHERE embedding IS NOT NULL ORDER BY RANDOM() LIMIT 5",
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
    let config = SearchConfig::default();
    let source_types = [SearchSourceType::Facts];
    let mut total = 0.0;
    for (i, query) in queries.iter().enumerate() {
        let brute =
            brute_force_vector_outcome(&conn, query, 10, -1.0, None, Some(&source_types), None)?;
        let fib = fib_quant_vector_outcome(
            &conn,
            query,
            10,
            -1.0,
            &config,
            None,
            Some(&source_types),
            None,
        )?;
        let ids: std::collections::HashSet<_> = brute.hits.iter().map(|h| &h.id).collect();
        let overlap = fib.hits.iter().filter(|h| ids.contains(&h.id)).count();
        let recall = overlap as f64 / brute.hits.len().max(1) as f64;
        total += recall;
        println!("query {}: recall@10={recall:.3}", i + 1);
    }
    if !queries.is_empty() {
        println!("mean recall@10: {:.3}", total / queries.len() as f64);
    }
    Ok(())
}
