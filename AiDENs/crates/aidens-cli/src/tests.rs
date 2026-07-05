use super::*;
use crate::package::P24_REQUIRED_GATE_COMMANDS;
use aidens_contracts::{MemoryModeV1, ReportLevelV1};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "aidens-cli-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

#[test]
fn doctor_without_config_reports_disabled_provider() {
    let report = doctor(Some("definitely-missing-aidens.toml".into())).unwrap();

    assert!(report.contains("provider:disabled"));
    assert!(report.contains("receipts:canonical-log"));
}

#[test]
fn doctor_reports_scaffold_crates_as_deferred_not_healthy() {
    let report = doctor(Some("definitely-missing-aidens.toml".into())).unwrap();
    let report: AiDENsDoctorReportV1 = serde_json::from_str(&report).unwrap();
    let scaffold = report
        .sections
        .get("scaffold_surfaces")
        .expect("scaffold surface section");

    assert_eq!(scaffold.len(), SCAFFOLD_ONLY_CRATES.len());
    assert!(!scaffold
        .iter()
        .any(|truth| truth.capability_id == "crate:aidens-delegation-kit"));
    assert!(!scaffold
        .iter()
        .any(|truth| truth.capability_id == "crate:aidens-governance-kit"));
    assert!(!scaffold
        .iter()
        .any(|truth| truth.capability_id == "crate:aidens-repair-kit"));
    assert!(!scaffold
        .iter()
        .any(|truth| truth.capability_id == "crate:aidens-daemon-kit"));
    assert!(!scaffold
        .iter()
        .any(|truth| truth.capability_id == "crate:aidens-queue-kit"));
    assert!(!scaffold
        .iter()
        .any(|truth| truth.capability_id == "crate:aidens-memory-kit"));
    assert!(report.sections["daemon"][0]
        .states
        .contains(&CapabilityStateV1::Available));
    assert!(report.sections["queue"][0]
        .states
        .contains(&CapabilityStateV1::Available));
    assert!(report.sections["schedule"][0]
        .states
        .contains(&CapabilityStateV1::Available));
    assert!(report.sections["wake"][0]
        .states
        .contains(&CapabilityStateV1::Available));
    assert!(report.sections["governance"][0]
        .states
        .contains(&CapabilityStateV1::Available));
    assert!(report.sections["repair"][0]
        .states
        .contains(&CapabilityStateV1::Available));
    for surface in scaffold {
        assert!(surface.capability_id.starts_with("crate:"));
        assert!(surface.states.contains(&CapabilityStateV1::Deferred));
        assert!(surface.states.contains(&CapabilityStateV1::Disabled));
        assert!(!surface.states.contains(&CapabilityStateV1::Healthy));
    }
}

#[test]
fn profile_commands_report_truthful_surface_statuses() {
    let list = profile_list().unwrap();

    assert!(list.contains("chat-only\tsupported"));
    assert!(list.contains("coding-agent\tsupported"));
    assert!(list.contains("memory-agent\tpartial/proof-only"));
    assert!(list.contains("autonomous-daemon\tpartial/safe-mode"));
    assert!(list.contains("research-workbench\tdeferred/example-only"));

    let research = profile_explain("research-workbench").unwrap();
    assert!(research.contains("Status: deferred/example-only"));
    assert!(research.contains("not a complete product surface"));
}

#[test]
fn plan_commands_accept_test_agent_sources_without_fake_runner_path() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let out = root.join("basic-agent.plan.json");
    let config = workspace_root()
        .join("fixtures")
        .join("test-agent")
        .join("basic-agent.toml");

    let validate = plan_validate(&config.display().to_string()).unwrap();
    assert!(validate.contains("valid: aidens-basic-test-agent"));
    assert!(validate.contains("source=loaded test-agent"));

    let compile = plan_compile(&config.display().to_string(), &out.display().to_string()).unwrap();
    assert!(compile.contains("compiled: aidens-basic-test-agent"));
    let compiled: AiDENsCompiledPlanV1 =
        serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(compiled.plan.profile_id, "coding-agent");
    assert_eq!(compiled.provider_route.route_label, "mock");
    assert!(compiled.parity_report.is_passing());
    assert!(compiled
        .config_apply_receipt
        .reason_codes
        .contains(&"plan-kit:execution-plan-assembly-only".into()));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn plan_validate_rejects_unknown_profile_without_fallback() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let config = root.join("aidens.toml");
    std::fs::write(
        &config,
        r#"
app_id = "unknown-profile-agent"
profile_id = "mystery-agent"
memory_mode = "disabled"
receipt_level = "standard"

[provider]
kind = "mock"
mock_response = "ok"
"#,
    )
    .unwrap();

    let error = plan_validate(&config.display().to_string()).unwrap_err();

    assert!(error.to_string().contains("unknown AiDENs profile"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn doctor_reports_provider_capability_matrix_without_cloud_or_native_overclaims() {
    let report = doctor(Some("definitely-missing-aidens.toml".into())).unwrap();
    let raw_report: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert_eq!(
        raw_report["semantic_disclosure"]["semantic_status"],
        "display_only"
    );
    assert_eq!(
        raw_report["semantic_disclosure"]["support_tier"],
        "mixed-operator-report"
    );
    assert!(raw_report["semantic_disclosure"]["proof_checks"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("support-tier-buckets-emitted")));
    let support_tiers = &raw_report["operator_support_tiers"];
    assert!(support_tiers["scaffold"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("crate:aidens-profile-research")));
    assert!(support_tiers["deferred"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("provider-matrix:openai")));
    assert!(support_tiers["supported"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("provider-matrix:mock")));

    let report: AiDENsDoctorReportV1 = serde_json::from_value(raw_report).unwrap();
    let matrix = report
        .sections
        .get("provider_capability_matrix")
        .expect("provider capability matrix section");

    let find = |provider: &str| {
        matrix
            .iter()
            .find(|truth| truth.capability_id == format!("provider-matrix:{provider}"))
            .unwrap_or_else(|| panic!("missing provider matrix row for {provider}"))
    };

    let mock = find("mock");
    assert!(mock.states.contains(&CapabilityStateV1::ExecutableThisTurn));
    assert!(mock
        .reason
        .as_deref()
        .unwrap()
        .contains("support_label=fixture-supported-not-cloud"));
    assert!(mock
        .reason
        .as_deref()
        .unwrap()
        .contains("native_tool_loop_executable=false"));

    let ollama = find("ollama");
    assert!(ollama
        .reason
        .as_deref()
        .unwrap()
        .contains("support_label=partial-local-chat"));
    assert!(ollama
        .reason
        .as_deref()
        .unwrap()
        .contains("ollama-local-service-required"));
    assert!(!ollama.states.contains(&CapabilityStateV1::Healthy));

    for provider in ["compatible", "openai", "openrouter", "anthropic"] {
        let row = find(provider);
        assert!(row.states.contains(&CapabilityStateV1::Unavailable));
        assert!(row.states.contains(&CapabilityStateV1::Deferred));
        assert!(!row.states.contains(&CapabilityStateV1::Healthy));
        assert!(!row.states.contains(&CapabilityStateV1::ExecutableThisTurn));
        let reason = row.reason.as_deref().unwrap();
        assert!(reason.contains("support_label=deferred/unavailable"));
        assert!(reason.contains("chat_completion_executable=false"));
        assert!(reason.contains("native_tool_loop_executable=false"));
        assert!(reason.contains("streaming_executable=false"));
        assert!(reason.contains("structured_output_executable=false"));
    }

    let openai_compatible = find("openai-compatible");
    assert!(!openai_compatible
        .states
        .contains(&CapabilityStateV1::Healthy));
    let reason = openai_compatible.reason.as_deref().unwrap();
    assert!(reason.contains("chat_completion_executable=false"));
    assert!(reason.contains("native_tool_loop_executable=false"));
}

#[test]
fn provider_check_reports_missing_api_key_without_claiming_executable() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("aidens.toml");
    std::fs::write(
        &path,
        r#"
app_id = "agent"
memory_mode = "disabled"
receipt_level = "standard"

[provider]
kind = "openai"
model = "gpt-test"
"#,
    )
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&provider_check(Some(path.display().to_string())).unwrap()).unwrap();

    assert_eq!(report["provider"], "openai");
    assert_eq!(report["configured"], true);
    assert_eq!(report["executable"], false);
    assert_eq!(report["native_tool_loop"], false);
    assert_eq!(report["structured_output"], false);
    assert_eq!(report["support_label"], "deferred/unavailable");
    assert_eq!(report["support_tier"], "deferred");
    assert!(report["reason_codes"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("api-key-missing")));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn provider_check_reports_configured_cloud_providers_as_unavailable() {
    for provider in ["openai", "openrouter", "anthropic", "openai-compatible"] {
        let root = temp_root();
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("aidens.toml");
        std::fs::write(
            &path,
            format!(
                r#"
app_id = "{provider}-agent"
memory_mode = "disabled"
receipt_level = "standard"

[provider]
kind = "{provider}"
model = "test-model"
api_key = "configured"
"#
            ),
        )
        .unwrap();

        let report: serde_json::Value =
            serde_json::from_str(&provider_check(Some(path.display().to_string())).unwrap())
                .unwrap();

        assert_eq!(report["provider"], provider);
        assert_eq!(report["configured"], true, "{provider}");
        assert_eq!(report["executable"], false, "{provider}");
        assert_eq!(report["chat_completion"], false, "{provider}");
        assert_eq!(report["route"], "unavailable", "{provider}");
        assert_eq!(report["native_tool_loop"], false, "{provider}");
        assert_eq!(report["structured_output"], false, "{provider}");
        assert_eq!(report["streaming"], false, "{provider}");
        let expected_backend_status = if provider == "openai-compatible" {
            "executable"
        } else {
            "boundary-unavailable"
        };
        let expected_support_label = if provider == "openai-compatible" {
            "executable-test-backed"
        } else {
            "deferred/unavailable"
        };
        let expected_support_tier = if provider == "openai-compatible" {
            "partial"
        } else {
            "deferred"
        };
        assert_eq!(report["backend_status"], expected_backend_status);
        assert_eq!(report["support_label"], expected_support_label);
        assert_eq!(report["support_tier"], expected_support_tier);
        assert!(report["reason_codes"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("provider-boundary-unavailable")));
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[test]
fn status_reports_blocked_modes_without_hiding_degradation() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("aidens.toml");
    std::fs::write(
        &path,
        r#"
app_id = "status-agent"
memory_mode = "disabled"
receipt_level = "standard"

[provider]
kind = "openai"
model = "gpt-test"
"#,
    )
    .unwrap();

    let report = status(Some(path.display().to_string())).unwrap();
    let report: OperatorStatusReportV1 = serde_json::from_str(&report).unwrap();

    assert_eq!(
        report.kind,
        aidens_contracts::ArtifactKindV1::OperatorStatusReport
    );
    assert_eq!(report.provider_route_label, "unavailable");
    assert!(report.exposes_degraded_modes());
    assert!(report.blocked_modes.contains(&"provider:openai".into()));

    let raw_report: serde_json::Value =
        serde_json::from_str(&status(Some(path.display().to_string())).unwrap()).unwrap();
    assert!(raw_report["operator_support_tiers"]["deferred"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("provider-matrix:openai")));
    assert!(raw_report["operator_support_tiers"]["scaffold"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("crate:aidens-profile-daemon")));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn package_readiness_blocks_scaffold_completion_claims() {
    let root = temp_root();
    std::fs::create_dir_all(root.join("examples")).unwrap();
    std::fs::write(
        root.join("README.md"),
        "The aidens-profile-research crate is complete for operators.\n",
    )
    .unwrap();
    std::fs::write(
        root.join("examples").join("aidens.mock.toml"),
        r#"
app_id = "release-agent"
profile_id = "coding-agent"
memory_mode = "disabled"
receipt_level = "full"

[provider]
kind = "mock"
model = "mock-model"
mock_response = "release smoke"

[receipts]
store_root = "receipts"

[tools]
sandbox_root = "."
enabled_bundles = ["safe-coding"]
"#,
    )
    .unwrap();

    let report = release_readiness_report(&root, "examples/aidens.mock.toml", false).unwrap();
    assert!(report.blocks_release());
    assert_eq!(report.public_doc_findings.len(), 1);
    assert_eq!(
        report.public_doc_findings[0].surface_id,
        "crate:aidens-profile-research"
    );

    let error = package_command(PackageCommand::Readiness {
        root: root.display().to_string(),
        config: "examples/aidens.mock.toml".into(),
        include_verify: false,
    })
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("public-doc-claims-scaffold-complete"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn package_examples_manifest_covers_public_profiles_honestly() {
    let manifest = package_command(PackageCommand::Examples {
        root: workspace_root().display().to_string(),
    })
    .unwrap();
    let raw_manifest: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    assert!(raw_manifest["operator_support_tiers"]["partial"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("examples/aidens.memory.toml")));
    assert!(raw_manifest["operator_support_tiers"]["scaffold"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value
            .as_str()
            .unwrap_or_default()
            .contains("aidens-profile-research")));

    let manifest: ExampleAppManifestV1 = serde_json::from_value(raw_manifest).unwrap();

    for profile in [
        "chat-only",
        "coding-agent",
        "memory-agent",
        "autonomous-daemon",
        "research-workbench",
    ] {
        assert!(manifest.covers_profile(profile), "missing {profile}");
    }
    assert!(manifest
        .unsupported_advanced_features
        .iter()
        .any(|feature| feature.contains("aidens-profile-research")));
    assert!(manifest.examples.iter().any(|example| {
        example.path == "examples/aidens.memory.toml"
            && example.status == ReleaseSurfaceStateV1::Partial
    }));
    assert!(manifest.examples.iter().any(|example| {
        example.path == "examples/aidens.ollama.toml"
            && example.status == ReleaseSurfaceStateV1::Partial
            && example
                .reason_codes
                .contains(&"provider-local-service-required:ollama".into())
    }));
}

#[test]
fn package_completion_audit_reports_deferred_horizon_without_healthy_claims() {
    let gate_results = P24_REQUIRED_GATE_COMMANDS
        .iter()
        .map(|command| format!("{command}=passed"))
        .collect::<Vec<_>>();
    let encoded = package_command(PackageCommand::CompletionAudit {
        root: workspace_root().display().to_string(),
        config: "examples/aidens.mock.toml".into(),
        gate_results,
    })
    .unwrap();
    let report: CompletionAuditReportV1 = serde_json::from_str(&encoded).unwrap();

    assert_eq!(
        report.source_basis,
        workspace_source_basis_label(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .unwrap()
        )
    );
    assert!(!report
        .source_basis
        .contains("libraries-source-clean-20260426.zip"));
    assert!(!report.release_bar_passed);
    assert_ne!(
        report.completion_state,
        aidens_contracts::CompletionAuditStateV1::Complete
    );
    assert!(report
        .deferred_surfaces
        .iter()
        .any(|surface| surface == "crate:aidens-profile-daemon"));
    assert!(report.known_limitations.current);
    assert!(report
        .reason_codes
        .iter()
        .any(|reason| reason == "release-bar-blocked"));
    assert!(!report
        .release_readiness
        .public_doc_findings
        .iter()
        .any(|finding| finding.reason_code.contains("healthy")));
}

#[test]
fn doctor_fails_memory_required_without_memory_store() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("aidens.toml");
    std::fs::write(
        &path,
        r#"
app_id = "memory-required-agent"
memory_mode = "required"
receipt_level = "full"

[provider]
kind = "mock"
mock_response = "ok"
"#,
    )
    .unwrap();

    let error = doctor(Some(path.display().to_string())).unwrap_err();

    assert!(error
        .to_string()
        .contains("memory-required-without-durable-store"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn doctor_reports_optional_memory_without_store_as_degraded() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("aidens.toml");
    std::fs::write(
        &path,
        r#"
app_id = "optional-memory-agent"
memory_mode = "optional"
receipt_level = "full"

[provider]
kind = "mock"
mock_response = "ok"
"#,
    )
    .unwrap();

    let report = doctor(Some(path.display().to_string())).unwrap();
    let report: AiDENsDoctorReportV1 = serde_json::from_str(&report).unwrap();
    let memory = report.sections.get("memory").unwrap();

    assert!(memory[0].states.contains(&CapabilityStateV1::Degraded));
    assert!(!memory[0].states.contains(&CapabilityStateV1::Healthy));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn run_fails_memory_required_without_memory_store() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("aidens.toml");
    std::fs::write(
        &path,
        r#"
app_id = "memory-required-agent"
memory_mode = "required"
receipt_level = "full"

[provider]
kind = "mock"
mock_response = "ok"
"#,
    )
    .unwrap();

    let error =
        run_once_command(Some(path.display().to_string()), vec!["hello".into()]).unwrap_err();

    assert!(error
        .to_string()
        .contains("memory-required-without-durable-store"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn list_tools_exposes_safe_tools_only() {
    let report = list_tools().unwrap();

    assert!(report.contains("aidens:repo-read:1"));
    assert!(report.contains("declared"));
    assert!(report.contains("executable"));
    assert!(report.contains("hidden_this_turn"));
    assert!(report.contains("blocked_this_turn"));
    assert!(report.contains("declared_but_not_registered"));
    assert!(report.contains("provider_schema_tool_ids"));
    assert!(report.contains("aidens:shell:1"));
}

#[test]
fn inspect_tools_reports_registered_vs_executable() {
    let report = inspect_tools(None).unwrap();
    let report: serde_json::Value = serde_json::from_str(&report).unwrap();

    assert!(report["declared"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("aidens:repo-read:1")));
    assert!(report["registered"].as_array().unwrap().len() > 1);
    assert!(report["executable"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("aidens:repo-read:1")));
    assert!(report["hidden_this_turn"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("aidens:shell:1")));
    assert!(report["blocked_this_turn"].as_array().unwrap().len() >= 2);
    assert!(report["declared_but_not_registered"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("aidens:shell:1")));
    assert!(report["provider_schema_tool_ids"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("aidens:repo-read:1")));
    assert!(report["requires_permit"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("aidens:patch-apply:1")));
    assert!(report["support_tiers"]["supported"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("aidens:repo-read:1")));
    assert!(report["support_tiers"]["partial"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("aidens:patch-apply:1")));
    assert!(report["tool_capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .any(|tool| tool["tool_id"] == "aidens:patch-apply:1"
            && tool["requires_permit"] == true
            && tool["blocked_this_turn"] == true
            && tool["support_tier"] == "partial"));
}

#[test]
fn permit_commands_emit_typed_approval_and_permit_artifacts() {
    let request = permit_command(PermitCommand::Request {
        tool_id: "aidens:file-write:1".into(),
        risk: "file-write".into(),
        sandbox_root: "/repo".into(),
    })
    .unwrap();
    let request: ApprovalRequestV1 = serde_json::from_str(&request).unwrap();
    assert_eq!(request.tool_id, "aidens:file-write:1");
    assert_eq!(request.sandbox_root, "/repo");

    let approval = permit_command(PermitCommand::Approve {
        request_id: request.request_id.0.clone(),
        tool_id: "aidens:file-write:1".into(),
        risk: "file-write".into(),
        sandbox_root: "/repo".into(),
        decided_by: "operator".into(),
    })
    .unwrap();
    let approval: ApprovalDecisionV1 = serde_json::from_str(&approval).unwrap();
    assert!(approval.approved);
    assert_eq!(
        approval
            .permit_grant
            .as_ref()
            .map(|grant| grant.tool_id.as_str()),
        Some("aidens:file-write:1")
    );

    let denial = permit_command(PermitCommand::Deny {
        request_id: request.request_id.0,
        decided_by: "operator".into(),
        reason: "no".into(),
    })
    .unwrap();
    let denial: ApprovalDecisionV1 = serde_json::from_str(&denial).unwrap();
    assert!(!denial.approved);

    let revocation = permit_command(PermitCommand::Revoke {
        permit_id: "permit:test".into(),
        tool_id: "aidens:file-write:1".into(),
        risk: "file-write".into(),
        sandbox_root: "/repo".into(),
        reason: "expired".into(),
    })
    .unwrap();
    let revocation: PermitUseReportV1 = serde_json::from_str(&revocation).unwrap();
    assert!(!revocation.allowed);
    assert!(revocation
        .reason_codes
        .contains(&"permit-revoked:expired".into()));
}

#[test]
fn p10_coding_commands_read_propose_apply_and_packetize() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("README.md"), "hello p10\n").unwrap();

    let read = coding_command(CodingCommand::RepoRead {
        sandbox_root: root.display().to_string(),
        path: "README.md".into(),
    })
    .unwrap();
    assert!(read.contains("RepoRead") || read.contains("repo-read"));

    let diff = "--- a/README.md\n+++ b/README.md\n@@\n-hello p10\n+hello p10 cli\n";
    let proposal = coding_command(CodingCommand::PatchPropose {
        sandbox_root: root.display().to_string(),
        summary: "update readme".into(),
        diff: diff.into(),
    })
    .unwrap();
    assert!(proposal.contains("\"mutates_files\": false"));
    assert_eq!(
        std::fs::read_to_string(root.join("README.md")).unwrap(),
        "hello p10\n"
    );

    let request = permit_command(PermitCommand::Request {
        tool_id: "aidens:patch-apply:1".into(),
        risk: "file-write".into(),
        sandbox_root: root.canonicalize().unwrap().display().to_string(),
    })
    .unwrap();
    let request: ApprovalRequestV1 = serde_json::from_str(&request).unwrap();
    let approval = permit_command(PermitCommand::Approve {
        request_id: request.request_id.0,
        tool_id: "aidens:patch-apply:1".into(),
        risk: "file-write".into(),
        sandbox_root: root.canonicalize().unwrap().display().to_string(),
        decided_by: "operator".into(),
    })
    .unwrap();

    let applied = coding_command(CodingCommand::PatchApply {
        sandbox_root: root.display().to_string(),
        diff: diff.into(),
        permit_json: approval,
    })
    .unwrap();
    assert!(applied.contains("\"applied\": true"));
    assert_eq!(
        std::fs::read_to_string(root.join("README.md")).unwrap(),
        "hello p10 cli\n"
    );

    let packet = coding_command(CodingCommand::CodexPacket {
        current_pass: "P10".into(),
        next_pass: "P11".into(),
        issue: "coding packet".into(),
        source_map: vec!["crates/aidens-tool-kit/src/lib.rs".into()],
        changed_files: vec!["crates/aidens-tool-kit/src/lib.rs".into()],
        command_receipts: vec![serde_json::to_string(&CommandRunReportV1::blocked(
            root.display().to_string(),
            vec!["cargo".into(), "check".into(), "--workspace".into()],
            "fixture-command-receipt",
        ))
        .unwrap()],
        receipt_ids: vec!["receipt:fixture".into()],
        blockers: Vec::new(),
        notes: vec!["handoff".into()],
    })
    .unwrap();
    let packet: CodexPacketV1 = serde_json::from_str(&packet).unwrap();
    assert!(packet.has_resume_context());
    assert_eq!(packet.commands_run.len(), 1);
    assert_eq!(
        packet.receipt_ids,
        vec![ArtifactId("receipt:fixture".into())]
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn p11_daemon_commands_suppress_duplicates_persist_cancel_and_safe_mode() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let root_arg = root.display().to_string();
    let due_at = "2026-04-27T00:00:00Z".parse().unwrap();

    let first = daemon_command(DaemonCommand::Schedule {
        root: root_arg.clone(),
        name: "cli-p11".into(),
        owner: "daemon-a".into(),
        schedule_id: "once".into(),
        occurrence_key: "same-occurrence".into(),
        due_at,
        payload: r#"{"task":"work"}"#.into(),
        risk: "read-only".into(),
    })
    .unwrap();
    let first: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert_eq!(first["enqueued"], true);
    let job_id = first["job"]["job_id"].as_str().unwrap().to_string();

    let second = daemon_command(DaemonCommand::Schedule {
        root: root_arg.clone(),
        name: "cli-p11".into(),
        owner: "daemon-a".into(),
        schedule_id: "once".into(),
        occurrence_key: "same-occurrence".into(),
        due_at,
        payload: r#"{"task":"work"}"#.into(),
        risk: "read-only".into(),
    })
    .unwrap();
    let second: serde_json::Value = serde_json::from_str(&second).unwrap();
    assert_eq!(second["enqueued"], false);
    assert_eq!(
        second["duplicate_suppression_receipt"]["existing_job_id"],
        job_id
    );

    daemon_command(DaemonCommand::Cancel {
        root: root_arg.clone(),
        name: "cli-p11".into(),
        owner: "daemon-a".into(),
        job_id: job_id.clone(),
        reason: "operator-cancelled".into(),
    })
    .unwrap();
    let snapshot = daemon_command(DaemonCommand::List {
        root: root_arg.clone(),
        name: "cli-p11".into(),
        owner: "daemon-a".into(),
    })
    .unwrap();
    let snapshot: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
    assert_eq!(snapshot["jobs"].as_array().unwrap().len(), 1);
    assert_eq!(snapshot["jobs"][0]["state"], "cancelled");

    daemon_command(DaemonCommand::SafeMode {
        root: root_arg.clone(),
        name: "cli-p11".into(),
        owner: "daemon-a".into(),
        enabled: true,
        reason: "operator-safe-mode".into(),
    })
    .unwrap();
    let blocked = daemon_command(DaemonCommand::Wake {
        root: root_arg.clone(),
        name: "cli-p11".into(),
        owner: "daemon-a".into(),
        source: "filesystem".into(),
        signal_key: "risky".into(),
        payload: r#"{"cmd":"cargo test"}"#.into(),
        risk: "shell".into(),
    })
    .unwrap();
    let blocked: serde_json::Value = serde_json::from_str(&blocked).unwrap();
    assert_eq!(blocked["enqueued"], false);
    assert_eq!(
        blocked["safe_mode_receipt"]["operation"],
        "blocked-risky-job"
    );

    let drained = daemon_command(DaemonCommand::Drain {
        root: root_arg,
        name: "cli-p11".into(),
        owner: "daemon-a".into(),
        reason: "operator-drain".into(),
    })
    .unwrap();
    let drained: serde_json::Value = serde_json::from_str(&drained).unwrap();
    assert_eq!(drained.as_array().unwrap().len(), 0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn provider_route_does_not_claim_native_when_backend_is_unavailable() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("aidens.toml");
    std::fs::write(
        &path,
        r#"
app_id = "agent"
memory_mode = "disabled"
receipt_level = "standard"

[provider]
kind = "openrouter"
model = "test"
api_key = "configured"
"#,
    )
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&provider_check(Some(path.display().to_string())).unwrap()).unwrap();

    assert_eq!(report["executable"], false);
    assert_eq!(report["route"], "unavailable");
    assert_eq!(report["native_tool_loop"], false);
    assert_ne!(report["route"], "native-openai-chat");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn provider_check_reports_ollama_chat_with_native_tool_loop() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("aidens.toml");
    std::fs::write(
        &path,
        r#"
app_id = "agent"
memory_mode = "disabled"
receipt_level = "standard"

[provider]
kind = "ollama"
model = "llama3"
"#,
    )
    .unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&provider_check(Some(path.display().to_string())).unwrap()).unwrap();

    assert_eq!(report["executable"], true);
    assert_eq!(report["route"], "ollama-chat");
    assert_eq!(report["native_tool_loop"], true);
    assert_eq!(report["structured_output"], false);
    assert_eq!(report["support_label"], "partial-local-chat");
    assert_eq!(report["support_tier"], "partial");
    assert!(report["reason_codes"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("ollama-local-service-required")));
    assert!(report["reason_codes"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!(
            "ollama-native-tool-loop-via-function-calling"
        )));
    assert_ne!(report["route"], "native-ollama");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn new_app_scaffold_contains_safe_config_and_tests() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();

    let summary = scaffold_project(AiDENsProfile::CodingAgent, "My Agent", &root).unwrap();

    assert_eq!(summary.package_name, "my-agent");
    assert!(summary.app_dir.join("Cargo.toml").exists());
    assert!(summary
        .app_dir
        .join("aidens-scaffold-manifest.json")
        .exists());
    assert!(summary.app_dir.join("README.md").exists());
    assert!(summary.app_dir.join("AGENT.md").exists());
    assert!(summary.app_dir.join("docs").join("tools.md").exists());
    assert!(summary.app_dir.join("docs").join("permits.md").exists());
    assert!(summary.app_dir.join("docs").join("receipts.md").exists());
    assert!(summary.app_dir.join("src").join("main.rs").exists());
    assert!(summary.app_dir.join("tests").join("smoke.rs").exists());
    let cfg = load_config_file(summary.app_dir.join("aidens.toml"))
        .unwrap()
        .config;
    assert_eq!(cfg.provider.kind, "mock");
    assert_eq!(cfg.provider.model.as_deref(), Some("aidens-safe-mock"));
    assert!(cfg
        .provider
        .mock_response
        .as_deref()
        .unwrap()
        .contains(r#""tool_id":"aidens:repo-read:1""#));
    assert_eq!(cfg.memory_mode, MemoryModeV1::Optional);
    assert_eq!(cfg.receipt_level, ReportLevelV1::Full);
    assert_eq!(
        cfg.receipts.store_root.as_deref(),
        Some("target/aidens-receipts/my-agent")
    );
    assert_eq!(
        cfg.tools.enabled_bundles,
        vec![
            "repo-read",
            "repo-list",
            "file-stat",
            "repo-search",
            "patch-propose"
        ]
    );
    assert!(!cfg.tools.enabled_bundles.contains(&"patch-apply".into()));
    assert!(!cfg.tools.enabled_bundles.contains(&"run-checks".into()));
    assert!(cfg.provider.api_key.is_none());
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(summary.app_dir.join("aidens-scaffold-manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["schema"], "AiDENsScaffoldManifestV1");
    assert_eq!(manifest["provider_route"], "explicit-mock-fixture");
    assert_eq!(manifest["receipt_store"], "target/aidens-receipts/my-agent");
    assert_eq!(
        manifest["readiness_claim"],
        "scaffold-generated; run receipts and smoke tests before claiming a completed app run"
    );
    assert!(manifest["reason_codes"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("scaffold-manifest-first")));
    let smoke = std::fs::read_to_string(summary.app_dir.join("tests").join("smoke.rs")).unwrap();
    assert!(smoke.contains("generated_config_is_receipt_first_and_secret_free"));
    assert!(smoke.contains("aidens-scaffold-manifest.json"));
    let main = std::fs::read_to_string(summary.app_dir.join("src").join("main.rs")).unwrap();
    assert!(main.contains("AiDENsApp::from_config"));
    assert!(main.contains("run_once"));

    let output = run_once_command(
        Some(summary.app_dir.join("aidens.toml").display().to_string()),
        vec!["read README".into()],
    )
    .unwrap();
    assert!(output.contains("README evidence summary"));
    assert!(output.contains("explicit mock fixture"));
    assert!(summary
        .app_dir
        .join("target")
        .join("aidens-receipts")
        .join("my-agent")
        .join("canonical-receipts.ndjson")
        .exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn new_app_scaffold_refuses_overwrite_and_preserves_existing_content() {
    let root = temp_root();
    let app_dir = root.join("existing-agent");
    std::fs::create_dir_all(&app_dir).unwrap();
    std::fs::write(app_dir.join("README.md"), "operator content\n").unwrap();

    let error = scaffold_project_at(AiDENsProfile::CodingAgent, &app_dir).unwrap_err();

    assert!(error
        .to_string()
        .contains("target app directory already exists"));
    assert_eq!(
        std::fs::read_to_string(app_dir.join("README.md")).unwrap(),
        "operator content\n"
    );
    assert!(!root.read_dir().unwrap().any(|entry| entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains("aidens-scaffold-tmp")));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn staged_scaffold_write_is_create_new_and_rejects_path_escape() {
    let root = temp_root();
    let stage = root.join(".stage");
    std::fs::create_dir_all(&stage).unwrap();
    write_scaffold_file(&stage, "README.md", "first\n").unwrap();

    let duplicate = write_scaffold_file(&stage, "README.md", "second\n").unwrap_err();
    assert!(duplicate
        .to_string()
        .contains("failed to create-new scaffold file"));
    assert_eq!(
        std::fs::read_to_string(stage.join("README.md")).unwrap(),
        "first\n"
    );

    let escape = write_scaffold_file(&stage, "../outside.md", "bad\n").unwrap_err();
    assert!(escape
        .to_string()
        .contains("invalid scaffold relative path"));
    assert!(!root.join("outside.md").exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn new_user_mock_flow_reaches_receipt_inspection() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let summary = scaffold_project(AiDENsProfile::CodingAgent, "New User Agent", &root).unwrap();
    assert!(summary.app_dir.join("aidens.toml").exists());

    let path = root.join("aidens.mock.toml");
    std::fs::write(
        &path,
        r#"
app_id = "new-user-mock"
profile_id = "coding-agent"
memory_mode = "disabled"
receipt_level = "full"

[provider]
kind = "mock"
model = "mock-model"
mock_response = "new user mock response"

[receipts]
store_root = "receipts"

[tools]
sandbox_root = "."
enabled_bundles = ["safe-coding"]
"#,
    )
    .unwrap();

    let provider_report: serde_json::Value =
        serde_json::from_str(&provider_check(Some(path.display().to_string())).unwrap()).unwrap();
    assert_eq!(provider_report["executable"], true);
    assert!(inspect_tools(Some(path.display().to_string()))
        .unwrap()
        .contains("aidens:repo-read:1"));
    assert_eq!(
        run_once_command(Some(path.display().to_string()), vec!["hello".into()]).unwrap(),
        "new user mock response"
    );
    let receipts = receipts_command(EventLogCommand::List {
        store: None,
        config: Some(path.display().to_string()),
    })
    .unwrap();
    assert!(receipts.contains("\"owner_crate\": \"aidens-orchestration\""));
    assert!(receipts.contains("\"schema_name\": \"run-report-v1\""));
    assert!(workspace_root().join("scripts").join("verify.sh").exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn run_test_agent_writes_bundle_and_receipts_through_runner() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let config = workspace_root()
        .join("fixtures")
        .join("test-agent")
        .join("basic-agent.toml");

    let report = run_test_agent_command(
        &config.display().to_string(),
        None,
        Some(root.display().to_string()),
    )
    .unwrap();

    assert!(report.contains("AiDENs run-test-agent"));
    for file in [
        "final.txt",
        "run-bundle.json",
        "run-report.json",
        "turn-report.json",
        "tool-exposure.json",
        "agency-policy-reports.json",
        "event-log.ndjson",
        "summary.md",
    ] {
        assert!(root.join(file).exists(), "missing bundle file {file}");
    }
    let run_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("run-report.json")).unwrap())
            .unwrap();
    assert_eq!(run_report["provider_route"]["provider_kind"], "mock");
    assert_eq!(run_report["provider_route"]["native_tool_loop"], false);
    assert!(run_report["tool_invocation_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|receipt| receipt["tool_id"] == "aidens:repo-read:1"));

    let agency_reports: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("agency-policy-reports.json")).unwrap(),
    )
    .unwrap();
    assert!(!agency_reports.as_array().unwrap().is_empty());
    let event_log = std::fs::read_to_string(root.join("event-log.ndjson")).unwrap();
    assert!(event_log.contains("\"event\":\"provider_route_selected\""));
    assert!(event_log.contains("\"event\":\"tool_invocation_recorded\""));
    assert!(event_log.contains("\"event\":\"agency_policy_evaluated\""));
    assert!(root
        .join("receipts")
        .join("canonical-receipts.ndjson")
        .exists());
    let run_bundle: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("run-bundle.json")).unwrap())
            .unwrap();
    assert_eq!(run_bundle["schema"], "AiDENsRunBundleV2");
    assert_eq!(run_bundle["support"]["support_tier"], "fixture-supported");
    assert_eq!(
        run_bundle["canonical_execution_context"]["provider_route"],
        "mock"
    );
    assert!(run_bundle["tool_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|receipt| receipt.as_str().unwrap_or_default().starts_with("tool-")));
    assert!(run_bundle["support"]["deferred"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("cloud-provider-execution")));
    assert!(run_bundle["event_log"]["digest"].as_str().unwrap().len() == 64);
    assert!(run_bundle["canonical_execution_context"]["trace_ctx"].is_object());
    let inspected = inspect_run_bundle_command(&root.display().to_string()).unwrap();
    let inspected: serde_json::Value = serde_json::from_str(&inspected).unwrap();
    assert_eq!(inspected["support_tier"], "fixture-supported");
    assert_eq!(inspected["event_log_digest_verified"], true);
    assert!(inspected["canonical_record_count"].as_u64().unwrap() > 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn run_coding_agent_writes_v2_bundle_and_blocks_unapproved_write() {
    let root = temp_root();
    let repo = root.join("repo");
    let out = root.join("out");
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::write(
        repo.join("README.md"),
        "# Local Coding Fixture\n\nstatus: draft\n",
    )
    .unwrap();
    std::fs::write(
        repo.join("src").join("lib.rs"),
        "pub fn ok() -> bool { true }\n",
    )
    .unwrap();
    let config = root.join("coding-agent.toml");
    std::fs::write(
            &config,
            format!(
                r#"
app_id = "p24-test-coding-agent"
profile_id = "coding-agent"
memory_mode = "disabled"
receipt_level = "full"

[provider]
kind = "mock"
mock_response = "unused"

[tools]
enabled_bundles = ["repo-read", "repo-list", "repo-search", "file-stat", "patch-propose", "patch-apply"]
sandbox_root = "{}"
"#,
                repo.display()
            ),
        )
        .unwrap();

    let summary = run_coding_agent_command(
        &config.display().to_string(),
        Some(out.display().to_string()),
        None,
    )
    .unwrap();
    assert!(summary.contains("AiDENs run-coding-agent"));
    assert_eq!(
        std::fs::read_to_string(repo.join("README.md")).unwrap(),
        "# Local Coding Fixture\n\nstatus: draft\n"
    );
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join("coding-agent-report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(report["support_tier"], "supported-local");
    assert!(report["steps"].as_array().unwrap().iter().any(|step| {
        step["label"] == "patch_apply_permit_gate"
            && step["status"] == "blocked_or_failed"
            && step["approval_request"].is_object()
    }));
    assert!(report["steps"].as_array().unwrap().iter().any(|step| {
        step["label"] == "run_checks_permit_gate"
            && step["status"] == "blocked_or_failed"
            && step["approval_request"].is_object()
    }));
    assert_eq!(report["semantic_status"], "exact_check");
    assert!(report["receipt_chain"].as_array().unwrap().len() >= 7);
    assert_eq!(
        report["v11a_evidence"]["completion_gate"]["status"],
        "complete"
    );
    assert_eq!(
        report["v11a_evidence"]["completion_gate"]["material_done"],
        true
    );
    assert_eq!(
        report["v11a_evidence"]["completion_gate"]["proof_debt_blocks"],
        false
    );
    assert_eq!(
        report["v11a_evidence"]["artifact_envelope"]["lifecycle_state"],
        "verified"
    );
    assert_eq!(
        report["v11a_evidence"]["execution_context"]["provider_route"],
        "local-tools-only"
    );
    assert_eq!(
        report["v11a_evidence"]["operator_contract"]["operator_id"],
        "aidens.runner.turn"
    );
    assert!(
        report["v11a_evidence"]["input_manifest"]["inputs"]
            .as_array()
            .unwrap()
            .len()
            == 1
    );
    assert!(
        report["v11a_evidence"]["output_manifest"]["outputs"]
            .as_array()
            .unwrap()
            .len()
            == 1
    );
    assert_eq!(
        report["v11a_evidence"]["semantic_state"]["exactness"],
        "exact"
    );
    assert_eq!(
        report["v11a_evidence"]["view_disclosure"]["support_label"],
        "supported-local"
    );
    assert!(report["loop_summary"]["blocked_steps"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("patch_apply_permit_gate")));
    let bundle: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("run-bundle.json")).unwrap())
            .unwrap();
    assert_eq!(bundle["schema"], "AiDENsRunBundleV2");
    assert_eq!(bundle["support"]["support_tier"], "supported-local");
    assert_eq!(bundle["failure"]["class"], "none");
    assert!(bundle["tool_receipts"].as_array().unwrap().len() >= 7);

    let inspected =
        inspect_run_bundle_command(&out.join("run-bundle.json").display().to_string()).unwrap();
    let inspected: serde_json::Value = serde_json::from_str(&inspected).unwrap();
    assert_eq!(inspected["support_tier"], "supported-local");
    assert_eq!(inspected["event_log_digest_verified"], true);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn run_coding_agent_records_failed_checks_with_admin_permit() {
    let root = temp_root();
    let repo = root.join("repo");
    let out = root.join("out");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::write(repo.join("README.md"), "# Failed Check Fixture\n").unwrap();
    std::fs::write(
        repo.join("Cargo.toml"),
        "[package]\nname = \"broken-check\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    let config = write_coding_agent_test_config(&root, &repo, "p27-failed-check-agent");
    let admin_grant = PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Admin,
        "aidens:run-checks:1",
        repo.canonicalize().unwrap().display().to_string(),
        "operator",
    );

    run_coding_agent_command(
        &config.display().to_string(),
        Some(out.display().to_string()),
        Some(serde_json::to_string(&vec![admin_grant]).unwrap()),
    )
    .unwrap();

    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join("coding-agent-report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(report["semantic_status"], "degraded_exact_check");
    assert!(report["steps"].as_array().unwrap().iter().any(|step| {
        step["label"] == "run_checks_permit_gate" && step["status"] == "check_failed"
    }));
    assert_eq!(
        report["loop_summary"]["failed_checks"][0],
        "run_checks_permit_gate"
    );
    let bundle: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("run-bundle.json")).unwrap())
            .unwrap();
    assert_eq!(bundle["failure"]["class"], "tool-failed");
    assert_eq!(bundle["failure"]["degraded"], true);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn run_coding_agent_applies_patch_and_runs_successful_check_with_scoped_permits() {
    let root = temp_root();
    let repo = root.join("repo");
    let out = root.join("out");
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::write(repo.join("README.md"), "# Successful Check Fixture\n").unwrap();
    std::fs::write(
        repo.join("Cargo.toml"),
        "[package]\nname = \"successful-check\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        repo.join("src").join("lib.rs"),
        "pub fn ok() -> bool { true }\n",
    )
    .unwrap();
    let config = write_coding_agent_test_config(&root, &repo, "p27-successful-check-agent");
    let sandbox = repo.canonicalize().unwrap().display().to_string();
    let write_grant = PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        sandbox.clone(),
        "operator",
    );
    let admin_grant = PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Admin,
        "aidens:run-checks:1",
        sandbox,
        "operator",
    );

    run_coding_agent_command(
        &config.display().to_string(),
        Some(out.display().to_string()),
        Some(serde_json::to_string(&vec![write_grant, admin_grant]).unwrap()),
    )
    .unwrap();

    let readme = std::fs::read_to_string(repo.join("README.md")).unwrap();
    assert!(readme.contains("[P24 local coding-agent proposed change]"));
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join("coding-agent-report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(report["semantic_status"], "exact_check");
    assert_eq!(report["loop_summary"]["applied_patch"], true);
    assert_eq!(report["loop_summary"]["changed_files"][0], "README.md");
    assert!(report["steps"].as_array().unwrap().iter().any(|step| {
        step["label"] == "run_checks_permit_gate"
            && step["status"] == "success"
            && step["output"]["succeeded"] == true
    }));
    let bundle: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("run-bundle.json")).unwrap())
            .unwrap();
    assert_eq!(bundle["failure"]["class"], "none");
    assert_eq!(
        bundle["failure"]["reason_codes"][0],
        "patch-and-check-loop-succeeded"
    );
    assert!(bundle["permit_receipts"].as_array().unwrap().len() >= 2);
    let _ = std::fs::remove_dir_all(&root);
}

fn write_coding_agent_test_config(root: &Path, repo: &Path, app_id: &str) -> PathBuf {
    let config = root.join(format!("{app_id}.toml"));
    std::fs::write(
            &config,
            format!(
                r#"
app_id = "{app_id}"
profile_id = "coding-agent"
memory_mode = "disabled"
receipt_level = "full"

[provider]
kind = "mock"
mock_response = "unused"

[tools]
enabled_bundles = ["repo-read", "repo-list", "repo-search", "file-stat", "patch-propose", "patch-apply", "run-checks"]
sandbox_root = "{}"
"#,
                repo.display()
            ),
        )
        .unwrap();
    config
}

#[test]
fn receipts_commands_inspect_restart_visible_run() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("aidens.toml");
    std::fs::write(
        &path,
        r#"
app_id = "receipt-agent"
memory_mode = "disabled"
receipt_level = "full"

[provider]
kind = "mock"
mock_response = "persisted response"

[receipts]
store_root = "receipts"
"#,
    )
    .unwrap();

    let output = run_once_command(
        Some(path.display().to_string()),
        vec!["hello".into(), "receipt".into()],
    )
    .unwrap();
    assert_eq!(output, "persisted response");

    let list = receipts_command(EventLogCommand::List {
        store: None,
        config: Some(path.display().to_string()),
    })
    .unwrap();
    let list: serde_json::Value = serde_json::from_str(&list).unwrap();
    let run_receipt_id = list["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["schema_name"] == "run-report-v1")
        .and_then(|record| record["receipt_id"].as_str())
        .unwrap()
        .to_string();

    let inspect = receipts_command(EventLogCommand::Inspect {
        store: None,
        config: Some(path.display().to_string()),
        receipt_id: run_receipt_id.clone(),
    })
    .unwrap();
    let inspected: serde_json::Value = serde_json::from_str(&inspect).unwrap();
    assert_eq!(inspected["owner_crate"], "aidens-orchestration");
    assert_eq!(inspected["schema_name"], "run-report-v1");
    assert_eq!(inspected["body"]["kind"], "run");
    assert_eq!(inspected["content_digest"].as_str().unwrap().len(), 64);

    let verified = receipts_command(EventLogCommand::VerifyDigest {
        store: None,
        config: Some(path.display().to_string()),
        receipt_id: run_receipt_id,
    })
    .unwrap();
    assert!(verified.contains("\"verified\": true"));

    let export = receipts_command(EventLogCommand::Export {
        store: None,
        config: Some(path.display().to_string()),
    })
    .unwrap();
    assert!(export.contains("\"schema_name\": \"run-report-v1\""));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn agent_run_persists_v3_bundle_in_receipt_store_and_inspects_after_restart() {
    let root = temp_root();
    let agent_dir = root.join("local-agent");
    let out = root.join("run");
    agent_new_command("local-coding", &agent_dir.display().to_string()).unwrap();

    let summary = agent_run_command(
        &agent_dir.join("agent.json").display().to_string(),
        &agent_dir.join("task.md").display().to_string(),
        &out.display().to_string(),
        Some(agent_dir.join("sandbox").display().to_string()),
        None,
        None,
    )
    .unwrap();

    assert!(summary.contains("run_bundle_store:"));
    assert!(out.join("run-bundle.json").exists());
    assert!(out.join("run-bundle-store-record.json").exists());
    assert!(out
        .join("receipts")
        .join("run-bundles")
        .join("index.ndjson")
        .exists());
    let loop_output: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join("plan-act-verify-output.json")).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        loop_output["semantic_disclosure"]["semantic_status"].as_str(),
        Some("exact_check" | "degraded_exact_check")
    ));
    assert_eq!(
        loop_output["semantic_disclosure"]["support_tier"],
        "supported-local"
    );

    let inspected_from_output = inspect_run_bundle_command(&out.display().to_string()).unwrap();
    let inspected_from_output: serde_json::Value =
        serde_json::from_str(&inspected_from_output).unwrap();
    assert_eq!(inspected_from_output["bundle_schema"], "AiDENsRunBundleV3");
    assert_eq!(inspected_from_output["event_log_digest_verified"], true);
    assert_eq!(
        inspected_from_output["semantic_disclosure"]["semantic_status"],
        "degraded_exact_check"
    );
    assert_eq!(
        inspected_from_output["semantic_disclosure"]["support_tier"],
        "supported-local"
    );
    assert_eq!(
        inspected_from_output["run_bundle_store_record"]["semantic_status"],
        "degraded_exact_check"
    );

    let inspected_from_store =
        inspect_run_bundle_command(&out.join("receipts").display().to_string()).unwrap();
    let inspected_from_store: serde_json::Value =
        serde_json::from_str(&inspected_from_store).unwrap();
    assert_eq!(inspected_from_store["bundle_schema"], "AiDENsRunBundleV3");
    assert_eq!(inspected_from_store["event_log_digest_verified"], true);
    assert_eq!(
        inspected_from_store["run_bundle_store_record"]["artifact_kind"],
        "local_operator_run_bundle_store_record"
    );
    assert!(
        inspected_from_store["canonical_record_count"]
            .as_u64()
            .unwrap()
            > 0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn boundary_compile_cli_rejects_duplicate_keys() {
    let output = boundary_command(BoundaryCommand::Compile {
        input: r#"{"path":"a","path":"b"}"#.into(),
        schema: None,
        treatment_fields: Vec::new(),
        hard_fail_treatment_change: false,
    })
    .unwrap();
    let output: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(output["accepted"], false);
    assert_eq!(output["duplicate_key_findings"][0]["key"], "path");
    assert_eq!(output["reason_codes"][0], "duplicate-json-object-key");
}

#[test]
fn boundary_compile_cli_reports_schema_failure() {
    let output = boundary_command(BoundaryCommand::Compile {
        input: r#"{"path":7}"#.into(),
        schema: Some(
            r#"{"type":"object","required":["path"],"properties":{"path":{"type":"string"}}}"#
                .into(),
        ),
        treatment_fields: Vec::new(),
        hard_fail_treatment_change: false,
    })
    .unwrap();
    let output: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(output["accepted"], false);
    assert_eq!(output["schema_validation"]["kind"], "schema-validation");
    assert_eq!(output["schema_validation"]["valid"], false);
}

#[test]
fn boundary_compile_cli_rejects_unsupported_schema_keywords() {
    let output = boundary_command(BoundaryCommand::Compile {
        input: r#""abc""#.into(),
        schema: Some(r#"{"type":"string","pattern":"^a"}"#.into()),
        treatment_fields: Vec::new(),
        hard_fail_treatment_change: false,
    })
    .unwrap();
    let output: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(output["accepted"], false);
    assert!(output["schema_validation"]["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|error| error
            .as_str()
            .unwrap()
            .contains("unsupported schema keyword 'pattern'")));
}

#[test]
fn phase13_agent_validate_schema_checks_and_rejects_duplicate_keys() {
    let root = temp_root();
    let agent_dir = root.join("agent");
    agent_new_command("local-coding", &agent_dir.display().to_string()).unwrap();

    let valid = agent_validate_command(&agent_dir.join("agent.json").display().to_string())
        .expect("generated AgentSpec should validate");
    let valid: serde_json::Value = serde_json::from_str(&valid).unwrap();
    assert_eq!(valid["schema_validation"]["valid"], true);
    assert_eq!(valid["validation"]["valid"], true);
    assert_eq!(
        valid["semantic_disclosure"]["semantic_status"],
        "exact_check"
    );
    assert_eq!(valid["semantic_disclosure"]["exactness"], "exact_check");
    assert_eq!(
        valid["semantic_disclosure"]["support_tier"],
        "supported-local"
    );
    assert!(valid["semantic_disclosure"]["proof_checks"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("AgentSpecV1-policy-validation")));

    std::fs::write(
        agent_dir.join("agent.json"),
        r#"{"schema":"AgentSpecV1","schema":"AgentSpecV1"}"#,
    )
    .unwrap();
    let error = agent_validate_command(&agent_dir.join("agent.json").display().to_string())
        .expect_err("duplicate-key AgentSpec must fail strict parse");
    assert!(format!("{error:?}").contains("duplicate json object keys"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn phase13_inspect_run_bundle_rejects_duplicate_keys_before_schema_use() {
    let root = temp_root();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("run-bundle.json"),
        r#"{"schema":"AiDENsRunBundleV3","schema":"AiDENsRunBundleV3"}"#,
    )
    .unwrap();

    let error = inspect_run_bundle_command(&root.display().to_string())
        .expect_err("duplicate-key run bundle must fail strict parse");
    assert!(format!("{error:?}").contains("duplicate json object keys"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn phase13_permit_json_rejects_duplicate_keys() {
    let duplicate = r#"{"permit_id":"permit:test","permit_id":"permit:other"}"#;
    let error = permit_policy_from_json(duplicate)
        .expect_err("duplicate-key permit JSON must fail strict parse");
    assert!(format!("{error:?}").contains("duplicate json object keys"));
}

#[test]
fn view_query_cli_uses_canonical_runtime() {
    let root = temp_root();
    let memory_root = root.join("memory");

    let output = view_command(ViewCommand::Query {
        memory_store: memory_root.display().to_string(),
        view_mode: "semantic".into(),
        query: "repository status".into(),
        subject: None,
        predicate: None,
        valid_at: None,
        recorded_at: None,
        aliases: Vec::new(),
        allow_alias_expansion: false,
        allow_timeless_fallback: false,
    })
    .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(parsed["kind"], "canonical-runtime-view");
    assert_eq!(parsed["canonical_owner"], "knowledge-runtime");
    assert_eq!(parsed["memory_owner"], "semantic-memory");
    assert_eq!(parsed["result_count"], 0);
    assert!(parsed["trace"].is_object());

    let temporal_output = view_command(ViewCommand::Query {
        memory_store: memory_root.display().to_string(),
        view_mode: "temporal".into(),
        query: "repository status".into(),
        subject: None,
        predicate: None,
        valid_at: Some("2026-04-01T00:00:00Z".parse().unwrap()),
        recorded_at: Some("9999-01-01T00:00:00Z".parse().unwrap()),
        aliases: Vec::new(),
        allow_alias_expansion: false,
        allow_timeless_fallback: false,
    })
    .unwrap();
    let temporal: serde_json::Value = serde_json::from_str(&temporal_output).unwrap();
    assert_eq!(temporal["kind"], "canonical-runtime-view");
    assert_eq!(temporal["trace"]["valid_as_of"], "2026-04-01T00:00:00Z");
    assert_eq!(temporal["trace"]["recorded_as_of"], "9999-01-01T00:00:00Z");

    let local_filter_error = view_command(ViewCommand::Query {
        memory_store: memory_root.display().to_string(),
        view_mode: "entity".into(),
        query: "repository status".into(),
        subject: Some("repository".into()),
        predicate: Some("status".into()),
        valid_at: None,
        recorded_at: None,
        aliases: vec!["repo".into()],
        allow_alias_expansion: false,
        allow_timeless_fallback: false,
    })
    .unwrap_err();
    assert!(local_filter_error
        .to_string()
        .contains("legacy AiDENs memory view filters were removed"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn memory_seam_fixture_imports_and_queries_canonical_stack() {
    let root = temp_root();
    let output = memory_seam_fixture_command(Some(root.display().to_string())).unwrap();
    assert!(output.contains("AiDENs memory seam fixture"));
    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("memory-runtime-seam-report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(report["digest_preserved"], true);
    assert_eq!(report["import_result"]["status"], "complete");
    assert!(report["query"]["result_count"].as_u64().unwrap() > 0);
    assert_eq!(report["canonical_owners"]["bridge"], "forge-memory-bridge");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn schemas_generate_is_deterministic_and_check_passes() {
    let root = temp_root();
    let root_arg = root.display().to_string();

    let first = schemas_generate(&root_arg).unwrap();
    let manifest =
        std::fs::read_to_string(root.join("generated_schema_manifest_v1.json")).expect("manifest");
    let compatibility_schema = std::fs::read_to_string(
        root.join("schema-compatibility-report")
            .join("v1.schema.json"),
    )
    .expect("compatibility schema");
    let second = schemas_generate(&root_arg).unwrap();

    assert_eq!(first, second);
    assert_eq!(
        manifest,
        std::fs::read_to_string(root.join("generated_schema_manifest_v1.json")).unwrap()
    );
    assert_eq!(
        compatibility_schema,
        std::fs::read_to_string(
            root.join("schema-compatibility-report")
                .join("v1.schema.json")
        )
        .unwrap()
    );
    let report: SchemaCompatibilityReportV1 =
        serde_json::from_str(&schemas_check(&root_arg).unwrap()).unwrap();
    assert!(report.compatible);
    assert_eq!(
        report.checked_schema_count,
        generated_schema_documents().len()
    );
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("generated_schema_manifest_v1.json")).unwrap(),
    )
    .unwrap();
    assert!(manifest["schemas"]
        .as_array()
        .unwrap()
        .iter()
        .all(|entry| entry["schema_identity"]
            .as_str()
            .is_some_and(
                |identity| identity.starts_with("schema:") && identity.contains("blake3:")
            )));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn schemas_check_fails_on_unregistered_artifact_family() {
    let root = temp_root();
    let root_arg = root.display().to_string();
    schemas_generate(&root_arg).unwrap();
    let unknown = root.join("unknown-family");
    std::fs::create_dir_all(&unknown).unwrap();
    std::fs::write(unknown.join("v1.schema.json"), "{}\n").unwrap();

    let error = schemas_check(&root_arg).expect_err("unknown schema should fail check");

    assert!(error
        .to_string()
        .contains("unregistered-artifact-family-schema"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn phase13_schemas_check_rejects_duplicate_keys_in_schema_files() {
    let root = temp_root();
    let root_arg = root.display().to_string();
    schemas_generate(&root_arg).unwrap();
    std::fs::write(
        root.join("agent-spec").join("v1.schema.json"),
        r#"{"type":"object","type":"array"}"#,
    )
    .unwrap();

    let error = schemas_check(&root_arg).expect_err("duplicate-key schema must fail check");
    assert!(error
        .to_string()
        .contains("schema-json-invalid-or-duplicate-key"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn phase11_schemas_check_rejects_case_folded_path_collisions() {
    let root = temp_root();
    let root_arg = root.display().to_string();
    schemas_generate(&root_arg).unwrap();
    let collision_dir = root.join("Agent-Spec");
    std::fs::create_dir_all(&collision_dir).unwrap();
    std::fs::copy(
        root.join("agent-spec").join("v1.schema.json"),
        collision_dir.join("v1.schema.json"),
    )
    .unwrap();

    let error = schemas_check(&root_arg).expect_err("case-folded schema path collision");

    assert!(error
        .to_string()
        .contains("schema-path-case-fold-collision"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn schemas_check_fails_on_same_major_schema_drift() {
    let root = temp_root();
    let root_arg = root.display().to_string();
    schemas_generate(&root_arg).unwrap();
    let schema_path = root
        .join("schema-compatibility-report")
        .join("v1.schema.json");
    let mut schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_path).unwrap()).unwrap();
    schema["description"] = serde_json::Value::String("manual drift without version bump".into());
    std::fs::write(&schema_path, serde_json::to_string_pretty(&schema).unwrap()).unwrap();

    let error = schemas_check(&root_arg).expect_err("schema drift should fail check");

    assert!(error
        .to_string()
        .contains("schema-content-drift-without-major-bump"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn sanitize_rejects_empty_names() {
    assert!(sanitize_package_name("!!!").is_err());
    assert_eq!(
        sanitize_package_name("123 Agent").unwrap(),
        "aidens-123-agent"
    );
}

#[test]
fn p30_agent_run_fallback_attempt_family_id_is_material_derived() {
    let agent_src = include_str!("agent.rs");
    assert!(agent_src.contains(r#"generated_artifact_id_from_material("attempt-family", &run_id)"#));
    assert!(!agent_src.contains(r#"display_only_unstable_id("attempt-family")"#));
}

#[test]
fn p30_cli_source_does_not_reference_legacy_generated_artifact_id_api() {
    let lib_src = include_str!("lib.rs");
    let agent_src = include_str!("agent.rs");
    assert!(!lib_src.contains("generated_artifact_id("));
    assert!(!agent_src.contains("generated_artifact_id("));
}
