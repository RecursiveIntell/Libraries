use rusqlite::Connection;
use semantic_memory::config::SearchConfig;
use semantic_memory::search::{brute_force_vector_outcome, per_dim_vector_outcome};
use semantic_memory::types::SearchSourceType;
use std::collections::HashSet;
use std::time::{Duration, Instant};

const QUERY_COUNT: usize = 20;
const TOP_K: usize = 10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("/home/sikmindz/.hermes/semantic-memory.db/memory.db")?;
    let mut stmt = conn.prepare(
        "SELECT embedding FROM facts WHERE embedding IS NOT NULL ORDER BY RANDOM() LIMIT ?1",
    )?;
    let queries: Vec<Vec<f32>> = stmt
        .query_map([QUERY_COUNT as i64], |row| {
            let blob: Vec<u8> = row.get(0)?;
            Ok(blob
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect())
        })?
        .collect::<Result<_, _>>()?;

    if queries.is_empty() {
        println!("no embedded facts found; nothing to benchmark");
        return Ok(());
    }

    let source_types = [SearchSourceType::Facts];
    let mut exact_config = SearchConfig::default();
    exact_config.per_dim_bits = 8;
    exact_config.per_dim_require_exact_rerank = true;
    let mut fast_config = exact_config.clone();
    fast_config.per_dim_require_exact_rerank = false;

    let mut exact_duration = Duration::ZERO;
    let mut fast_duration = Duration::ZERO;
    let mut total_recall = 0.0;

    for (i, query) in queries.iter().enumerate() {
        let exact_start = Instant::now();
        let exact = per_dim_vector_outcome(
            &conn,
            query,
            TOP_K,
            -1.0,
            &exact_config,
            None,
            Some(&source_types),
            None,
        )?;
        exact_duration += exact_start.elapsed();

        let fast_start = Instant::now();
        let fast = per_dim_vector_outcome(
            &conn,
            query,
            TOP_K,
            -1.0,
            &fast_config,
            None,
            Some(&source_types),
            None,
        )?;
        fast_duration += fast_start.elapsed();

        let exact_ids: HashSet<_> = exact.hits.iter().map(|hit| &hit.id).collect();
        let overlap = fast
            .hits
            .iter()
            .filter(|hit| exact_ids.contains(&hit.id))
            .count();
        let recall = overlap as f64 / exact.hits.len().max(1) as f64;
        total_recall += recall;
        println!("query {}: recall@{TOP_K}={recall:.3}", i + 1);
    }

    let brute_start = Instant::now();
    let mut brute_recall = 0.0;
    for query in &queries {
        let fast = per_dim_vector_outcome(
            &conn,
            query,
            TOP_K,
            -1.0,
            &fast_config,
            None,
            Some(&source_types),
            None,
        )?;
        let brute =
            brute_force_vector_outcome(&conn, query, TOP_K, -1.0, None, Some(&source_types), None)?;
        let brute_ids: HashSet<_> = brute.hits.iter().map(|hit| &hit.id).collect();
        brute_recall += fast
            .hits
            .iter()
            .filter(|hit| brute_ids.contains(&hit.id))
            .count() as f64
            / brute.hits.len().max(1) as f64;
    }
    let brute_duration = brute_start.elapsed();
    let count = queries.len() as f64;

    println!("queries: {}", queries.len());
    println!("fast config: per_dim_require_exact_rerank=false");
    println!("mean recall@{TOP_K} vs exact: {:.3}", total_recall / count);
    println!(
        "mean recall@{TOP_K} vs brute force: {:.3}",
        brute_recall / count
    );
    println!(
        "exact rerank total: {:?} ({:.3} ms/query)",
        exact_duration,
        exact_duration.as_secs_f64() * 1000.0 / count
    );
    println!(
        "fast total: {:?} ({:.3} ms/query)",
        fast_duration,
        fast_duration.as_secs_f64() * 1000.0 / count
    );
    println!(
        "brute force total: {:?} ({:.3} ms/query)",
        brute_duration,
        brute_duration.as_secs_f64() * 1000.0 / count
    );
    println!(
        "fast vs exact speedup: {:.2}x",
        exact_duration.as_secs_f64() / fast_duration.as_secs_f64()
    );

    Ok(())
}
