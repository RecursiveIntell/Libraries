//! One-time governed migration of facts from a retired semantic-memory store.
//!
//! This operator utility is intentionally available only with the `testing`
//! feature because that feature exposes the in-process test authority issuer.
//! Production builds do not contain a caller-mintable issuer.

use rusqlite::Connection;
use semantic_memory::{embedder::CandleEmbedder, AuthorityPermit, MemoryConfig, MemoryStore};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Candidate {
    legacy_id: String,
    namespace: String,
    content: String,
    source: Option<String>,
    metadata: Option<Value>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct MigratedReceipt {
    legacy_id: String,
    namespace: String,
    receipt_id: String,
    receipt_digest: String,
    affected_ids: Vec<String>,
}

fn load_unique_candidates(
    legacy_db: &Path,
    canonical_db: &Path,
) -> Result<Vec<Candidate>, Box<dyn std::error::Error>> {
    let connection =
        Connection::open_with_flags(legacy_db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    connection.execute(
        "ATTACH DATABASE ?1 AS canonical",
        [canonical_db.to_string_lossy().as_ref()],
    )?;
    let mut statement = connection.prepare(
        "SELECT MIN(l.id), l.namespace, l.content, MIN(l.source), MIN(l.metadata),
                MIN(l.created_at), MAX(l.updated_at)
         FROM facts l
         WHERE NOT EXISTS (
             SELECT 1 FROM canonical.facts c
             WHERE c.namespace = l.namespace AND c.content = l.content
         )
         GROUP BY l.namespace, l.content
         ORDER BY l.namespace, l.content",
    )?;
    let rows = statement.query_map([], |row| {
        let metadata_raw: Option<String> = row.get(4)?;
        Ok(Candidate {
            legacy_id: row.get(0)?,
            namespace: row.get(1)?,
            content: row.get(2)?,
            source: row.get(3)?,
            metadata: metadata_raw
                .as_deref()
                .and_then(|raw| serde_json::from_str(raw).ok()),
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn idempotency_key(candidate: &Candidate) -> String {
    let mut material = Vec::new();
    for field in [
        candidate.legacy_id.as_str(),
        candidate.namespace.as_str(),
        candidate.content.as_str(),
    ] {
        material.extend_from_slice(&(field.len() as u64).to_be_bytes());
        material.extend_from_slice(field.as_bytes());
    }
    format!(
        "legacy-store-migration:v1:{}",
        blake3::hash(&material).to_hex()
    )
}

fn migration_metadata(candidate: &Candidate, legacy_db: &Path) -> Value {
    let mut metadata = match candidate.metadata.clone() {
        Some(Value::Object(object)) => object,
        Some(other) => {
            let mut object = serde_json::Map::new();
            object.insert("legacy_metadata".into(), other);
            object
        }
        None => serde_json::Map::new(),
    };
    metadata.insert(
        "legacy_store_migration".into(),
        json!({
            "schema_version": "semantic-memory-legacy-store-migration-v1",
            "legacy_fact_id": candidate.legacy_id,
            "legacy_store": legacy_db,
            "legacy_created_at": candidate.created_at,
            "legacy_updated_at": candidate.updated_at,
        }),
    );
    Value::Object(metadata)
}

fn required_path(arguments: &mut impl Iterator<Item = String>, name: &str) -> PathBuf {
    arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let legacy_dir = required_path(&mut arguments, "LEGACY_DIR");
    let canonical_dir = required_path(&mut arguments, "CANONICAL_DIR");
    let report_path = required_path(&mut arguments, "REPORT_PATH");
    let apply = arguments.next().as_deref() == Some("--apply");
    if arguments.next().is_some() {
        return Err("unexpected extra arguments".into());
    }

    let legacy_db = legacy_dir.join("memory.db");
    let canonical_db = canonical_dir.join("memory.db");
    let candidates = load_unique_candidates(&legacy_db, &canonical_db)?;
    let counts = candidates
        .iter()
        .fold(BTreeMap::new(), |mut counts, candidate| {
            *counts.entry(candidate.namespace.clone()).or_insert(0usize) += 1;
            counts
        });
    println!("unique legacy candidates: {}", candidates.len());
    for (namespace, count) in &counts {
        println!("  {namespace}: {count}");
    }
    if !apply {
        println!("dry run only; pass --apply to perform governed appends");
        return Ok(());
    }

    let config = MemoryConfig {
        base_dir: canonical_dir,
        ..Default::default()
    };
    let embedder = Box::new(CandleEmbedder::try_new(&config.embedding)?);
    let store = MemoryStore::open_with_embedder(config, embedder)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let mut receipts = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let receipt = store
                .authority()
                .append_with_metadata(
                    AuthorityPermit::operator_system(
                        "operator:legacy-store-migration",
                        "semantic-memory-legacy-migrator",
                        AuthorityPermit::APPEND_CAPABILITY,
                    ),
                    idempotency_key(&candidate),
                    candidate.namespace.clone(),
                    candidate.content.clone(),
                    candidate.source.clone(),
                    Some(migration_metadata(&candidate, &legacy_db)),
                )
                .await?;
            receipts.push(MigratedReceipt {
                legacy_id: candidate.legacy_id,
                namespace: candidate.namespace,
                receipt_id: receipt.receipt_id,
                receipt_digest: receipt.receipt_digest,
                affected_ids: receipt.affected_ids,
            });
        }

        let remaining = load_unique_candidates(&legacy_db, &canonical_db)?;
        let report = json!({
            "schema_version": "semantic-memory-legacy-store-migration-report-v1",
            "legacy_store": legacy_db,
            "canonical_store": canonical_db,
            "migrated_count": receipts.len(),
            "remaining_unique_count": remaining.len(),
            "receipts": receipts,
        });
        std::fs::write(&report_path, serde_json::to_vec_pretty(&report)?)?;
        if !remaining.is_empty() {
            return Err(format!("{} legacy facts remain after migration", remaining.len()).into());
        }
        println!(
            "governed migration complete; report: {}",
            report_path.display()
        );
        Ok(())
    })
}
