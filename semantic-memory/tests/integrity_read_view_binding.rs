//! Actual MemoryStore regressions: evidence must stay bound to the opened view.
//! Compare complete stable proposals so these assertions exercise both V1 (red)
//! and V2 (green), without a production compatibility alias or renamed-field red.
use rusqlite::Connection;
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use std::path::Path;

type ResultT = Result<(), Box<dyn std::error::Error>>;
fn config(path: &Path) -> MemoryConfig {
    MemoryConfig {
        base_dir: path.to_path_buf(),
        ..Default::default()
    }
}
fn seed(path: &Path, orphan: Option<&str>) -> ResultT {
    drop(MemoryStore::open_with_embedder(
        config(path),
        Box::new(MockEmbedder::new(768)),
    )?);
    let db = Connection::open(path.join("memory.db"))?;
    db.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA foreign_keys=OFF;")?;
    if let Some(id) = orphan {
        db.execute(
            "INSERT INTO origin_authority_labels VALUES (?1,'{}','digest','today')",
            [id],
        )?;
    }
    Ok(())
}
fn reader(path: &Path) -> Result<MemoryStore, Box<dyn std::error::Error>> {
    Ok(MemoryStore::open_existing_read_only_with_embedder(
        config(path),
        Box::new(MockEmbedder::new(768)),
    )?)
}

#[cfg(unix)]
#[tokio::test]
async fn pathname_replacement_never_mixes_rows_and_image_evidence() -> ResultT {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let replacement = root.path().join("replacement");
    seed(&source, Some("original-orphan"))?;
    seed(&replacement, None)?;
    let original_bytes = std::fs::read(source.join("memory.db"))?;
    let replacement_bytes = std::fs::read(replacement.join("memory.db"))?;
    let store = reader(&source)?;
    let before = store
        .authority()
        .plan_orphaned_authority_quarantine(100, 100_000)
        .await?;
    assert_eq!(before.affected_fact_ids, ["original-orphan"]);
    std::fs::rename(source.join("memory.db"), source.join("original.db"))?;
    std::fs::rename(replacement.join("memory.db"), source.join("memory.db"))?;
    let after = store
        .authority()
        .plan_orphaned_authority_quarantine(100, 100_000)
        .await?;
    assert_eq!(
        after, before,
        "a retained owner reader must never attach replacement-image evidence to original rows"
    );
    let fresh = reader(&source)?
        .authority()
        .plan_orphaned_authority_quarantine(100, 100_000)
        .await?;
    assert!(fresh.rows.is_empty());
    assert_ne!(after, fresh);
    assert_eq!(std::fs::read(source.join("original.db"))?, original_bytes);
    assert_eq!(std::fs::read(source.join("memory.db"))?, replacement_bytes);
    Ok(())
}

#[test]
fn relative_path_cwd_change_never_mixes_rows_and_image_evidence() -> ResultT {
    // cwd is process-global. Run the hostile case in a dedicated test process;
    // neither this integration binary's other tests nor Cargo are raced.
    let root = tempfile::tempdir()?;
    let output = std::process::Command::new(std::env::current_exe()?)
        .args([
            "--exact",
            "relative_path_cwd_child",
            "--ignored",
            "--nocapture",
        ])
        .env("SEMANTIC_MEMORY_CWD_BINDING_ROOT", root.path())
        .output()?;
    assert!(
        output.status.success(),
        "isolated cwd regression failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
#[ignore = "only launched by the parent regression in an isolated process"]
fn relative_path_cwd_child() -> ResultT {
    let root = std::env::var_os("SEMANTIC_MEMORY_CWD_BINDING_ROOT")
        .ok_or("missing isolated cwd fixture root")?;
    let root = Path::new(&root);
    let a = root.join("a");
    let b = root.join("b");
    seed(&a.join("store"), Some("relative-orphan"))?;
    seed(&b.join("store"), None)?;
    let a_before = std::fs::read(a.join("store/memory.db"))?;
    let b_before = std::fs::read(b.join("store/memory.db"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        std::env::set_current_dir(&a)?;
        let store = reader(Path::new("store"))?;
        let before = store
            .authority()
            .plan_orphaned_authority_quarantine(100, 100_000)
            .await?;
        assert_eq!(before.affected_fact_ids, ["relative-orphan"]);
        std::env::set_current_dir(&b)?;
        let after = store
            .authority()
            .plan_orphaned_authority_quarantine(100, 100_000)
            .await?;
        assert_eq!(
            after, before,
            "cwd must not rebind opened-reader evidence to another same-named image"
        );
        assert_eq!(std::fs::read(a.join("store/memory.db"))?, a_before);
        assert_eq!(std::fs::read(b.join("store/memory.db"))?, b_before);
        Ok(())
    })
}
