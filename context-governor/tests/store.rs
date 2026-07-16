use context_governor::{
    compact_context, context_expand, context_search, CompactRequest, CompactionPolicy,
    ExactRecoveryStateV1, FileContextStore, Message, SearchScope,
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
    assert!(
        !status.index_built,
        "save must not rebuild the global index"
    );
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
        !status.index_built,
        "save must leave indexing to the first search"
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
    assert!(fresh_store.status().unwrap().index_built);

    let pruned = fresh_store.prune_receipts_keep_last(1).unwrap();
    assert_eq!(pruned.removed_receipts, 1);
    let after = fresh_store.status().unwrap();
    assert_eq!(after.receipt_count, 1);
    assert_eq!(
        after.last_receipt.as_deref(),
        receipt_ids.last().map(String::as_str)
    );
    assert!(
        after.index_built,
        "prune must update an existing index instead of deleting it"
    );
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

#[test]
fn overwriting_a_receipt_updates_the_persisted_index_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileContextStore::new(dir.path());
    let mut response = compact_context(CompactRequest {
        session_id: "overwrite-index".into(),
        messages: vec![
            msg("tool", &"OLD_INDEX_TOKEN ".repeat(300)),
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

    store.save(&response).unwrap();
    assert!(!store
        .search("OLD_INDEX_TOKEN", 10, context_governor::SearchScope::All)
        .unwrap()
        .is_empty());
    assert!(store.status().unwrap().index_built);

    for item in &mut response.exact_store {
        item.content = item.content.replace("OLD_INDEX_TOKEN", "NEW_INDEX_TOKEN");
    }
    for message in &mut response.compacted_messages {
        message.content = message
            .content
            .replace("OLD_INDEX_TOKEN", "NEW_INDEX_TOKEN");
    }
    store.save(&response).unwrap();

    let fresh = FileContextStore::new(dir.path());
    assert!(
        fresh.status().unwrap().index_built,
        "overwrite must preserve a query-ready index"
    );
    assert!(fresh
        .search("OLD_INDEX_TOKEN", 10, context_governor::SearchScope::All)
        .unwrap()
        .is_empty());
    assert!(!fresh
        .search("NEW_INDEX_TOKEN", 10, context_governor::SearchScope::All)
        .unwrap()
        .is_empty());
}

#[test]
fn saving_a_new_receipt_updates_an_existing_index_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileContextStore::new(dir.path());
    let first = indexed_fixture(
        "incremental-append-first",
        &format!("{} APPEND_FIRST_NEEDLE", "historical ".repeat(300)),
    );
    let second = indexed_fixture(
        "incremental-append-second",
        &format!("{} APPEND_SECOND_NEEDLE", "historical ".repeat(300)),
    );

    store.save(&first).unwrap();
    assert_eq!(
        store
            .search("APPEND_FIRST_NEEDLE", 10, SearchScope::All)
            .unwrap()
            .len(),
        1
    );
    assert!(store.status().unwrap().index_built);

    store.save(&second).unwrap();

    let fresh = FileContextStore::new(dir.path());
    assert!(
        fresh.status().unwrap().index_built,
        "append must preserve a query-ready index"
    );
    assert_eq!(
        fresh
            .search("APPEND_FIRST_NEEDLE", 10, SearchScope::All)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        fresh
            .search("APPEND_SECOND_NEEDLE", 10, SearchScope::All)
            .unwrap()
            .len(),
        1
    );
}

fn indexed_fixture(session_id: &str, content: &str) -> context_governor::CompactResponse {
    compact_context(CompactRequest {
        session_id: session_id.into(),
        messages: vec![msg("tool", content), msg("user", "latest")],
        policy: CompactionPolicy {
            target_tokens: 80,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap()
}

#[test]
fn indexed_search_matches_authoritative_scan_for_punctuated_substrings() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileContextStore::new(dir.path());
    let punctuated = indexed_fixture(
        "punctuated",
        &format!("{} NEEDLE_PARSER", "historical ".repeat(300)),
    );
    let exact = indexed_fixture(
        "exact-token",
        &format!("{} NEEDLE", "historical ".repeat(300)),
    );
    let unpunctuated = indexed_fixture(
        "unpunctuated-substring",
        &format!("{} PRENEEDLEPOST", "historical ".repeat(300)),
    );
    let unicode = indexed_fixture(
        "unicode-substring",
        &format!("{} 前NEEDLE後", "historical ".repeat(300)),
    );
    store.save(&punctuated).unwrap();
    store.save(&exact).unwrap();
    store.save(&unpunctuated).unwrap();
    store.save(&unicode).unwrap();

    let expected = store
        .list_receipts()
        .unwrap()
        .into_iter()
        .filter(|receipt_id| {
            let response = store.load(receipt_id).unwrap();
            !context_search(&response, "NEEDLE", 10, SearchScope::All).is_empty()
        })
        .collect::<std::collections::BTreeSet<_>>();
    let actual = store
        .search("NEEDLE", 10, SearchScope::All)
        .unwrap()
        .into_iter()
        .map(|hit| hit.receipt_id)
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 4);
}

#[test]
fn corrupt_legacy_index_is_migrated_to_a_compact_rebuildable_index() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileContextStore::new(dir.path());
    let response = indexed_fixture(
        "legacy-index-migration",
        &format!("{} MIGRATION_NEEDLE", "historical ".repeat(300)),
    );
    store.save(&response).unwrap();
    let receipt_path = dir
        .path()
        .join(format!("{}.json", response.receipt.receipt_id));
    let receipt_before = std::fs::read(&receipt_path).unwrap();
    let legacy_index_path = dir.path().join(".receipt-index.json");
    let legacy_bytes = b"{truncated";
    std::fs::write(&legacy_index_path, legacy_bytes).unwrap();

    let hits = FileContextStore::new(dir.path())
        .search("MIGRATION_NEEDLE", 10, SearchScope::All)
        .unwrap();

    assert_eq!(hits.len(), 1);
    assert!(dir.path().join(".receipt-index.sqlite3").exists());
    assert_eq!(std::fs::read(legacy_index_path).unwrap(), legacy_bytes);
    assert_eq!(std::fs::read(receipt_path).unwrap(), receipt_before);
}

#[test]
fn concurrent_same_id_writers_publish_one_complete_receipt_and_leave_no_temp_files() {
    let dir = tempfile::tempdir().unwrap();
    let first = indexed_fixture(
        "concurrent-same-id",
        &format!("{} FIRST_WRITER", "historical ".repeat(300)),
    );
    let mut second = indexed_fixture(
        "concurrent-second-payload",
        &format!("{} SECOND_WRITER", "historical ".repeat(300)),
    );
    second.receipt.receipt_id = first.receipt.receipt_id.clone();

    let root = dir.path().to_path_buf();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let spawn_writer = |response: context_governor::CompactResponse| {
        let root = root.clone();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            FileContextStore::new(root).save(&response)
        })
    };
    let first_writer = spawn_writer(first.clone());
    let second_writer = spawn_writer(second.clone());
    barrier.wait();
    first_writer.join().unwrap().unwrap();
    second_writer.join().unwrap().unwrap();

    let loaded = FileContextStore::new(dir.path())
        .load(&first.receipt.receipt_id)
        .unwrap();
    let rendered = serde_json::to_string(&loaded).unwrap();
    assert_ne!(
        rendered.contains("FIRST_WRITER"),
        rendered.contains("SECOND_WRITER")
    );
    assert!(std::fs::read_dir(dir.path()).unwrap().all(|entry| {
        entry
            .unwrap()
            .path()
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("tmp")
    }));
}

#[test]
fn corrupt_compact_index_rebuilds_from_authoritative_receipts() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileContextStore::new(dir.path());
    let response = indexed_fixture(
        "compact-index-rebuild",
        &format!("{} REBUILD_NEEDLE", "historical ".repeat(300)),
    );
    store.save(&response).unwrap();
    store
        .search("REBUILD_NEEDLE", 10, SearchScope::All)
        .unwrap();
    let index_path = dir.path().join(".receipt-index.sqlite3");
    assert!(index_path.exists());
    std::fs::write(&index_path, b"not a sqlite database").unwrap();

    let hits = FileContextStore::new(dir.path())
        .search("REBUILD_NEEDLE", 10, SearchScope::All)
        .unwrap();

    assert_eq!(hits.len(), 1);
    assert!(std::fs::metadata(index_path).unwrap().len() > 64);
}

#[test]
fn corrupt_trigram_signature_falls_back_exactly_and_is_quarantined() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileContextStore::new(dir.path());
    let response = indexed_fixture(
        "signature-corruption",
        &format!("{} SIGNATURE_NEEDLE", "historical ".repeat(300)),
    );
    store.save(&response).unwrap();
    store
        .search("SIGNATURE_NEEDLE", 10, SearchScope::All)
        .unwrap();
    let index_path = dir.path().join(".receipt-index.sqlite3");
    let connection = rusqlite::Connection::open(&index_path).unwrap();
    connection
        .execute(
            "UPDATE receipts SET trigram_hashes = X'00' WHERE receipt_id = ?1",
            rusqlite::params![response.receipt.receipt_id],
        )
        .unwrap();
    drop(connection);

    let hits = store
        .search("SIGNATURE_NEEDLE", 10, SearchScope::All)
        .unwrap();

    assert_eq!(hits.len(), 1, "corruption must not create a false negative");
    assert!(
        !store.status().unwrap().index_built,
        "corrupt derived signatures must be quarantined after exact fallback"
    );
    assert_eq!(
        store
            .search("SIGNATURE_NEEDLE", 10, SearchScope::All)
            .unwrap()
            .len(),
        1
    );
    assert!(store.status().unwrap().index_built);
}

#[test]
fn external_receipt_replacement_is_reconciled_without_stale_hits() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileContextStore::new(dir.path());
    let mut response = indexed_fixture(
        "external-replacement",
        &format!("{} EXTERNAL_OLD_NEEDLE", "historical ".repeat(300)),
    );
    store.save(&response).unwrap();
    store
        .search("EXTERNAL_OLD_NEEDLE", 10, SearchScope::All)
        .unwrap();

    for item in &mut response.exact_store {
        item.content = item
            .content
            .replace("EXTERNAL_OLD_NEEDLE", "EXTERNAL_NEW_NEEDLE");
    }
    for message in &mut response.compacted_messages {
        message.content = message
            .content
            .replace("EXTERNAL_OLD_NEEDLE", "EXTERNAL_NEW_NEEDLE");
    }
    let receipt_path = dir
        .path()
        .join(format!("{}.json", response.receipt.receipt_id));
    std::fs::write(&receipt_path, serde_json::to_vec_pretty(&response).unwrap()).unwrap();

    assert!(store
        .search("EXTERNAL_OLD_NEEDLE", 10, SearchScope::All)
        .unwrap()
        .is_empty());
    assert_eq!(
        store
            .search("EXTERNAL_NEW_NEEDLE", 10, SearchScope::All)
            .unwrap()
            .len(),
        1
    );
    assert!(store.status().unwrap().index_built);
}

#[test]
fn expand_rejects_tampered_exact_content() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileContextStore::new(dir.path());
    let response = indexed_fixture(
        "tamper-expand",
        &format!("{} TAMPER_NEEDLE", "historical ".repeat(300)),
    );
    let receipt_id = response.receipt.receipt_id.clone();
    store.save(&response).unwrap();
    let item_id = response.exact_store[0].item_id.clone();
    let path = dir.path().join(format!("{receipt_id}.json"));
    let mut persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    persisted["exact_store"][0]["content"] = serde_json::Value::String("tampered".into());
    std::fs::write(&path, serde_json::to_vec_pretty(&persisted).unwrap()).unwrap();

    assert!(FileContextStore::new(dir.path())
        .expand(&receipt_id, &item_id, 100_000)
        .is_err());
}
