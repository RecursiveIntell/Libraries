//! E2E vertical slice: unified agent with memory, governance, and receipts.

use aidens_app_kit::{AiDENsApp, AiDENsProfile};
use aidens_contracts::AiDENsAppPlanV1;

#[tokio::test]
async fn e2e_unified_agent_produces_receipt_chain() {
    // Use from_plan with a mock provider to avoid the disabled default.
    // Use a unique app ID to avoid receipt ID collisions with other parallel tests.
    let app_id = format!("e2e-receipt-{}", std::process::id());
    let plan = AiDENsProfile::CodingAgent
        .expand(&app_id)
        .expect("expand plan");
    let app = AiDENsApp::from_plan(plan)
        .mock_provider("Test response from mock provider")
        .build()
        .await
        .expect("app build");

    let output = app.run_once("What is 2+2?").await.expect("run once");

    assert!(!output.text.is_empty(), "run must produce text output");
    assert!(
        output.receipt.completed_at.is_some(),
        "run must have a completed receipt"
    );
}

#[tokio::test]
async fn e2e_runner_with_governance_context() {
    use aidens_governance_kit::{canonical_stack, GovernanceContext};
    use aidens_runner::AiDENsRunner;

    let policy =
        canonical_stack::PolicySnapshot::permissive("e2e-test-policy", "2026-01-01T00:00:00Z");
    let runner = AiDENsRunner::builder()
        .mock_provider("Governed response")
        .governance(Some(GovernanceContext::new(policy)))
        .build()
        .expect("build runner");

    let output = runner
        .run(aidens_runner::AiDENsRunInput::new("What is governance?"))
        .await
        .expect("execute with governance");

    assert!(!output.text.is_empty(), "run must produce text");
    assert!(
        output.receipt.completed_at.is_some(),
        "run must complete with receipt"
    );
}

#[tokio::test]
async fn e2e_runner_with_kernel_reasoning() {
    use aidens_kernel_kit::CanonicalKernelAdapter;
    use aidens_runner::AiDENsRunner;

    let runner = AiDENsRunner::builder()
        .mock_provider("Reasoned response")
        .kernel(Some(CanonicalKernelAdapter::default()))
        .build()
        .expect("build runner");

    let output = runner
        .run(aidens_runner::AiDENsRunInput::new("What is reasoning?"))
        .await
        .expect("execute with kernel");

    assert!(!output.text.is_empty(), "run must produce text");
    assert!(
        output.receipt.completed_at.is_some(),
        "run must complete with receipt"
    );
}

#[tokio::test]
async fn e2e_runner_with_all_capabilities() {
    use aidens_governance_kit::{canonical_stack, GovernanceContext};
    use aidens_kernel_kit::CanonicalKernelAdapter;
    use aidens_runner::AiDENsRunner;

    let policy =
        canonical_stack::PolicySnapshot::permissive("e2e-full-policy", "2026-01-01T00:00:00Z");
    let runner = AiDENsRunner::builder()
        .mock_provider("Full capability response")
        .governance(Some(GovernanceContext::new(policy)))
        .kernel(Some(CanonicalKernelAdapter::default()))
        .build()
        .expect("build runner");

    let output = runner
        .run(aidens_runner::AiDENsRunInput::new(
            "What is a unified agent framework?",
        ))
        .await
        .expect("execute with all capabilities");

    assert!(!output.text.is_empty(), "run must produce text");
    assert!(
        output.receipt.completed_at.is_some(),
        "run must complete with receipt"
    );
    assert!(
        !output.receipt.turn_receipts.is_empty(),
        "run must have turn receipts"
    );
}

#[tokio::test]
async fn e2e_memory_agent_profile_has_memory_enabled() {
    let defaults = AiDENsProfile::MemoryAgent
        .runtime_defaults("e2e-memory-profile-test")
        .expect("runtime defaults");

    assert!(
        !matches!(
            defaults.memory_mode,
            aidens_contracts::MemoryModeV1::Disabled
        ),
        "MemoryAgent profile must not disable memory"
    );
}

#[tokio::test]
async fn e2e_autonomous_daemon_profile_has_governance_enabled() {
    let defaults = AiDENsProfile::AutonomousDaemon
        .runtime_defaults("e2e-daemon-profile-test")
        .expect("runtime defaults");

    assert!(
        defaults.governance_enabled,
        "AutonomousDaemon profile must enable governance"
    );
}

#[tokio::test]
async fn e2e_research_workbench_profile_has_kernel_enabled() {
    let defaults = AiDENsProfile::ResearchWorkbench
        .runtime_defaults("e2e-research-profile-test")
        .expect("runtime defaults");

    assert!(
        defaults.kernel_reasoning_enabled,
        "ResearchWorkbench profile must enable kernel reasoning"
    );
}
