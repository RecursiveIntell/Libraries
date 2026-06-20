use super::*;
use crate::executors::{capped_utf8_lossy, run_command_with_timeout, MAX_COMMAND_OUTPUT_BYTES};
use crate::registry::canonical_descriptor_from_aidens;
use aidens_contracts::{
    CanonicalToolSideEffectClass, CapabilityGateOutcomeV1, ToolDescriptorV1, ToolExposureSetV1,
    ToolLifecycleStateV1, ToolSchemaV1,
};
use aidens_permit_kit::PermitPolicyV1;
use std::collections::BTreeSet;
use std::path::Path;
use std::time::{Duration, Instant};

fn read_tool() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "test".into(),
        name: "read".into(),
        version: "1".into(),
        description: "read test fixture".into(),
        risk_class: CanonicalToolSideEffectClass::ReadOnly,
        read_only: true,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: serde_json::json!({"type": "object"}),
            parser_fallback_hint: "test read".into(),
        },
    }
}

fn normalized_exposure_json(exposure: &ToolExposureSetV1) -> serde_json::Value {
    let mut lifecycles = serde_json::Map::new();
    for decision in &exposure.decisions {
        lifecycles.insert(
            decision.capability_id.clone(),
            serde_json::Value::Array(
                decision
                    .lifecycle
                    .iter()
                    .map(aidens_testkit::json_string)
                    .map(serde_json::Value::String)
                    .collect(),
            ),
        );
    }
    serde_json::json!({
        "declared_tool_ids": sorted(exposure.declared_tool_ids.clone()),
        "registered_tool_ids": sorted(exposure.registered_tool_ids.clone()),
        "executable_tool_ids": sorted(exposure.executable_tool_ids.clone()),
        "exposed_tool_ids": sorted(exposure.exposed_tool_ids.clone()),
        "hidden_tool_ids": sorted(exposure.hidden_tool_ids.clone()),
        "blocked_tool_ids": sorted(exposure.blocked_tool_ids.clone()),
        "lifecycles": serde_json::Value::Object(lifecycles),
        "reason_codes": sorted(exposure.reason_codes.clone())
    })
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

#[test]
fn disabled_tool_is_absent() {
    let mut registry = ToolRegistryV1::default();
    let tool = read_tool();
    let id = tool.tool_id();
    registry.register_enabled(tool, false);
    assert!(!registry.contains_tool_id(&id));
    assert!(registry.descriptor(&id).is_none());
    assert!(!registry.expose_read_only().exposed_tool_ids.contains(&id));
}

#[tokio::test]
async fn disabled_means_absent_at_registration_exposure_and_invocation() {
    let mut registry = ToolRegistryV1::default();
    registry.register_enabled(
        side_effect_descriptor("shell", CanonicalToolSideEffectClass::Admin),
        false,
    );
    let dispatcher = ToolDispatcher::new(registry);

    let error = dispatcher
        .invoke("aidens:shell:1", serde_json::json!({}))
        .await
        .expect_err("disabled shell is not registered");
    assert!(error.to_string().contains("not registered"));
    let receipt = error
        .downcast_ref::<ToolInvocationError>()
        .expect("typed invocation error")
        .receipt();
    assert!(receipt.reason_codes.contains(&"tool-not-registered".into()));
}

#[test]
fn read_only_default_exposes_only_read_only_executable_tools() {
    let dir = std::env::temp_dir();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(dir).unwrap();
    let exposure = registry.expose_read_only();

    assert!(exposure
        .exposed_tool_ids
        .contains(&"aidens:repo-read:1".into()));
    assert!(exposure
        .exposed_tool_ids
        .contains(&"aidens:patch-propose:1".into()));
    assert!(!exposure
        .exposed_tool_ids
        .contains(&"aidens:patch-apply:1".into()));
    assert!(exposure.exposed_tool_ids.iter().all(|tool_id| {
        let descriptor = registry.descriptor(tool_id).expect("descriptor");
        descriptor.read_only
            && descriptor.risk_class == CanonicalToolSideEffectClass::ReadOnly
            && registry.can_execute(tool_id)
    }));
}

#[test]
fn exposure_policy_can_bound_tool_count() {
    let dir = std::env::temp_dir();
    let mut registry = ToolRegistryV1::default();
    registry
        .register_enabled_with_repo_read_dispatcher(read_tool(), true, &dir)
        .unwrap();
    registry
        .register_enabled_with_repo_read_dispatcher(
            ToolDescriptorV1 {
                namespace: "test".into(),
                name: "read2".into(),
                version: "1".into(),
                description: "second read test fixture".into(),
                risk_class: CanonicalToolSideEffectClass::ReadOnly,
                read_only: true,
                hidden: false,
                requires_native_tool_loop: false,
                schema: ToolSchemaV1 {
                    input_schema: serde_json::json!({"type": "object"}),
                    output_schema: serde_json::json!({"type": "object"}),
                    parser_fallback_hint: "test read 2".into(),
                },
            },
            true,
            &dir,
        )
        .unwrap();
    let exposure = registry.plan_exposure(&ToolExposurePolicyV1 {
        allowed_tool_ids: None,
        allowed_risk_classes: BTreeSet::from([CanonicalToolSideEffectClass::ReadOnly]),
        max_tools: Some(1),
        native_tool_loop_available: false,
        permit_policy: PermitPolicyV1::default(),
        sandbox_root: None,
    });
    assert_eq!(exposure.exposed_tool_ids.len(), 1);
    assert_eq!(exposure.hidden_tool_ids.len(), 1);
    assert!(exposure.reason_codes.contains(&"max-tools-reached".into()));
}

#[test]
fn safe_coding_registry_does_not_register_dangerous_tools() {
    let registry = safe_coding_registry();

    assert!(registry.contains_tool_id("aidens:repo-read:1"));
    assert!(registry.contains_tool_id("aidens:patch-propose:1"));
    assert!(registry.contains_tool_id("aidens:patch-apply:1"));
    assert!(registry.contains_tool_id("aidens:run-checks:1"));
    assert!(!registry.contains_tool_id("aidens:file-write:1"));
    assert!(!registry.contains_tool_id("aidens:shell:1"));
    assert!(!registry.contains_tool_id("aidens:network:1"));
    assert!(!registry.contains_tool_id("aidens:memory-write:1"));
    assert!(!registry.contains_tool_id("aidens:schedule:1"));
}

#[test]
fn patch_propose_is_executable_and_non_mutating() {
    let registry = safe_coding_registry_for_current_dir();
    let exposure = registry.expose_read_only();

    assert!(registry.contains_tool_id("aidens:patch-propose:1"));
    assert!(registry.can_execute("aidens:patch-propose:1"));
    assert!(exposure
        .exposed_tool_ids
        .contains(&"aidens:patch-propose:1".into()));
}

#[test]
fn side_effect_tool_requires_permit() {
    let mut registry = ToolRegistryV1::default();
    registry.register_enabled(
        side_effect_descriptor("file-write", CanonicalToolSideEffectClass::Write),
        true,
    );
    let exposure = registry.plan_exposure(&ToolExposurePolicyV1 {
        allowed_tool_ids: None,
        allowed_risk_classes: BTreeSet::from([CanonicalToolSideEffectClass::Write]),
        max_tools: Some(16),
        native_tool_loop_available: false,
        permit_policy: PermitPolicyV1::default(),
        sandbox_root: Some(".".into()),
    });

    assert!(exposure
        .blocked_tool_ids
        .contains(&"aidens:file-write:1".into()));
    assert!(exposure
        .reason_codes
        .contains(&"permit-required:write".into()));
    assert_eq!(exposure.approval_requests.len(), 1);
    let decision = exposure
        .decisions
        .iter()
        .find(|decision| decision.capability_id == "aidens:file-write:1")
        .expect("file-write decision");
    assert_eq!(decision.outcome, CapabilityGateOutcomeV1::Blocked);
    assert!(decision.lifecycle.contains(&ToolLifecycleStateV1::Blocked));
    assert!(decision.approval_request.is_some());
}

#[test]
fn side_effect_tool_without_sandbox_root_fails_closed_without_wildcard_scope() {
    let mut registry = ToolRegistryV1::default();
    registry.register_enabled(
        side_effect_descriptor("file-write", CanonicalToolSideEffectClass::Write),
        true,
    );

    let exposure = registry.plan_exposure(&ToolExposurePolicyV1 {
        allowed_tool_ids: None,
        allowed_risk_classes: BTreeSet::from([CanonicalToolSideEffectClass::Write]),
        max_tools: Some(16),
        native_tool_loop_available: false,
        permit_policy: PermitPolicyV1::default(),
        sandbox_root: None,
    });

    assert!(exposure
        .blocked_tool_ids
        .contains(&"aidens:file-write:1".into()));
    assert!(exposure
        .reason_codes
        .contains(&"permit-scope-missing-sandbox-root".into()));
    assert!(exposure.approval_requests.is_empty());
    let decision = exposure
        .decisions
        .iter()
        .find(|decision| decision.capability_id == "aidens:file-write:1")
        .expect("file-write decision");
    assert_eq!(decision.outcome, CapabilityGateOutcomeV1::Blocked);
    assert!(decision.approval_request.is_none());
    assert_eq!(decision.sandbox_root, None);
}

#[test]
fn declared_registered_executable_hidden_and_blocked_are_distinct() {
    let registry = safe_coding_registry_for_current_dir();
    let declarations = safe_coding_tool_declarations();
    let exposure = registry.plan_exposure_with_declarations(
        &ToolExposurePolicyV1::read_only_default(),
        declarations.clone(),
    );

    assert!(exposure
        .declared_tool_ids
        .contains(&"aidens:file-write:1".into()));
    assert!(exposure
        .registered_tool_ids
        .contains(&"aidens:repo-read:1".into()));
    assert!(exposure
        .executable_tool_ids
        .contains(&"aidens:repo-read:1".into()));
    assert!(exposure
        .exposed_tool_ids
        .contains(&"aidens:repo-read:1".into()));
    assert!(exposure
        .exposed_tool_ids
        .contains(&"aidens:patch-propose:1".into()));
    assert!(exposure
        .blocked_tool_ids
        .contains(&"aidens:patch-apply:1".into()));
    assert!(registry
        .declared_not_registered_tool_ids(&declarations)
        .contains(&"aidens:file-write:1".into()));
}

#[test]
fn safe_coding_exposure_matches_reference_interpreter() {
    let registry = safe_coding_registry_for_current_dir();
    let declarations = safe_coding_tool_declarations();
    let exposure = registry
        .plan_exposure_with_declarations(&ToolExposurePolicyV1::read_only_default(), declarations);
    let case = aidens_testkit::reference_safe_coding_exposure_case();
    let report = aidens_testkit::compare_case_to_actual(
        &case,
        "aidens-tool-kit::plan_exposure_with_declarations",
        normalized_exposure_json(&exposure),
    );

    assert!(
        report.passed,
        "{}",
        report
            .findings
            .iter()
            .map(|finding| finding.human_diff.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[tokio::test]
async fn side_effect_invocation_without_permit_is_receipt_bearing_denial() {
    let mut registry = ToolRegistryV1::default();
    registry.register_enabled(
        side_effect_descriptor("file-write", CanonicalToolSideEffectClass::Write),
        true,
    );
    let dispatcher = ToolDispatcher::new(registry);

    let error = dispatcher
        .invoke("aidens:file-write:1", serde_json::json!({"path":"x"}))
        .await
        .expect_err("side-effect invocation requires permit");
    let invocation_error = error
        .downcast_ref::<ToolInvocationError>()
        .expect("typed invocation error");

    assert!(invocation_error
        .receipt()
        .reason_codes
        .contains(&"permit-required:write".into()));
    assert!(invocation_error.receipt().approval_request_id.is_some());
    assert!(invocation_error.approval_request().is_some());
}

#[test]
fn exposure_receipt_matches_provider_tool_schema() {
    let dir = std::env::temp_dir();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let exposure = registry.expose_read_only();
    let schema_ids = exposure
        .provider_tool_schemas
        .iter()
        .map(|schema| schema.tool_id.clone())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        exposure
            .exposed_tool_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        schema_ids
    );
    assert!(exposure
        .provider_tool_schemas
        .iter()
        .all(|schema| schema.input_schema["type"] == "object"));
}

#[test]
fn side_effect_descriptors_require_durable_runtime_receipts() {
    let read_descriptor = canonical_descriptor_from_aidens(&repo_read_descriptor());
    assert_eq!(
        read_descriptor.receipt_persistence,
        canonical_stack::ToolReceiptPersistence::ForgeRaw
    );

    let patch_apply_descriptor = canonical_descriptor_from_aidens(&patch_apply_descriptor());
    assert_eq!(
        patch_apply_descriptor.receipt_persistence,
        canonical_stack::ToolReceiptPersistence::ForgeRaw
    );
}

#[test]
fn safe_coding_registry_for_current_dir_fallback_is_degraded() {
    let missing_root = std::env::temp_dir().join(format!(
        "aidens-tool-kit-missing-root-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&missing_root);
    let registry = safe_coding_registry_for_sandbox_root(&missing_root);
    let exposure = registry.plan_exposure(&ToolExposurePolicyV1::read_only_default());

    assert!(
        exposure.degraded,
        "fallback path should emit a degraded tool registry exposure"
    );
    assert!(
        exposure
            .reason_codes
            .iter()
            .any(|reason| reason.contains("safe-coding dispatcher construction failed")),
        "fallback reasons should be surfaced through exposure reason codes"
    );
    assert!(
        !registry.tool_ids().is_empty(),
        "safe-registry fallback should still declare safe tools"
    );
    assert!(
        !registry.can_execute("aidens:repo-read:1"),
        "fallback tools should remain non-executable when dispatchers fail"
    );
}

#[tokio::test]
async fn repo_read_rejects_traversal() {
    let dir = std::env::temp_dir().join(format!("aidens-tool-kit-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(dir.join(".ssh")).unwrap();
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    std::fs::create_dir_all(dir.join(".aws")).unwrap();
    std::fs::write(dir.join("README.md"), "hello").unwrap();
    std::fs::write(dir.join(".ssh").join("id_rsa"), "secret").unwrap();
    std::fs::write(dir.join(".git").join("config"), "secret").unwrap();
    std::fs::write(dir.join(".env"), "TOKEN=secret").unwrap();
    std::fs::write(dir.join(".npmrc"), "//registry/:_authToken=secret").unwrap();
    std::fs::write(dir.join(".aws").join("credentials"), "secret").unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let dispatcher = ToolDispatcher::new(registry);

    let output = dispatcher
        .invoke(
            "aidens:repo-read:1",
            serde_json::json!({ "path": "README.md" }),
        )
        .await
        .unwrap();
    assert!(output.output_text().contains("hello"));

    let error = dispatcher
        .invoke(
            "aidens:repo-read:1",
            serde_json::json!({ "path": "../secret" }),
        )
        .await
        .unwrap_err();
    assert!(error.to_string().contains("traversal"));
    let invocation_error = error
        .downcast_ref::<ToolInvocationError>()
        .expect("typed traversal receipt");
    assert!(invocation_error
        .receipt()
        .reason_codes
        .iter()
        .any(|reason| { reason.contains("sandbox") || reason.contains("traversal") }));
    let sensitive = dispatcher
        .invoke(
            "aidens:repo-read:1",
            serde_json::json!({ "path": ".ssh/id_rsa" }),
        )
        .await
        .unwrap_err();
    let sensitive = sensitive
        .downcast_ref::<ToolInvocationError>()
        .expect("typed sensitive-prefix receipt");
    assert!(sensitive
        .receipt()
        .reason_codes
        .contains(&"sandbox-sensitive-prefix-denied".into()));
    for denied_path in [".git/config", ".env", ".npmrc", ".aws/credentials"] {
        let error = dispatcher
            .invoke(
                "aidens:repo-read:1",
                serde_json::json!({ "path": denied_path }),
            )
            .await
            .unwrap_err();
        let invocation_error = error
            .downcast_ref::<ToolInvocationError>()
            .expect("typed sensitive fixture receipt");
        assert!(
            invocation_error
                .receipt()
                .reason_codes
                .iter()
                .any(|reason| reason == "sandbox-sensitive-prefix-denied"
                    || reason == "sandbox-hidden-component-denied"),
            "missing sensitive denial for {denied_path}: {:?}",
            invocation_error.receipt().reason_codes
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = dir
            .parent()
            .unwrap_or_else(|| Path::new("/tmp"))
            .join(format!("aidens-outside-secret-{}", std::process::id()));
        std::fs::write(&outside, "outside secret").unwrap();
        symlink(&outside, dir.join("outside-link")).unwrap();
        std::fs::hard_link(&outside, dir.join("outside-hardlink")).unwrap();

        let symlink_error = dispatcher
            .invoke(
                "aidens:repo-read:1",
                serde_json::json!({ "path": "outside-link" }),
            )
            .await
            .unwrap_err();
        let symlink_text = symlink_error.to_string();
        assert!(symlink_text.contains("sandbox"));
        assert!(
            !symlink_text.contains(outside.parent().unwrap().to_string_lossy().as_ref()),
            "symlink denial leaked host path: {symlink_text}"
        );

        let hardlink_error = dispatcher
            .invoke(
                "aidens:repo-read:1",
                serde_json::json!({ "path": "outside-hardlink" }),
            )
            .await
            .unwrap_err();
        let hardlink_receipt = hardlink_error
            .downcast_ref::<ToolInvocationError>()
            .expect("typed hardlink receipt");
        assert!(hardlink_receipt
            .receipt()
            .reason_codes
            .contains(&"sandbox-hardlink-denied".into()));
        let _ = std::fs::remove_file(outside);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn p10_read_inspect_search_and_patch_propose_are_read_only() {
    let dir = std::env::temp_dir().join(format!("aidens-p10-read-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("README.md"), "hello p10\n").unwrap();
    std::fs::write(dir.join("src").join("lib.rs"), "pub fn p10() {}\n").unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let dispatcher = ToolDispatcher::new(registry);

    let list = dispatcher
        .invoke("aidens:repo-list:1", serde_json::json!({"path":"."}))
        .await
        .unwrap();
    assert!(list.output["entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["path"] == "README.md" }));

    let stat = dispatcher
        .invoke(
            "aidens:file-stat:1",
            serde_json::json!({"path":"README.md"}),
        )
        .await
        .unwrap();
    assert_eq!(stat.output["is_file"], true);

    let search = dispatcher
        .invoke(
            "aidens:repo-search:1",
            serde_json::json!({"query":"p10","path":"."}),
        )
        .await
        .unwrap();
    assert!(!search.output["matches"].as_array().unwrap().is_empty());

    let before = std::fs::read_to_string(dir.join("README.md")).unwrap();
    let proposal = dispatcher
        .invoke(
            "aidens:patch-propose:1",
            serde_json::json!({
                "summary": "extend readme",
                "diff": "--- a/README.md\n+++ b/README.md\n@@\n-hello p10\n+hello p10 patched\n"
            }),
        )
        .await
        .unwrap();
    let after = std::fs::read_to_string(dir.join("README.md")).unwrap();
    assert_eq!(before, after);
    assert_eq!(proposal.output["mutates_files"], false);
    assert_eq!(proposal.output["proposal"]["mutates_files"], false);
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn p10_patch_apply_requires_permit_and_then_writes_receipt() {
    let dir = std::env::temp_dir().join(format!("aidens-p10-apply-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "hello p10\n").unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let dispatcher = ToolDispatcher::new(registry.clone());
    let diff = "--- a/README.md\n+++ b/README.md\n@@\n-hello p10\n+hello p10 patched\n";

    let denied = dispatcher
        .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
        .await
        .expect_err("patch apply requires file-write permit");
    let denied = denied
        .downcast_ref::<ToolInvocationError>()
        .expect("typed denial");
    assert!(denied.approval_request().is_some());
    assert!(denied.receipt().approval_request_id.is_some());

    let grant = aidens_contracts::PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let permitted = ToolDispatcher::new(registry)
        .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
    let applied = permitted
        .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
        .await
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(dir.join("README.md")).unwrap(),
        "hello p10 patched\n"
    );
    assert!(applied.receipt.succeeded);
    assert!(applied.receipt.permit_use_receipt_id.is_some());
    assert_eq!(applied.output["receipt"]["applied"], true);
    assert_eq!(applied.output["dry_run_checked"], true);
    assert_eq!(applied.output["semantic_status"], "exact_check");
    assert_eq!(applied.output["changed_files"][0], "README.md");
    assert_eq!(applied.output["receipt"]["changed_files"][0], "README.md");
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn p10_patch_apply_check_only_validates_without_mutation() {
    let dir = std::env::temp_dir().join(format!("aidens-p10-check-only-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "hello p10\n").unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let grant = aidens_contracts::PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let dispatcher = ToolDispatcher::new(registry)
        .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
    let diff = "--- a/README.md\n+++ b/README.md\n@@\n-hello p10\n+hello p10 checked\n";

    let checked = dispatcher
        .invoke(
            "aidens:patch-apply:1",
            serde_json::json!({"diff": diff, "check_only": true}),
        )
        .await
        .unwrap();

    assert_eq!(checked.output["applied"], false);
    assert_eq!(checked.output["dry_run_checked"], true);
    assert_eq!(
        checked.output["receipt"]["reason_codes"][0],
        "patch-validated-without-application"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("README.md")).unwrap(),
        "hello p10\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn phase04_patch_apply_multifile_records_before_after_digests() {
    let dir = std::env::temp_dir().join(format!("aidens-phase04-multifile-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.txt"), "alpha\n").unwrap();
    std::fs::write(dir.join("b.txt"), "beta\n").unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let grant = aidens_contracts::PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let dispatcher = ToolDispatcher::new(registry)
        .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
    let diff = "--- a/a.txt\n+++ b/a.txt\n@@\n-alpha\n+alpha patched\n--- a/b.txt\n+++ b/b.txt\n@@\n-beta\n+beta patched\n";

    let applied = dispatcher
        .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
        .await
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "alpha patched\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("b.txt")).unwrap(),
        "beta patched\n"
    );
    assert_eq!(applied.output["changed_files"].as_array().unwrap().len(), 2);
    assert!(!applied.output["receipt"]["before_digests"]["a.txt"].is_null());
    assert!(!applied.output["receipt"]["after_digests"]["b.txt"].is_null());
    assert_eq!(applied.output["semantic_status"], "exact_check");
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn p10_patch_apply_ambiguous_diff_fails_closed_with_receipt() {
    let dir = std::env::temp_dir().join(format!("aidens-p10-ambiguous-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "repeat\nrepeat\n").unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let grant = aidens_contracts::PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let dispatcher = ToolDispatcher::new(registry)
        .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
    let diff = "--- a/README.md\n+++ b/README.md\n@@\n-repeat\n+patched\n";

    let denied = dispatcher
        .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
        .await
        .expect_err("ambiguous diff must fail closed");
    let denied = denied
        .downcast_ref::<ToolInvocationError>()
        .expect("typed patch failure");

    assert!(!denied.receipt().succeeded);
    assert!(denied
        .receipt()
        .reason_codes
        .contains(&"patch-ambiguous-failed-closed".into()));
    let output = denied.receipt().output.as_ref().expect("failure output");
    assert_eq!(output["applied"], false);
    assert_eq!(output["dry_run_checked"], true);
    assert_eq!(output["semantic_status"], "failed_exact_check");
    assert_eq!(output["failure_kind"], "ambiguous-patch");
    assert_eq!(output["receipt"]["changed_files"][0], "README.md");
    assert_eq!(
        std::fs::read_to_string(dir.join("README.md")).unwrap(),
        "repeat\nrepeat\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn p28_patch_apply_missing_parent_leaves_no_dirty_directory() {
    let dir =
        std::env::temp_dir().join(format!("aidens-p28-missing-parent-{}", std::process::id()));
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
    let diff = "--- a/missing/child.txt\n+++ b/missing/child.txt\n@@\n-old\n+new\n";

    let error = dispatcher
        .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
        .await
        .expect_err("missing parent must fail before filesystem mutation");

    assert!(error.to_string().contains("parent cannot be resolved"));
    assert!(!dir.join("missing").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[tokio::test]
async fn p28_patch_apply_rejects_symlink_write_targets() {
    let dir = std::env::temp_dir().join(format!("aidens-p28-symlink-{}", std::process::id()));
    let outside =
        std::env::temp_dir().join(format!("aidens-p28-symlink-outside-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(&outside);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&outside, "secret\n").unwrap();
    std::os::unix::fs::symlink(&outside, dir.join("link.txt")).unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let grant = aidens_contracts::PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let dispatcher = ToolDispatcher::new(registry)
        .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
    let diff = "--- a/link.txt\n+++ b/link.txt\n@@\n-secret\n+patched\n";

    let error = dispatcher
        .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
        .await
        .expect_err("symlink write target must be rejected");

    assert!(error.to_string().contains("symlink write target rejected"));
    assert_eq!(std::fs::read_to_string(&outside).unwrap(), "secret\n");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(&outside);
}

#[cfg(unix)]
#[tokio::test]
async fn p28_repo_list_reports_symlinks_without_following_targets() {
    let dir = std::env::temp_dir().join(format!("aidens-p28-list-symlink-{}", std::process::id()));
    let outside =
        std::env::temp_dir().join(format!("aidens-p28-list-outside-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&outside);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("secret.txt"), "secret\n").unwrap();
    std::os::unix::fs::symlink(&outside, dir.join("outside-link")).unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let dispatcher = ToolDispatcher::new(registry);

    let list = dispatcher
        .invoke("aidens:repo-list:1", serde_json::json!({"path":"."}))
        .await
        .unwrap();
    let link = list.output["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == "outside-link")
        .expect("symlink entry");
    assert_eq!(link["entry_kind"], "symlink");
    assert!(link.get("bytes").map_or(true, serde_json::Value::is_null));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&outside);
}

#[tokio::test]
async fn p28_repo_list_truncation_discloses_total_and_full_digest() {
    let dir = std::env::temp_dir().join(format!("aidens-p28-list-trunc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for index in 0..3 {
        std::fs::write(dir.join(format!("file-{index}.txt")), format!("{index}\n")).unwrap();
    }
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let dispatcher = ToolDispatcher::new(registry);

    let list = dispatcher
        .invoke(
            "aidens:repo-list:1",
            serde_json::json!({"path":".","max_entries":1}),
        )
        .await
        .unwrap();

    assert_eq!(list.output["returned_entries"], 1);
    assert_eq!(list.output["total_entries"], 3);
    assert_eq!(list.output["truncated"], true);
    assert!(list.output["full_listing_digest"].is_object());
    assert_eq!(list.output["receipt"]["truncated"], true);
    assert_eq!(list.output["receipt"]["total_entries"], 3);
    assert!(list.output["receipt"]["reason_codes"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("repo-list-truncated-with-full-digest")));
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn p28_file_stat_fails_on_unreadable_digest_input() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p28-file-stat-binary-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("binary.dat"), [0xff, 0xfe, 0xfd]).unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let dispatcher = ToolDispatcher::new(registry);

    let error = dispatcher
        .invoke(
            "aidens:file-stat:1",
            serde_json::json!({"path":"binary.dat"}),
        )
        .await
        .expect_err("invalid utf8 digest input must fail");
    assert!(error.to_string().contains("file-stat cannot read"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn p28_repo_search_skips_denied_components_anywhere_in_path() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p28-search-components-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("nested").join("target")).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("nested").join("target").join("secret.txt"),
        "needle\n",
    )
    .unwrap();
    std::fs::write(dir.join("src").join("lib.rs"), "needle\n").unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let dispatcher = ToolDispatcher::new(registry);

    let search = dispatcher
        .invoke(
            "aidens:repo-search:1",
            serde_json::json!({"query":"needle","path":"."}),
        )
        .await
        .unwrap();
    let paths = search.output["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert!(paths.iter().any(|path| path == "src/lib.rs"));
    assert!(paths.iter().all(|path| !path.contains("/target/")));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn p28_run_command_timeout_marks_partial_execution() {
    let dir = std::env::temp_dir();
    let output = run_command_with_timeout(
        &dir,
        &["bash".into(), "-c".into(), "sleep 2".into()],
        Duration::from_millis(50),
    )
    .unwrap();
    assert!(output.timed_out);
}

#[test]
fn p28_command_timeout_wait_uses_adaptive_backoff() {
    let dir = std::env::temp_dir();
    let started = Instant::now();
    let output = run_command_with_timeout(
        &dir,
        &["bash".into(), "-c".into(), "sleep 1".into()],
        Duration::from_millis(20),
    )
    .unwrap();
    assert!(output.timed_out);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[cfg(unix)]
#[test]
fn p30_command_timeout_terminates_process_group() {
    let dir = std::env::temp_dir();
    let started = Instant::now();
    let output = run_command_with_timeout(
        &dir,
        &["bash".into(), "-c".into(), "sleep 2 & wait".into()],
        Duration::from_millis(20),
    )
    .unwrap();
    assert!(output.timed_out);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn p10_run_checks_requires_shell_permit_and_blocks_unallowed_commands() {
    let dir = std::env::temp_dir().join(format!("aidens-p10-checks-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let dispatcher = ToolDispatcher::new(registry.clone());

    let denied = dispatcher
        .invoke(
            "aidens:run-checks:1",
            serde_json::json!({"command":["cargo","check","--workspace"]}),
        )
        .await
        .expect_err("run-checks requires shell permit");
    let denied = denied
        .downcast_ref::<ToolInvocationError>()
        .expect("typed denial");
    assert!(denied.approval_request().is_some());

    let grant = aidens_contracts::PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Admin,
        "aidens:run-checks:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let permitted = ToolDispatcher::new(registry)
        .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
    let blocked = permitted
        .invoke(
            "aidens:run-checks:1",
            serde_json::json!({"command":["rm","-rf","."]}),
        )
        .await
        .unwrap();

    assert_eq!(blocked.output["succeeded"], false);
    assert_eq!(
        blocked.output["receipt"]["reason_codes"][0],
        "command-not-allowed-by-policy"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn phase05_run_checks_rejects_string_commands_and_caps_output() {
    let dir = std::env::temp_dir().join(format!("aidens-phase05-checks-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let grant = aidens_contracts::PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Admin,
        "aidens:run-checks:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let dispatcher = ToolDispatcher::new(registry)
        .with_permit_policy(PermitPolicyV1::default().with_grant(grant));

    let string_command = dispatcher
        .invoke(
            "aidens:run-checks:1",
            serde_json::json!({"command":"bash scripts/verify.sh"}),
        )
        .await
        .expect_err("string command must not be parsed");
    let string_command = string_command
        .downcast_ref::<ToolInvocationError>()
        .expect("typed schema rejection");
    assert!(string_command
        .receipt()
        .reason_codes
        .contains(&"schema-validation-failed".into()));

    let oversized = vec![b'x'; MAX_COMMAND_OUTPUT_BYTES + 8];
    assert_eq!(
        capped_utf8_lossy(&oversized, MAX_COMMAND_OUTPUT_BYTES).len(),
        MAX_COMMAND_OUTPUT_BYTES
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn schema_invalid_tool_input_is_blocked_before_executor() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-tool-kit-schema-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "hello").unwrap();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let dispatcher = ToolDispatcher::new(registry);

    let error = dispatcher
        .invoke("aidens:repo-read:1", serde_json::json!({ "path": 7 }))
        .await
        .expect_err("invalid tool input should be schema-blocked");
    let invocation_error = error
        .downcast_ref::<ToolInvocationError>()
        .expect("typed invocation error");
    let schema_validation = invocation_error
        .schema_validation_receipt()
        .expect("schema validation receipt");

    assert!(!schema_validation.valid);
    assert_eq!(
        schema_validation.tool_id.as_deref(),
        Some("aidens:repo-read:1")
    );
    assert!(invocation_error
        .receipt()
        .reason_codes
        .contains(&"schema-validation-failed".into()));
    assert_eq!(
        invocation_error.receipt().schema_validation_receipt_ids,
        vec![schema_validation.receipt_id.clone()]
    );
    assert!(!error.to_string().contains("requires string field"));
    let _ = std::fs::remove_dir_all(&dir);
}
