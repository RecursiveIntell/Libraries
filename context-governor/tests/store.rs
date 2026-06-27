use context_governor::{
    compact_context, context_expand, CompactRequest, CompactionPolicy, FileContextStore, Message,
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
