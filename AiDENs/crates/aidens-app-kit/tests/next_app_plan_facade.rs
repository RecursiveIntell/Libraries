use aidens_app_kit::{AiDENsApp, AiDENsProfile};

#[tokio::test]
async fn app_from_explicit_mock_config_runs_through_facade() {
    let dir = std::env::temp_dir().join(format!("aidens-next-app-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cfg = dir.join("aidens.toml");
    std::fs::write(
        &cfg,
        r#"
app_id = "next-app"
memory_mode = "disabled"
receipt_level = "full"

[provider]
kind = "mock"
model = "mock-model"
mock_response = "facade mock response"

[receipts]
store_root = "receipts"
"#,
    )
    .unwrap();

    let app = AiDENsApp::builder()
        .name("ignored")
        .profile(AiDENsProfile::CodingAgent)
        .config_file(cfg.to_str().unwrap())
        .build()
        .await
        .expect("app builds");

    let output = app.run_once("hello").await.expect("run succeeds");
    assert_eq!(output.text, "facade mock response");
    assert!(!output.text.to_ascii_lowercase().contains("placeholder"));

    let _ = std::fs::remove_dir_all(&dir);
}
