use aidens_tool_kit::{ToolDispatcher, ToolRegistryV1};
use serde_json::json;

#[tokio::test]
async fn repo_read_tool_executes_and_emits_receipt() {
    let tmp = std::env::temp_dir().join(format!("aidens-next-tool-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("README.md"), "hello from repo").unwrap();

    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&tmp).expect("registry");
    let dispatcher = ToolDispatcher::new(registry);
    let outcome = dispatcher
        .invoke("aidens:repo-read:1", json!({ "path": "README.md" }))
        .await
        .expect("repo-read succeeds");

    assert!(outcome.output_text().contains("hello from repo"));
    assert_eq!(outcome.receipt.tool_id, "aidens:repo-read:1");
    assert!(outcome.receipt.succeeded);

    let traversal = dispatcher
        .invoke("aidens:repo-read:1", json!({ "path": "../secret.txt" }))
        .await
        .expect_err("traversal is rejected");
    let text = traversal.to_string().to_ascii_lowercase();
    assert!(text.contains("traversal") || text.contains("sandbox") || text.contains("escape"));

    let _ = std::fs::remove_dir_all(&tmp);
}
