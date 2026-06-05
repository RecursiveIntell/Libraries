#[cfg(feature = "poly-kv-pool")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use semantic_memory::{
        build_embedding_snapshot, build_provekv_pool_generation, EmbeddingSnapshotRow,
    };
    use serde_json::json;
    use std::time::Instant;

    let dim = 384usize;
    let row_count = 128usize;
    let rows = (0..row_count)
        .map(|i| {
            let embedding = (0..dim)
                .map(|j| (((i * 31 + j * 17) % 1009) as f32 / 1009.0) - 0.5)
                .collect::<Vec<_>>();
            EmbeddingSnapshotRow {
                item_id: format!("bench-{i:04}"),
                source_type: "fact".to_string(),
                embedding,
            }
        })
        .collect::<Vec<_>>();

    let snapshot = build_embedding_snapshot(rows, dim)?;
    let started = Instant::now();
    let (generation, payload, item_map) = build_provekv_pool_generation(snapshot, 42)?;
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let raw_f32_bytes = (row_count * dim * std::mem::size_of::<f32>()) as u64;
    let payload_bytes = payload.len() as u64;
    let compression_ratio_vs_embedding_f32 = raw_f32_bytes as f64 / payload_bytes.max(1) as f64;

    let receipt = json!({
        "schema_version": "semantic_memory_provekv_pool_benchmark_receipt_v1",
        "backend": "provekv_pool_candidate_then_exact_f32",
        "generation_id": generation.generation_id,
        "embedding_snapshot_digest": generation.embedding_snapshot_digest,
        "pool_manifest_digest": generation.pool_manifest_digest,
        "codec_family": generation.codec_family,
        "codec_profile": generation.codec_profile,
        "dim": dim,
        "row_count": row_count,
        "item_map_count": item_map.len(),
        "raw_f32_bytes": raw_f32_bytes,
        "payload_bytes": payload_bytes,
        "compression_ratio_vs_embedding_f32": compression_ratio_vs_embedding_f32,
        "elapsed_ms": elapsed_ms,
        "exact_f32_rerank_required": true,
        "authoritative_store": "semantic-memory sqlite f32 embeddings",
        "candidate_only": true
    });
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}

#[cfg(not(feature = "poly-kv-pool"))]
fn main() {
    eprintln!("enable --features poly-kv-pool to run this benchmark receipt example");
    std::process::exit(2);
}
