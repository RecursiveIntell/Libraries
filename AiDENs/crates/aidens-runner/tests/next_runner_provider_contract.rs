use aidens_runner::{AiDENsRunInput, AiDENsRunner};

#[tokio::test]
async fn explicit_mock_provider_returns_configured_text_not_placeholder() {
    let runner = AiDENsRunner::builder()
        .app_id("next-runner")
        .mock_provider("real mock response from provider")
        .build()
        .expect("runner builds");

    let output = runner
        .run(AiDENsRunInput::new("hello"))
        .await
        .expect("mock provider run succeeds");

    assert_eq!(output.text, "real mock response from provider");
    assert!(!output.text.to_ascii_lowercase().contains("placeholder"));
    assert!(!output
        .receipt
        .warnings
        .iter()
        .any(|warning| warning.to_ascii_lowercase().contains("placeholder")));
    assert!(output.receipt.provider_route.is_some());
}

#[tokio::test]
async fn disabled_provider_does_not_produce_answer() {
    let runner = AiDENsRunner::builder()
        .app_id("next-disabled")
        .provider_kind("disabled")
        .build()
        .expect("runner builds");

    let err = runner
        .run(AiDENsRunInput::new("hello"))
        .await
        .expect_err("disabled provider must not answer");

    let err_text = err.to_string().to_ascii_lowercase();
    assert!(err_text.contains("disabled") || err_text.contains("unavailable"));
}
