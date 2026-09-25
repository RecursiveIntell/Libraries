use rusqlite::Connection;
use semantic_memory::{
    AuthorityRelationQuarantinePlanError, MemoryConfig, MemoryStore, MockEmbedder, SqliteCellV1,
};
use sha2::{Digest, Sha256};
use std::path::Path;

type ResultT = Result<(), Box<dyn std::error::Error>>;
fn config(path: &Path) -> MemoryConfig {
    MemoryConfig {
        base_dir: path.to_path_buf(),
        ..Default::default()
    }
}
fn seed(path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let store = MemoryStore::open_with_embedder(config(path), Box::new(MockEmbedder::new(768)))?;
    drop(store);
    let db = Connection::open(path.join("memory.db"))?;
    db.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA foreign_keys=OFF;")?;
    Ok(db)
}
fn reader(path: &Path) -> Result<MemoryStore, Box<dyn std::error::Error>> {
    Ok(MemoryStore::open_existing_read_only_with_embedder(
        config(path),
        Box::new(MockEmbedder::new(768)),
    )?)
}
#[tokio::test]
async fn clean_image_is_explicit_noop_and_unchanged() -> ResultT {
    let tmp = tempfile::tempdir()?;
    drop(seed(tmp.path())?);
    let before = std::fs::read(tmp.path().join("memory.db"))?;
    let plan = reader(tmp.path())?
        .authority()
        .plan_orphaned_authority_quarantine(100, 100_000)
        .await?;
    assert_eq!(plan.disposition, "no_orphaned_authority_rows");
    assert!(plan.violations.is_empty());
    assert!(plan.rows.is_empty());
    assert_eq!(
        plan.database_sha256,
        format!("sha256:{:x}", Sha256::digest(&before))
    );
    assert_eq!(std::fs::read(tmp.path().join("memory.db"))?, before);
    Ok(())
}
#[tokio::test]
async fn five_family_orphan_plan_is_lossless_repeatable_and_bounded() -> ResultT {
    let tmp = tempfile::tempdir()?;
    let db = seed(tmp.path())?;
    db.execute_batch("INSERT INTO authority_lineages VALUES ('lineage-a', 'absent-a', 0);
        INSERT INTO authority_versions (fact_id,lineage_id,version,operation_kind,is_active,is_redacted,content_digest) VALUES ('absent-a','lineage-a',1,'append',1,0,'digest');
        INSERT INTO origin_authority_labels VALUES ('absent-a', '{\"z\":2, \"a\":1}', 'opaque', 'today');
        INSERT INTO origin_authority_revocations VALUES ('revoke-a','absent-a','key-a','principal','ref','today');
        INSERT INTO forgotten_facts VALUES ('absent-a','receipt-a','ns','digest','today');")?;
    drop(db);
    let image = tmp.path().join("memory.db");
    let before = std::fs::read(&image)?;
    let store = reader(tmp.path())?;
    let authority = store.authority();
    let plan = authority
        .plan_orphaned_authority_quarantine(100, 100_000)
        .await?;
    assert_eq!(plan.violations.len(), 5);
    assert_eq!(plan.rows.len(), 5);
    assert_eq!(plan.affected_fact_ids, vec!["absent-a"]);
    assert_eq!(plan.affected_lineage_ids, vec!["lineage-a"]);
    let label = plan
        .rows
        .iter()
        .find(|row| row.table == "origin_authority_labels")
        .ok_or("missing label row")?;
    let label_idx = label
        .columns
        .iter()
        .position(|name| name == "label_json")
        .ok_or("missing label_json")?;
    assert_eq!(
        label.cells[label_idx],
        SqliteCellV1::TextHex("7b227a223a322c202261223a317d".into())
    );
    assert_eq!(
        plan,
        authority
            .plan_orphaned_authority_quarantine(100, 100_000)
            .await?
    );
    assert_eq!(std::fs::read(&image)?, before);
    assert!(matches!(
        authority
            .plan_orphaned_authority_quarantine(4, 100_000)
            .await,
        Err(AuthorityRelationQuarantinePlanError::LimitExceeded)
    ));
    assert!(matches!(
        authority.plan_orphaned_authority_quarantine(100, 1).await,
        Err(AuthorityRelationQuarantinePlanError::LimitExceeded)
    ));
    assert!(matches!(
        authority
            .plan_orphaned_authority_quarantine(usize::MAX, 100_000)
            .await,
        Err(AuthorityRelationQuarantinePlanError::LimitExceeded)
    ));
    Ok(())
}

#[tokio::test]
async fn oversized_orphan_cell_is_refused_without_changing_image() -> ResultT {
    let tmp = tempfile::tempdir()?;
    let db = seed(tmp.path())?;
    db.execute_batch(
        "INSERT INTO origin_authority_labels(fact_id,label_json,label_digest,recorded_at)
        VALUES ('absent',zeroblob(120000),'digest','today');",
    )?;
    drop(db);
    let image = tmp.path().join("memory.db");
    let before = std::fs::read(&image)?;
    let result = reader(tmp.path())?
        .authority()
        .plan_orphaned_authority_quarantine(10, 100_000)
        .await;
    assert!(matches!(
        result,
        Err(AuthorityRelationQuarantinePlanError::LimitExceeded)
    ));
    assert_eq!(std::fs::read(&image)?, before);
    Ok(())
}

#[tokio::test]
async fn three_missing_parents_thirteen_rows_match_supported_topology() -> ResultT {
    let tmp = tempfile::tempdir()?;
    let db = seed(tmp.path())?;
    db.execute_batch("INSERT INTO authority_lineages VALUES ('l1','a1',0),('l2','a2',0),('l3','a3',0);
        INSERT INTO authority_versions(fact_id,lineage_id,version,operation_kind,is_active,content_digest)
        VALUES ('a1','l1',1,'append',1,'d1'),('a2','l2',1,'append',1,'d2'),('a3','l3',1,'append',1,'d3');
        INSERT INTO origin_authority_labels(fact_id,label_json,label_digest,recorded_at)
        VALUES ('a1','{}','d1','today'),('a2','{}','d2','today'),('a3','{}','d3','today');
        INSERT INTO origin_authority_revocations(revocation_id,fact_id,caller_idempotency_key,principal,revocation_reference,revoked_at)
        VALUES ('r1','a1','k1','p','ref','today'),('r2','a2','k2','p','ref','today');
        INSERT INTO forgotten_facts(fact_id,receipt_id,namespace,content_digest,forgotten_at)
        VALUES ('a1','receipt1','n','d1','today'),('a2','receipt2','n','d2','today');")?;
    drop(db);
    let image = tmp.path().join("memory.db");
    let before = std::fs::read(&image)?;
    let plan = reader(tmp.path())?
        .authority()
        .plan_orphaned_authority_quarantine(13, 100_000)
        .await?;
    assert_eq!(plan.violations.len(), 13);
    assert_eq!(plan.rows.len(), 13);
    assert_eq!(plan.affected_fact_ids, ["a1", "a2", "a3"]);
    assert_eq!(plan.affected_lineage_ids, ["l1", "l2", "l3"]);
    assert_eq!(std::fs::read(&image)?, before);
    Ok(())
}

#[tokio::test]
async fn rejects_unrecognized_fk_and_present_lineage_sibling() -> ResultT {
    let tmp = tempfile::tempdir()?;
    let db = seed(tmp.path())?;
    db.execute_batch("CREATE TABLE unexpected_reference(id INTEGER PRIMARY KEY, fact_id TEXT REFERENCES facts(id));
        INSERT INTO unexpected_reference VALUES (1, 'absent');")?;
    drop(db);
    let result = reader(tmp.path())?
        .authority()
        .plan_orphaned_authority_quarantine(100, 100_000)
        .await;
    assert!(matches!(
        result,
        Err(AuthorityRelationQuarantinePlanError::UnsupportedSchema)
    ));

    let tmp = tempfile::tempdir()?;
    let db = seed(tmp.path())?;
    db.execute_batch("INSERT INTO facts(id,namespace,content) VALUES ('present','n','fixture');
        INSERT INTO authority_lineages VALUES ('lineage','absent',0);
        INSERT INTO authority_versions(fact_id,lineage_id,version,operation_kind,is_active,content_digest)
        VALUES ('absent','lineage',1,'append',1,'digest-a');
        INSERT INTO authority_versions(fact_id,lineage_id,version,operation_kind,is_active,content_digest)
        VALUES ('present','lineage',2,'supersede',0,'digest-b');")?;
    drop(db);
    let result = reader(tmp.path())?
        .authority()
        .plan_orphaned_authority_quarantine(100, 100_000)
        .await;
    assert!(matches!(
        result,
        Err(AuthorityRelationQuarantinePlanError::UnsupportedShape(_))
    ));
    Ok(())
}

#[tokio::test]
async fn rejects_unrecognized_schema_even_when_user_version_is_39() -> ResultT {
    let tmp = tempfile::tempdir()?;
    let db = seed(tmp.path())?;
    db.execute_batch("CREATE TABLE unexpected_extra(id INTEGER PRIMARY KEY);")?;
    drop(db);
    let result = reader(tmp.path())?
        .authority()
        .plan_orphaned_authority_quarantine(100, 100_000)
        .await;
    assert!(matches!(
        result,
        Err(AuthorityRelationQuarantinePlanError::UnsupportedSchema)
    ));
    Ok(())
}

#[tokio::test]
async fn historical_shaped_extra_objects_remain_unsupported_and_unchanged() -> ResultT {
    // This is a hostile synthetic shape, NOT a supported historical schema or
    // an attempt to define the absent sync/routing owners from observed DDL.
    let tmp = tempfile::tempdir()?;
    let db = seed(tmp.path())?;
    db.execute_batch("INSERT INTO authority_lineages VALUES ('l1','a1',0),('l2','a2',0),('l3','a3',0);
        INSERT INTO authority_versions(fact_id,lineage_id,version,operation_kind,is_active,content_digest)
        VALUES ('a1','l1',1,'append',1,'d1'),('a2','l2',1,'append',1,'d2'),('a3','l3',1,'append',1,'d3');
        INSERT INTO origin_authority_labels(fact_id,label_json,label_digest,recorded_at)
        VALUES ('a1','{}','d1','today'),('a2','{}','d2','today'),('a3','{}','d3','today');
        INSERT INTO origin_authority_revocations(revocation_id,fact_id,caller_idempotency_key,principal,revocation_reference,revoked_at)
        VALUES ('r1','a1','k1','p','ref','today'),('r2','a2','k2','p','ref','today');
        INSERT INTO forgotten_facts(fact_id,receipt_id,namespace,content_digest,forgotten_at)
        VALUES ('a1','receipt1','n','d1','today'),('a2','receipt2','n','d2','today');
        CREATE TABLE sync_state(key TEXT PRIMARY KEY, value TEXT);
        CREATE TABLE routing_policy(id TEXT PRIMARY KEY);
        CREATE INDEX idx_mutation_journal_sync ON mutation_journal(sequence);
        CREATE INDEX idx_graph_edges_invalidated ON graph_edges(is_invalidated);
        CREATE INDEX idx_graph_edges_source_target ON graph_edges(source,target);")?;
    drop(db);
    let image = tmp.path().join("memory.db");
    let before = std::fs::read(&image)?;
    let result = reader(tmp.path())?
        .authority()
        .plan_orphaned_authority_quarantine(100, 100_000)
        .await;
    assert!(matches!(
        result,
        Err(AuthorityRelationQuarantinePlanError::UnsupportedSchema)
    ));
    assert_eq!(std::fs::read(&image)?, before);
    Ok(())
}

#[tokio::test]
async fn rejects_wrong_mode_and_schema() -> ResultT {
    let tmp = tempfile::tempdir()?;
    let db = seed(tmp.path())?;
    let writable =
        MemoryStore::open_with_embedder(config(tmp.path()), Box::new(MockEmbedder::new(768)))?;
    assert!(matches!(
        writable
            .authority()
            .plan_orphaned_authority_quarantine(10, 10000)
            .await,
        Err(AuthorityRelationQuarantinePlanError::RequiresReadOnlyStore)
    ));
    drop(writable);
    db.execute_batch("PRAGMA user_version=38")?;
    drop(db);
    let result = reader(tmp.path())?
        .authority()
        .plan_orphaned_authority_quarantine(10, 10000)
        .await;
    assert!(matches!(
        result,
        Err(AuthorityRelationQuarantinePlanError::NotSealed)
    ));
    let db = Connection::open(tmp.path().join("memory.db"))?;
    db.execute_batch("PRAGMA journal_mode=DELETE;")?;
    drop(db);
    let result = reader(tmp.path())?
        .authority()
        .plan_orphaned_authority_quarantine(10, 10000)
        .await;
    assert!(matches!(
        result,
        Err(AuthorityRelationQuarantinePlanError::UnsupportedSchema)
    ));
    Ok(())
}
