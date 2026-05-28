use aidens_contracts::{ArtifactId, CanonicalToolSideEffectClass};
use aidens_permit_kit::PermitPolicyV1;
use aidens_tool_kit::{ToolDispatcher, ToolExposurePolicyV1, ToolInvocationError, ToolRegistryV1};
use std::collections::BTreeSet;

#[test]
fn p30_tool_exposure_id_is_content_derived() {
    let dir = std::env::temp_dir();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();

    let read_only = registry.plan_exposure(&ToolExposurePolicyV1::read_only_default());
    let read_only_replay = registry.plan_exposure(&ToolExposurePolicyV1::read_only_default());
    let hidden = registry.plan_exposure(&ToolExposurePolicyV1 {
        allowed_tool_ids: Some(BTreeSet::new()),
        allowed_risk_classes: BTreeSet::from([CanonicalToolSideEffectClass::ReadOnly]),
        max_tools: None,
        native_tool_loop_available: false,
        permit_policy: PermitPolicyV1::default(),
        sandbox_root: None,
    });

    assert_eq!(read_only.exposure_id, read_only_replay.exposure_id);
    assert_ne!(read_only.exposure_id, ArtifactId::new("tool-exposure"));
    assert_ne!(read_only.exposure_id, hidden.exposure_id);
}

#[tokio::test]
async fn p30_patch_apply_missing_file_fails_closed_instead_of_empty_input() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p30-missing-file-integration-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let grant = aidens_contracts::PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let dispatcher = ToolDispatcher::new(registry)
        .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
    let diff = "--- a/missing.txt\n+++ b/missing.txt\n@@\n-old\n+new\n";

    let error = dispatcher
        .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
        .await
        .expect_err("missing file must not be treated as empty input");
    let error = error
        .downcast_ref::<ToolInvocationError>()
        .expect("typed patch failure");

    assert!(!dir.join("missing.txt").exists());
    assert!(error
        .receipt()
        .reason_codes
        .contains(&"patch-target-read-failed-closed".into()));
    let output = error.receipt().output.as_ref().expect("failure output");
    assert_eq!(output["failure_kind"], "read-patch");
    assert_eq!(output["changed_files"][0], "missing.txt");
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn p30_run_checks_uses_fixed_executable_paths_without_ambient_path() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p30-fixed-command-integration-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(dir.join("scripts").join("verify.sh"), "printf fixed-path\n").unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let grant = aidens_contracts::PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Admin,
        "aidens:run-checks:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let dispatcher = ToolDispatcher::new(registry)
        .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
    let old_path = std::env::var_os("PATH");
    std::env::set_var("PATH", "/tmp/aidens-p30-poisoned-path");
    let result = dispatcher
        .invoke(
            "aidens:run-checks:1",
            serde_json::json!({"command":["bash","scripts/verify.sh"]}),
        )
        .await;
    if let Some(old_path) = old_path {
        std::env::set_var("PATH", old_path);
    } else {
        std::env::remove_var("PATH");
    }
    let result = result.expect("fixed bash path executes despite poisoned ambient PATH");

    assert!(result.receipt.succeeded);
    assert_eq!(result.output["stdout"], "fixed-path");
    let _ = std::fs::remove_dir_all(&dir);
}
