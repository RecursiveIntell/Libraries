use context_governor::{
    compact_context, context_expand, CompactRequest, CompactionPolicy, ExactRecoveryStateV1,
    FileContextStore, Message,
};

fn msg(role: &str, content: &str) -> Message {
    Message {
        id: None,
        role: role.into(),
        content: content.into(),
        name: None,
        metadata: Default::default(),
    }
}

#[test]
fn file_context_store_round_trips_receipt_and_exact_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let response = compact_context(CompactRequest {
        session_id: "store-session".into(),
        messages: vec![
            msg("system", "system"),
            msg("user", "start"),
            msg(
                "tool",
                &format!("{} STORE_NEEDLE", "verbose output ".repeat(600)),
            ),
            msg("user", "latest request"),
        ],
        policy: CompactionPolicy {
            target_tokens: 120,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let store = FileContextStore::new(dir.path());
    let receipt_id = response.receipt.receipt_id.clone();
    let saved_path = store.save(&response).unwrap();
    assert!(saved_path.exists());

    let loaded = store.load(&receipt_id).unwrap();
    assert_eq!(loaded.receipt.receipt_id, receipt_id);
    let item_id = loaded
        .exact_store
        .iter()
        .find(|item| item.content.contains("STORE_NEEDLE"))
        .unwrap()
        .item_id
        .clone();
    let expanded = context_expand(&loaded, &item_id, 100_000).unwrap();
    assert!(expanded.content.contains("STORE_NEEDLE"));

    let store_expanded = store.expand(&receipt_id, &item_id, 100_000).unwrap();
    assert!(store_expanded.content.contains("STORE_NEEDLE"));

    let hits = store
        .search("STORE_NEEDLE", 5, context_governor::SearchScope::All)
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].receipt_id, receipt_id);
    assert!(hits[0].hit.snippet.contains("STORE_NEEDLE"));

    let listed = store.list_receipts().unwrap();
    assert_eq!(listed, vec![receipt_id]);
}

#[test]
fn file_context_store_status_reports_lifecycle_bytes_and_cleans_tmp_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("stale.json.tmp"), "partial").unwrap();
    let response = compact_context(CompactRequest {
        session_id: "store-status".into(),
        messages: vec![
            msg("tool", &"bulk STORE_STATUS ".repeat(300)),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            target_tokens: 100,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let store = FileContextStore::new(dir.path());
    store.save(&response).unwrap();
    std::fs::write(dir.path().join("stale.json.tmp"), "partial").unwrap();
    let status = store.status().unwrap();

    assert_eq!(status.schema, "FileContextStoreStatusV1");
    assert_eq!(status.receipt_count, 1);
    assert!(status.total_bytes > 0);
    assert!(status.stale_tmp_files_removed >= 1);
    assert!(status.index_built);
    assert!(status.searchable);
    assert_eq!(
        status.last_receipt.as_deref(),
        Some(response.receipt.receipt_id.as_str())
    );
    assert!(!dir.path().join("stale.json.tmp").exists());
}

#[test]
fn file_context_store_persists_search_index_across_instances_and_prunes() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileContextStore::new(dir.path());
    let mut receipt_ids = Vec::new();

    for (session_id, needle) in [
        ("persist-a", "PERSIST_FIRST_NEEDLE"),
        ("persist-b", "PERSIST_SECOND_NEEDLE"),
    ] {
        let response = compact_context(CompactRequest {
            session_id: session_id.into(),
            messages: vec![
                msg("tool", &format!("{} {needle}", "bulk ".repeat(350))),
                msg("user", "latest"),
            ],
            policy: CompactionPolicy {
                target_tokens: 100,
                protect_last_n: 1,
                ..Default::default()
            },
            focus: None,
        })
        .unwrap();
        receipt_ids.push(response.receipt.receipt_id.clone());
        store.save(&response).unwrap();
    }

    let fresh_store = FileContextStore::new(dir.path());
    let status = fresh_store.status().unwrap();
    assert_eq!(status.receipt_count, 2);
    assert!(
        status.index_built,
        "save should persist an index usable by fresh processes"
    );
    assert!(status.searchable);
    assert_eq!(
        status.last_receipt.as_deref(),
        receipt_ids.last().map(String::as_str)
    );

    let hits = fresh_store
        .search(
            "PERSIST_FIRST_NEEDLE",
            5,
            context_governor::SearchScope::All,
        )
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].receipt_id, receipt_ids[0]);

    let pruned = fresh_store.prune_receipts_keep_last(1).unwrap();
    assert_eq!(pruned.removed_receipts, 1);
    let after = fresh_store.status().unwrap();
    assert_eq!(after.receipt_count, 1);
    assert_eq!(
        after.last_receipt.as_deref(),
        receipt_ids.last().map(String::as_str)
    );
    assert!(after.index_built);
    assert!(after.searchable);

    let first_hits = fresh_store
        .search(
            "PERSIST_FIRST_NEEDLE",
            5,
            context_governor::SearchScope::All,
        )
        .unwrap();
    assert!(first_hits.is_empty());
    let second_hits = fresh_store
        .search(
            "PERSIST_SECOND_NEEDLE",
            5,
            context_governor::SearchScope::All,
        )
        .unwrap();
    assert_eq!(second_hits.len(), 1);
    assert_eq!(second_hits[0].receipt_id, receipt_ids[1]);
}

#[test]
fn save_with_status_finalizes_exact_recovery_and_pruning_removes_it() {
    let dir = tempfile::tempdir().unwrap();
    let response = compact_context(CompactRequest {
        session_id: "save-status".into(),
        messages: vec![
            msg("tool", &"PERSISTENCE_NEEDLE ".repeat(500)),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            target_tokens: 100,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();
    let store = FileContextStore::new(dir.path());
    let saved = store.save_with_status(&response).unwrap();
    assert_eq!(saved.exact_recovery_state, ExactRecoveryStateV1::Persisted);
    assert!(saved.verified);

    let loaded = store.load(&response.receipt.receipt_id).unwrap();
    assert_eq!(
        loaded.receipt.summary_loss_report.exact_recovery_state,
        ExactRecoveryStateV1::Persisted
    );

    store.prune_receipts_keep_last(0).unwrap();
    assert!(store.load(&response.receipt.receipt_id).is_err());
}
