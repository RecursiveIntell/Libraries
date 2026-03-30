//! HNSW sidecar lifecycle helpers.

#[cfg(feature = "hnsw")]
use crate::episodes;
#[cfg(feature = "hnsw")]
use crate::error::MemoryError;
#[cfg(feature = "hnsw")]
use crate::StoragePaths;
#[cfg(feature = "hnsw")]
use crate::{db, pool::SqlitePool, MemoryStoreInner};
#[cfg(feature = "hnsw")]
use crate::{hnsw::HnswConfig, hnsw::HnswIndex};
#[cfg(feature = "hnsw")]
use rusqlite::Connection;

#[cfg(feature = "hnsw")]
pub(crate) fn ensure_hnsw_dir(dir: &std::path::Path) -> Result<(), MemoryError> {
    std::fs::create_dir_all(dir).map_err(|err| {
        MemoryError::StorageError(format!(
            "failed to create HNSW directory {}: {}",
            dir.display(),
            err
        ))
    })
}

#[cfg(feature = "hnsw")]
pub(crate) fn save_hnsw_sidecar(
    index: &HnswIndex,
    dir: &std::path::Path,
    basename: &str,
) -> Result<(), MemoryError> {
    ensure_hnsw_dir(dir)?;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| index.save(dir, basename))).map_err(
        |_| {
            MemoryError::HnswError(format!(
                "failed to save HNSW sidecar under {}: underlying hnsw writer panicked",
                dir.display()
            ))
        },
    )?
}

#[cfg(feature = "hnsw")]
pub(crate) fn rebuild_hnsw_from_sqlite(
    conn: &Connection,
    config: &HnswConfig,
) -> Result<HnswIndex, MemoryError> {
    let new_index = HnswIndex::new(config.clone())?;

    // Load fact embeddings
    {
        let mut stmt =
            conn.prepare("SELECT id, embedding FROM facts WHERE embedding IS NOT NULL")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;
        for row in rows {
            let (id, blob) = row?;
            if let Ok(emb) = db::bytes_to_embedding(&blob) {
                let key = format!("fact:{}", id);
                if let Err(e) = new_index.insert(key.clone(), &emb) {
                    tracing::warn!("Failed to insert {} into HNSW: {}", key, e);
                }
            }
        }
    }

    // Load chunk embeddings
    {
        let mut stmt =
            conn.prepare("SELECT id, embedding FROM chunks WHERE embedding IS NOT NULL")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;
        for row in rows {
            let (id, blob) = row?;
            if let Ok(emb) = db::bytes_to_embedding(&blob) {
                let key = format!("chunk:{}", id);
                if let Err(e) = new_index.insert(key.clone(), &emb) {
                    tracing::warn!("Failed to insert {} into HNSW: {}", key, e);
                }
            }
        }
    }

    // Load message embeddings
    {
        let mut stmt =
            conn.prepare("SELECT id, embedding FROM messages WHERE embedding IS NOT NULL")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;
        for row in rows {
            let (id, blob) = row?;
            if let Ok(emb) = db::bytes_to_embedding(&blob) {
                let key = format!("msg:{}", id);
                if let Err(e) = new_index.insert(key.clone(), &emb) {
                    tracing::warn!("Failed to insert {} into HNSW: {}", key, e);
                }
            }
        }
    }

    // Load episode embeddings (keyed by episode_id)
    {
        let mut stmt =
            conn.prepare("SELECT episode_id, embedding FROM episodes WHERE embedding IS NOT NULL")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;
        for row in rows {
            let (episode_id, blob) = row?;
            if let Ok(emb) = db::bytes_to_embedding(&blob) {
                let key = episodes::episode_item_key(&episode_id);
                if let Err(e) = new_index.insert(key.clone(), &emb) {
                    tracing::warn!("Failed to insert {} into HNSW: {}", key, e);
                }
            }
        }
    }

    Ok(new_index)
}

#[cfg(feature = "hnsw")]
pub(crate) fn sync_pending_hnsw_sidecar(inner: &MemoryStoreInner) -> Result<usize, MemoryError> {
    inner.pool.with_write_conn(|conn| {
        let pending_ops = db::list_pending_index_ops(conn)?;
        if pending_ops.is_empty() {
            if !db::is_sidecar_dirty(conn)? {
                return Ok(0);
            }

            let guard = inner.hnsw_index.read().unwrap_or_else(|e| e.into_inner());
            save_hnsw_sidecar(&guard, &inner.paths.hnsw_dir, &inner.paths.hnsw_basename)?;
            guard.flush_keymap(conn)?;
            guard.update_last_flush_epoch();
            db::set_sidecar_dirty(conn, false)?;
            return Ok(0);
        }

        let result: Result<usize, MemoryError> = (|| {
            let guard = inner.hnsw_index.write().unwrap_or_else(|e| e.into_inner());

            for op in &pending_ops {
                match op.op_kind {
                    db::IndexOpKind::Upsert => {
                        match db::load_embedding_for_index_key(conn, &op.item_key)? {
                            Some(embedding) => guard.insert(op.item_key.clone(), &embedding)?,
                            None => {
                                // Source row no longer exists; the desired end-state is absence.
                                guard.delete(&op.item_key)?;
                            }
                        }
                    }
                    db::IndexOpKind::Delete => {
                        guard.delete(&op.item_key)?;
                    }
                }
            }

            let processed_keys: Vec<String> =
                pending_ops.iter().map(|op| op.item_key.clone()).collect();
            save_hnsw_sidecar(&guard, &inner.paths.hnsw_dir, &inner.paths.hnsw_basename)?;
            guard.flush_keymap(conn)?;
            guard.update_last_flush_epoch();
            db::clear_pending_index_ops(conn, &processed_keys)?;
            db::set_sidecar_dirty(conn, false)?;
            Ok(pending_ops.len())
        })();

        if let Err(err) = result {
            let err_text = err.to_string();
            let keys: Vec<String> = pending_ops.iter().map(|op| op.item_key.clone()).collect();
            if let Err(mark_err) = db::mark_pending_index_ops_failed(conn, &keys, &err_text) {
                tracing::warn!(
                    error = %mark_err,
                    "failed to mark pending HNSW index ops as failed"
                );
            }
            return Err(err);
        }

        result
    })
}

#[cfg(feature = "hnsw")]
pub(crate) fn recover_hnsw_sidecar_sync(
    pool: &SqlitePool,
    paths: &StoragePaths,
    config: &HnswConfig,
) -> Result<HnswIndex, MemoryError> {
    let recovered = pool.with_read_conn(|conn| rebuild_hnsw_from_sqlite(conn, config))?;
    save_hnsw_sidecar(&recovered, &paths.hnsw_dir, &paths.hnsw_basename)?;
    pool.with_write_conn(|conn| {
        recovered.flush_keymap(conn)?;
        db::clear_all_pending_index_ops(conn)?;
        db::set_sidecar_dirty(conn, false)?;
        Ok(())
    })?;
    Ok(recovered)
}
