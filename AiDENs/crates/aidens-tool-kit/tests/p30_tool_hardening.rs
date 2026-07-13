use aidens_contracts::{ArtifactId, CanonicalToolSideEffectClass};
use aidens_permit_kit::{HostPermitAuthorityV1, PermitCheckContextV1, PermitPolicyV1};
use aidens_tool_kit::{ToolDispatcher, ToolExposurePolicyV1, ToolInvocationError, ToolRegistryV1};
use std::collections::BTreeSet;
use std::sync::OnceLock;
use tokio::sync::{Mutex, MutexGuard};

const TEST_RUN_ID: &str = "run:p30-tool-test";
const TEST_ATTEMPT_ID: &str = "attempt:p30-tool-test";

async fn host_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().await
}

fn trusted_policy_for(
    risk: CanonicalToolSideEffectClass,
    tool_id: &str,
    sandbox_root: String,
) -> PermitPolicyV1 {
    let root = std::env::temp_dir().join(format!("aidens-p30-host-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("permit-authority-v1.key"), [31_u8; 32]).unwrap();
    std::env::set_var("AIDENS_HOST_STATE_DIR", root);
    std::env::set_var("AIDENS_HOST_PERMIT_ISSUER", "p30-test-host");
    let authority = HostPermitAuthorityV1::load().unwrap();
    let context = PermitCheckContextV1::new(tool_id, risk, sandbox_root).with_run_attempt(
        Some(ArtifactId(TEST_RUN_ID.into())),
        Some(ArtifactId(TEST_ATTEMPT_ID.into())),
    );
    authority
        .policy()
        .with_grant(authority.issue_for_context(&context, "test"))
}

#[tokio::test]
async fn p30_dispatch_rejects_caller_minted_grant() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p30-untrusted-permit-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let grant = aidens_contracts::PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
        "caller",
    );
    let error = ToolDispatcher::new(registry)
        .with_permit_policy(PermitPolicyV1::default().with_grant(grant))
        .invoke(
            "aidens:patch-apply:1",
            serde_json::json!({"diff":"--- a/x\n+++ b/x\n"}),
        )
        .await
        .expect_err("untrusted caller grant must be blocked at dispatch");
    let error = error.downcast_ref::<ToolInvocationError>().unwrap();
    assert!(error.approval_request().is_some());
    let _ = std::fs::remove_dir_all(&dir);
}

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
    let _host_env = host_env_lock().await;
    let dir = std::env::temp_dir().join(format!(
        "aidens-p30-missing-file-integration-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let policy = trusted_policy_for(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
    );
    let dispatcher = ToolDispatcher::new(registry).with_permit_policy(policy);
    let diff = "--- a/missing.txt\n+++ b/missing.txt\n@@\n-old\n+new\n";

    let error = dispatcher
        .invoke_with_context(
            "aidens:patch-apply:1",
            serde_json::json!({"diff": diff}),
            Some(ArtifactId(TEST_RUN_ID.into())),
            Some(ArtifactId(TEST_ATTEMPT_ID.into())),
        )
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
    let _host_env = host_env_lock().await;
    let dir = std::env::temp_dir().join(format!(
        "aidens-p30-fixed-command-integration-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(dir.join("scripts").join("verify.sh"), "printf fixed-path\n").unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let policy = trusted_policy_for(
        CanonicalToolSideEffectClass::Admin,
        "aidens:run-checks:1",
        dir.canonicalize().unwrap().display().to_string(),
    );
    let dispatcher = ToolDispatcher::new(registry).with_permit_policy(policy);
    let old_path = std::env::var_os("PATH");
    std::env::set_var("PATH", "/tmp/aidens-p30-poisoned-path");
    let result = dispatcher
        .invoke_with_context(
            "aidens:run-checks:1",
            serde_json::json!({"command":["bash","scripts/verify.sh"]}),
            Some(ArtifactId(TEST_RUN_ID.into())),
            Some(ArtifactId(TEST_ATTEMPT_ID.into())),
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
