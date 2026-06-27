use context_governor::{compact_context, context_search, CompactRequest, Message, SearchScope};

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
fn context_search_handles_multibyte_char_boundaries() {
    let response = compact_context(CompactRequest {
        session_id: "unicode-search".into(),
        messages: vec![msg("tool", &format!("x{} NEEDLE", "→".repeat(2000)))],
        policy: Default::default(),
        focus: None,
    })
    .unwrap();

    let hits = context_search(&response, "NEEDLE", 1, SearchScope::All);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].snippet.contains("NEEDLE"));
}
