//! proveKV/poly-kv pool generation adapter.
//!
//! `semantic-memory` keeps authoritative f32 embeddings in SQLite. This module builds a
//! generation-level compressed candidate artifact from those rows when the `poly-kv-pool` feature is
//! enabled. The artifact is a derived acceleration pool only: search still exact-reranks against the
//! authoritative f32 rows before returning results.

use crate::error::MemoryError;
use crate::types::{ProveKvPoolGenerationV1, ProveKvPoolItemMapEntryV1};
use crate::vector_snapshot::{embedding_row_digest, EmbeddingSnapshotV1};
use chrono::Utc;

#[cfg(feature = "poly-kv-pool")]
use poly_kv_core::{AttentionType, KvTensorShape, PoolBuildReceipt, SharedKVPool};

#[cfg(feature = "poly-kv-pool")]
fn padded_vector_dim_for_poly_kv(vector_dim: usize) -> usize {
    let min_head_dim = vector_dim.div_ceil(2).max(8);
    let head_dim = min_head_dim.div_ceil(4) * 4;
    head_dim * 2
}

#[cfg(feature = "poly-kv-pool")]
fn semantic_embedding_shape(vector_dim: usize) -> KvTensorShape {
    let padded_vector_dim = padded_vector_dim_for_poly_kv(vector_dim);
    let head_dim = padded_vector_dim / 2;
    KvTensorShape {
        attention_type: AttentionType::MHA,
        num_layers: 1,
        num_heads: 1,
        num_kv_heads: 1,
        head_dim,
        hidden_size: head_dim,
    }
}

#[cfg(feature = "poly-kv-pool")]
fn semantic_embedding_corpus(snapshot: &EmbeddingSnapshotV1) -> Vec<(String, Vec<f32>)> {
    let padded_vector_dim = padded_vector_dim_for_poly_kv(snapshot.vector_dim);
    snapshot
        .rows
        .iter()
        .map(|row| {
            let mut embedding = row.embedding.clone();
            embedding.resize(padded_vector_dim, 0.0);
            let head_dim = padded_vector_dim / 2;
            if embedding[..head_dim].iter().all(|value| *value == 0.0) {
                embedding[0] = f32::EPSILON;
            }
            if embedding[head_dim..].iter().all(|value| *value == 0.0) {
                embedding[head_dim] = f32::EPSILON;
            }
            (format!("{}:{}", row.source_type, row.item_id), embedding)
        })
        .collect()
}

#[cfg(feature = "poly-kv-pool")]
fn push_len_prefixed_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(bytes);
}

#[cfg(feature = "poly-kv-pool")]
fn compact_pool_payload(
    snapshot: &EmbeddingSnapshotV1,
    shape: &KvTensorShape,
    pool: &SharedKVPool,
    receipt: &PoolBuildReceipt,
) -> Result<Vec<u8>, MemoryError> {
    let metadata = serde_json::json!({
        "schema_version": "semantic_memory_provekv_pool_payload_v1",
        "embedding_snapshot_digest": snapshot.embedding_snapshot_digest,
        "source_digest": snapshot.source_digest,
        "vector_dim": snapshot.vector_dim,
        "padded_vector_dim": shape.kv_elements_per_token_per_layer(),
        "shape": shape,
        "manifest": pool.manifest,
        "receipt": receipt,
    });
    let metadata = serde_json::to_vec(&metadata).map_err(|err| {
        MemoryError::Other(format!(
            "failed to serialize proveKV compact payload metadata: {err}"
        ))
    })?;
    let mut payload = Vec::new();
    payload.extend_from_slice(b"SMP1");
    payload.push(1);
    payload.extend_from_slice(&(pool.layers.len() as u32).to_le_bytes());
    push_len_prefixed_bytes(&mut payload, &metadata);
    for layer in &pool.layers {
        payload.extend_from_slice(&layer.layer_index.to_le_bytes());
        push_len_prefixed_bytes(&mut payload, layer.block_digest.hex().as_bytes());
        payload.extend_from_slice(&(layer.key_blocks.len() as u32).to_le_bytes());
        for block in &layer.key_blocks {
            push_len_prefixed_bytes(&mut payload, block.codec.as_bytes());
            push_len_prefixed_bytes(&mut payload, &block.encoded_payload);
        }
        payload.extend_from_slice(&(layer.value_blocks.len() as u32).to_le_bytes());
        for block in &layer.value_blocks {
            push_len_prefixed_bytes(&mut payload, block.codec.as_bytes());
            push_len_prefixed_bytes(&mut payload, &block.encoded_payload);
        }
    }
    Ok(payload)
}

/// Build a proveKV/poly-kv generation envelope from an authoritative embedding snapshot.
#[cfg(feature = "poly-kv-pool")]
pub fn build_provekv_pool_generation(
    snapshot: EmbeddingSnapshotV1,
    seed: u64,
) -> Result<
    (
        ProveKvPoolGenerationV1,
        Vec<u8>,
        Vec<ProveKvPoolItemMapEntryV1>,
    ),
    MemoryError,
> {
    if snapshot.rows.is_empty() {
        return Err(MemoryError::Other(
            "cannot build proveKV pool generation from an empty embedding snapshot".to_string(),
        ));
    }
    if snapshot.embedding_snapshot_digest.is_empty() || snapshot.source_digest.is_empty() {
        return Err(MemoryError::Other(
            "proveKV pool snapshot digests must be non-empty".to_string(),
        ));
    }

    let generation_id = uuid::Uuid::new_v4().to_string();
    let mut item_map = Vec::with_capacity(snapshot.rows.len());
    for (pool_index, row) in snapshot.rows.iter().enumerate() {
        item_map.push(ProveKvPoolItemMapEntryV1 {
            generation_id: generation_id.clone(),
            item_id: row.item_id.clone(),
            source_type: row.source_type.clone(),
            pool_index,
            embedding_digest: embedding_row_digest(row, snapshot.vector_dim)?,
        });
    }

    let shape = semantic_embedding_shape(snapshot.vector_dim);
    let corpus = semantic_embedding_corpus(&snapshot);
    let (pool, receipt) = SharedKVPool::build(&corpus, &shape, seed).map_err(|err| {
        MemoryError::Other(format!("failed to build proveKV/poly-kv pool: {err}"))
    })?;
    let payload = compact_pool_payload(&snapshot, &shape, &pool, &receipt)?;

    let mut manifest_hasher = blake3::Hasher::new();
    manifest_hasher.update(b"semantic-memory.provekv_pool_manifest.v1");
    manifest_hasher.update(&[0]);
    manifest_hasher.update(snapshot.embedding_snapshot_digest.as_bytes());
    manifest_hasher.update(&[0]);
    manifest_hasher.update(snapshot.source_digest.as_bytes());
    manifest_hasher.update(&[0]);
    manifest_hasher.update(&(snapshot.vector_dim as u64).to_le_bytes());
    manifest_hasher.update(&(snapshot.rows.len() as u64).to_le_bytes());
    manifest_hasher.update(&seed.to_le_bytes());
    manifest_hasher.update(&payload);
    for entry in &item_map {
        manifest_hasher.update(entry.item_id.as_bytes());
        manifest_hasher.update(&[0]);
        manifest_hasher.update(entry.source_type.as_bytes());
        manifest_hasher.update(&[0]);
        manifest_hasher.update(&(entry.pool_index as u64).to_le_bytes());
        manifest_hasher.update(entry.embedding_digest.as_bytes());
        manifest_hasher.update(&[0]);
    }
    let pool_manifest_digest = format!("blake3:{}", manifest_hasher.finalize().to_hex());

    let generation = ProveKvPoolGenerationV1 {
        schema_version: "semantic_memory_provekv_pool_generation_v1".to_string(),
        generation_id,
        embedding_snapshot_digest: snapshot.embedding_snapshot_digest,
        source_digest: snapshot.source_digest,
        pool_manifest_digest,
        codec_family: "provekv_pool".to_string(),
        codec_profile: "semantic-memory-f32-derived-candidate-v1".to_string(),
        vector_dim: snapshot.vector_dim,
        item_count: item_map.len(),
        payload_bytes: payload.len() as u64,
        created_at: Utc::now(),
    };
    Ok((generation, payload, item_map))
}
