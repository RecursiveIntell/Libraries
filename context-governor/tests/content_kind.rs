use context_governor::{compact_context, CompactRequest, CompactionPolicy, ContentKind, Message};

fn msg(role: &str, content: &str) -> Message {
    Message {
        id: None,
        role: role.into(),
        content: content.into(),
        name: None,
        metadata: Default::default(),
    }
}

fn kind_for(content: &str) -> ContentKind {
    let response = compact_context(CompactRequest {
        session_id: "kind".into(),
        messages: vec![msg("assistant", content), msg("user", "latest")],
        policy: CompactionPolicy {
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();
    response.allocation_plan.items[0].content_kind.clone()
}

#[test]
fn detects_json_diff_cargo_rust_markdown_and_plain_text() {
    assert_eq!(kind_for(r#"{"alpha": 1, "beta": true}"#), ContentKind::Json);
    assert_eq!(
        kind_for("diff --git a/src/lib.rs b/src/lib.rs\n@@ -1 +1 @@\n-old\n+new"),
        ContentKind::Diff
    );
    assert_eq!(
        kind_for("error[E0425]: cannot find value\nwarning: unused import\ntest result: FAILED"),
        ContentKind::CargoOutput
    );
    assert_eq!(
        kind_for("fn main() { println!(\"hi\"); }"),
        ContentKind::Rust
    );
    assert_eq!(kind_for("# Heading\n\n- bullet"), ContentKind::Markdown);
    assert_eq!(
        kind_for("ordinary prose with no structure"),
        ContentKind::PlainText
    );
}

#[test]
fn detects_search_results_before_generic_tool_logs() {
    assert_eq!(
        kind_for(
            "/home/demo/src/lib.rs:42: matching line\n/home/demo/tests/store.rs:9: another match"
        ),
        ContentKind::SearchResults
    );
}
