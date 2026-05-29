use super::*;

#[test]
fn artifact_id_roundtrips_through_canonical_stack_id() {
    let contracts_id = ArtifactId::new("artifact:contracts-smoke");
    let stack_id: stack_ids::ArtifactId = contracts_id.clone();
    let decoded: ArtifactId =
        serde_json::from_str(&serde_json::to_string(&stack_id).unwrap()).unwrap();
    assert_eq!(decoded.as_str(), "artifact:contracts-smoke");
    assert_eq!(decoded, contracts_id);
}

#[test]
fn stack_digest_is_exposed_as_non_authoritative_display_digest() {
    let digest = canonical_stack::digest_json(&serde_json::json!({"b": 2, "a": 1})).unwrap();
    assert_eq!(digest.hex().len(), 64);
    let wrapped = DisplayDigestV1::from_stack_content_digest_for_display(
        digest.clone(),
        "stack-json-c14n-v1",
    );
    assert_eq!(wrapped.algorithm, "blake3");
    assert!(wrapped.non_authoritative);
    assert_eq!(wrapped.digest, format!("blake3:{}", digest.hex()));
}

#[test]
fn p28_generated_artifact_ids_are_not_random_uuids_and_stable_material_ids_are_replayable() {
    let first = display_only_unstable_id("receipt");
    let second = display_only_unstable_id("receipt");
    assert!(first.0.starts_with("receipt:local-process-seq-"));
    assert!(second.0.starts_with("receipt:local-process-seq-"));
    assert_ne!(first, second);
    assert!(!first.0.contains('-') || !first.0.contains("0000-"));

    let stable_a = generated_artifact_id_from_material("receipt", "operator|input|output");
    let stable_b = generated_artifact_id_from_material("receipt", "operator|input|output");
    let stable_c = generated_artifact_id_from_material("receipt", "operator|different");
    assert_eq!(stable_a, stable_b);
    assert_ne!(stable_a, stable_c);
}

#[test]
fn p30_legacy_process_local_generated_artifact_id_api_is_absent() {
    let lib_rs = include_str!("lib.rs");
    assert!(!lib_rs.contains("pub fn generated_artifact_id("));
    assert!(lib_rs.contains("pub fn display_only_unstable_id("));
    assert!(lib_rs.contains("pub fn generated_artifact_id_from_material("));
}

#[test]
fn execution_context_exposes_stack_trace_and_attempt() {
    let context = AidensRunContextV1::new("stack-contracts-smoke");
    assert_eq!(context.stack_trace_ctx().trace_id, context.trace_id.0);
    assert_eq!(context.stack_attempt_id().as_str(), context.attempt_id.0);
}

#[test]
fn provider_route_labels_are_exact() {
    assert_eq!(
        ProviderRouteKindV1::ParserFallback.to_string(),
        "parser-fallback"
    );
    assert_eq!(ProviderRouteKindV1::OllamaChat.to_string(), "ollama-chat");
    assert_ne!(
        ProviderRouteKindV1::ParserFallback.to_string(),
        "native-ollama"
    );
}

#[test]
fn dangerous_auto_approval_is_rejected_by_default() {
    let plan = AiDENsAppPlanV1 {
        app_id: "app".into(),
        profile_id: "coding".into(),
        provider_required: true,
        memory_mode: MemoryModeV1::Optional,
        receipt_level: ReportLevelV1::Full,
        dangerous_auto_approval: true,
        risk_disclosures: vec![],
        enabled_tool_bundles: vec![],
        disabled_tool_bundles: vec![],
    };
    assert!(plan.validate().is_err());
}

#[test]
fn memory_required_is_valid_as_a_plan_but_config_gates_store_presence() {
    let plan = AiDENsAppPlanV1 {
        app_id: "app".into(),
        profile_id: "memory".into(),
        provider_required: true,
        memory_mode: MemoryModeV1::Required,
        receipt_level: ReportLevelV1::Full,
        dangerous_auto_approval: false,
        risk_disclosures: vec![],
        enabled_tool_bundles: vec![],
        disabled_tool_bundles: vec![],
    };

    assert!(plan.validate().is_ok());
    assert!(plan.human_summary().contains("memory_mode=required"));
}

#[test]
fn p01_api_honesty_receipt_distinguishes_honored_and_blocked_inputs() {
    let honored = ApiHonestyReportV1::honored(
        "AiDENsAppBuilder::build",
        vec!["provider".into(), "tools".into()],
        vec!["provider".into(), "tools".into()],
    );
    let blocked = ApiHonestyReportV1::blocked(
        "AiDENsApp::from_plan",
        vec!["plan.provider_required".into()],
        vec!["provider".into()],
        "provider-unbound",
    );

    assert!(honored.all_inputs_honored());
    assert_eq!(blocked.kind, ArtifactKindV1::ApiHonesty);
    assert!(!blocked.all_inputs_honored());
    assert_eq!(blocked.outcome, ApiHonestyOutcomeV1::Blocked);
}

#[test]
fn p01_parity_report_exposes_mismatches() {
    let report = PlanRuntimeParityReportV1::new(
        "app",
        vec![
            PlanRuntimeParityCheckV1::new(
                PlanRuntimeParityCheckKindV1::ProviderRoute,
                "mock",
                "mock",
            ),
            PlanRuntimeParityCheckV1::new(
                PlanRuntimeParityCheckKindV1::MemoryMode,
                "optional",
                "disabled",
            ),
        ],
    );

    assert!(!report.is_passing());
    assert_eq!(report.kind, ArtifactKindV1::PlanRuntimeParity);
    assert_eq!(report.mismatches.len(), 1);
    assert!(report.mismatches[0].contains("memory-mode"));
}

#[test]
fn p02_provider_route_v2_distinguishes_ollama_chat_from_native_loop() {
    let route = ProviderRouteReportV2::new(ProviderRouteReportDraftV2 {
        provider_kind: "ollama".into(),
        model: Some("llama3".into()),
        route: ProviderRouteKindV1::OllamaChat,
        route_label: "ollama-chat".into(),
        chat_completion_executable: true,
        native_tool_loop: false,
        degraded: false,
        degraded_reason: None,
        reason_codes: vec!["ollama-native-tool-loop-unimplemented".into()],
    });

    let legacy = route.to_v1();

    assert_eq!(route.kind, ArtifactKindV1::ProviderRoute);
    assert_eq!(legacy.route_label, "ollama-chat");
    assert!(!legacy.native_tool_loop);
    assert!(!legacy.degraded);
}

#[test]
fn p02_provider_readiness_receipt_is_receipt_bearing() {
    let readiness = ProviderReadinessReportV1::new(ProviderReadinessReportDraftV1 {
        provider_kind: "openai".into(),
        model: Some("gpt-test".into()),
        configured: true,
        executable: false,
        native_tool_loop_executable: false,
        route_label: "unavailable".into(),
        reason_codes: vec!["provider-boundary-unavailable".into()],
    });

    assert_eq!(readiness.kind, ArtifactKindV1::ProviderReadiness);
    assert!(readiness.configured);
    assert!(!readiness.executable);
    assert!(!readiness.native_tool_loop_executable);
}

#[test]
fn p02_backend_matrix_blocks_native_claims_without_executable_status() {
    let matrix = ProviderBackendMatrixV1::new(vec![ProviderBackendMatrixEntryV1 {
        provider_kind: "openai".into(),
        status: ProviderBackendStatusV1::BoundaryUnavailable,
        route_label: "unavailable".into(),
        api_key_required: true,
        chat_completion_executable: false,
        native_tool_loop_executable: false,
        streaming_executable: false,
        structured_output_executable: false,
        reason_codes: vec!["provider-boundary-unavailable".into()],
    }]);

    let openai = matrix.entry_for(" openai ").unwrap();

    assert_eq!(openai.status, ProviderBackendStatusV1::BoundaryUnavailable);
    assert!(!openai.native_tool_loop_ready());
}

#[test]
fn p02_provider_certification_fixture_records_expected_truth() {
    let fixture = ProviderCertificationFixtureV1::new(ProviderCertificationFixtureDraftV1 {
        provider_kind: "anthropic".into(),
        scenario: "configured-boundary-unavailable".into(),
        input_config: BTreeMap::from([
            ("api_key".into(), "configured".into()),
            ("model".into(), "claude-test".into()),
        ]),
        expected_configured: true,
        expected_executable: false,
        expected_route_label: "unavailable".into(),
        expected_native_tool_loop: false,
        expected_reason_codes: vec!["provider-boundary-unavailable".into()],
    });

    assert_eq!(fixture.provider_kind, "anthropic");
    assert!(!fixture.expected_native_tool_loop);
    assert!(fixture
        .expected_reason_codes
        .contains(&"provider-boundary-unavailable".into()));
}

#[test]
fn p03_tool_invocation_receipt_links_run_attempt_and_digests() {
    let context = AidensRunContextV1::new("p03-test");
    let receipt = ToolInvocationReportV1::started(
        "aidens:repo-read:1",
        serde_json::json!({"path":"README.md"}),
    )
    .with_execution_context(&context)
    .complete_success(serde_json::json!({"content":"ok"}));

    assert_eq!(receipt.kind, ArtifactKindV1::ToolInvocation);
    assert_eq!(receipt.run_id, Some(context.run_id));
    assert_eq!(receipt.attempt_id, Some(context.attempt_id));
    assert_eq!(receipt.outcome, ToolInvocationOutcomeV1::Succeeded);
    assert!(receipt.input_digest.starts_with("blake3:"));
    assert!(receipt.output_digest.is_some());
}

#[test]
fn p03_turn_receipt_records_stop_budget_and_tool_evidence() {
    let context = AidensRunContextV1::new("p03-test");
    let exposure = ToolExposureSetV1 {
        exposure_id: display_only_unstable_id("tool-exposure"),
        declared_tool_ids: vec!["aidens:repo-read:1".into()],
        registered_tool_ids: vec!["aidens:repo-read:1".into()],
        executable_tool_ids: vec!["aidens:repo-read:1".into()],
        exposed_tool_ids: vec!["aidens:repo-read:1".into()],
        hidden_tool_ids: Vec::new(),
        blocked_tool_ids: Vec::new(),
        decisions: Vec::new(),
        approval_requests: Vec::new(),
        permit_use_receipts: Vec::new(),
        provider_tool_schemas: Vec::new(),
        sandbox_root: Some(".".into()),
        degraded: false,
        reason_codes: Vec::new(),
        canonical_backpointers: canonical_owner_backpointer(
            "llm-tool-runtime",
            "ToolExposurePlan",
            "canonical-tool-exposure-owner",
        ),
        reason: None,
    };
    let provider_route = ProviderRouteReportV1::new(
        "mock",
        Some("model".into()),
        ProviderRouteKindV1::Mock,
        vec![],
    );
    let plan = TurnExecutionPlanV1::new(
        TurnModeV1::ParserFallback,
        provider_route,
        &exposure,
        1,
        0,
        1000,
        vec!["parser-fallback-selected".into()],
    );
    let request = ToolCallRequestV1::new(
        ToolCallSourceV1::ParserFallback,
        "aidens:repo-read:1",
        serde_json::json!({"path":"README.md"}),
        None,
        vec!["parser-fallback-tool-call".into()],
    );
    let invocation = ToolInvocationReportV1::started("aidens:repo-read:1", request.input.clone())
        .with_execution_context(&context)
        .complete_failure("budget-exhausted-before-dispatch");
    let stop = StopRuleReportV1::triggered(
        &context,
        StopRuleV1::MaxToolCalls,
        vec!["max-tool-calls-exhausted".into()],
    );
    let budget = BudgetExhaustionReportV1::new(BudgetExhaustionReportDraftV1 {
        run_id: context.run_id.clone(),
        attempt_id: context.attempt_id.clone(),
        max_tool_calls: 0,
        attempted_tool_calls: 1,
        max_retries: 0,
        retries: 0,
        max_turn_millis: 1000,
        elapsed_millis: 1,
        reason_codes: vec!["max-tool-calls-exhausted".into()],
    });
    let mut turn = TurnReportV1::started(&context, &plan);

    turn.record_tool_call(&request, &invocation);
    turn.record_stop_rule(&stop);
    turn.record_budget_exhaustion(&budget);
    let turn = turn.complete(TurnFinalStateV1::BudgetExhausted);

    assert_eq!(turn.kind, ArtifactKindV1::Turn);
    assert!(turn.degraded);
    assert!(turn.blocked);
    assert_eq!(turn.tool_call_ids, vec![request.call_id]);
    assert_eq!(turn.stop_rule_receipt_ids, vec![stop.receipt_id]);
    assert_eq!(turn.budget_exhaustion_receipt_id, Some(budget.receipt_id));
}

#[test]
fn p04_permit_scope_requires_risk_tool_sandbox_and_execution_family() {
    let context = AidensRunContextV1::new("p04-test");
    let grant = PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:file-write:1",
        "/repo",
        "operator",
    )
    .for_execution_context(&context);

    assert!(grant.matches_scope(
        &CanonicalToolSideEffectClass::Write,
        "aidens:file-write:1",
        "/repo",
        Some(&context.run_id),
        Some(&context.attempt_id),
    ));
    assert!(!grant.matches_scope(
        &CanonicalToolSideEffectClass::Write,
        "aidens:shell:1",
        "/repo",
        Some(&context.run_id),
        Some(&context.attempt_id),
    ));
    assert!(!grant.matches_scope(
        &CanonicalToolSideEffectClass::Write,
        "aidens:file-write:1",
        "/other",
        Some(&context.run_id),
        Some(&context.attempt_id),
    ));
}

#[test]
fn p04_unscoped_permit_does_not_match_any_tool_or_root() {
    let context = AidensRunContextV1::new("p04-test-unscoped");
    let unscoped = PermitGrantV1::new(CanonicalToolSideEffectClass::Admin, "/repo", "operator");

    assert!(!unscoped.matches_scope(
        &CanonicalToolSideEffectClass::Admin,
        "aidens:file-write:1",
        "/repo",
        None,
        None,
    ));
    assert!(!unscoped.matches_scope(
        &CanonicalToolSideEffectClass::Admin,
        "aidens:file-write:1",
        "/other",
        None,
        None,
    ));
    assert!(!unscoped.matches_scope(
        &CanonicalToolSideEffectClass::Admin,
        "aidens:repo-write:1",
        "/repo",
        None,
        None,
    ));
    let with_context = unscoped.for_execution_context(&context);
    assert!(!with_context.matches_scope(
        &CanonicalToolSideEffectClass::Admin,
        "aidens:file-write:1",
        "/repo",
        Some(&context.run_id),
        Some(&context.attempt_id),
    ));
}

#[test]
fn p04_gate_decision_carries_approval_or_permit_evidence() {
    let approval = ApprovalRequestV1::scoped(
        "aidens:file-write:1",
        CanonicalToolSideEffectClass::Write,
        "/repo",
        "side-effect tool requires explicit scoped permit",
    );
    let decision = CapabilityGateDecisionV1::for_tool(CapabilityGateDecisionDraftV1 {
        tool_id: "aidens:file-write:1".into(),
        outcome: CapabilityGateOutcomeV1::Blocked,
        lifecycle: vec![
            ToolLifecycleStateV1::Declared,
            ToolLifecycleStateV1::Registered,
            ToolLifecycleStateV1::Blocked,
        ],
        risk_class: CanonicalToolSideEffectClass::Write,
        permit_required: true,
        executable_this_turn: false,
        sandbox_root: Some("/repo".into()),
        approval_request: Some(approval.clone()),
        permit_grant_id: None,
        permit_use_receipt_id: None,
        reason_codes: vec!["permit-required:write".into()],
    });

    assert_eq!(decision.kind, ArtifactKindV1::ToolExposure);
    assert_eq!(decision.outcome, CapabilityGateOutcomeV1::Blocked);
    assert_eq!(
        decision.approval_request.map(|request| request.request_id),
        Some(approval.request_id)
    );
    assert!(decision.lifecycle.contains(&ToolLifecycleStateV1::Blocked));
}

#[test]
fn p05_display_json_digest_is_stack_owned_and_stable_across_json_layout() {
    let pretty: serde_json::Value = serde_json::from_str(
        r#"{
              "z": ["b", "a"],
              "a": { "n": 1, "ok": true }
            }"#,
    )
    .unwrap();
    let compact: serde_json::Value =
        serde_json::from_str(r#"{"a":{"ok":true,"n":1},"z":["b","a"]}"#).unwrap();

    assert_eq!(
        non_authoritative_json_display_digest(&pretty),
        non_authoritative_json_display_digest(&compact)
    );
}

#[test]
fn p05_remaining_artifact_constructors_produce_linkable_shapes() {
    let receipt_id = display_only_unstable_id("receipt");
    let parent_receipt_id = display_only_unstable_id("receipt");
    let content_digest =
        non_authoritative_json_display_digest(&serde_json::json!({"tool":"read","ok":false}));
    let poison = PoisonReportEntryV1::new("receipt-transactions.ndjson", 7, "{bad}", "json-error");
    let graph = ExecutionLineageGraphV1::new(
        Some(display_only_unstable_id("run")),
        vec![ExecutionLineageNodeV1 {
            receipt_id: receipt_id.clone(),
            kind: ArtifactKindV1::ToolInvocation,
            content_digest: content_digest.clone(),
        }],
        vec![ExecutionLineageEdgeV1 {
            parent_receipt_id,
            child_receipt_id: receipt_id,
            relation: "parent-receipt".into(),
        }],
    );

    assert!(poison.raw_digest.starts_with("blake3:"));
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn p06_boundary_contracts_are_receipt_bearing_and_cryptographic() {
    let left = serde_json::json!({"b": 2, "a": {"n": 1}});
    let right = serde_json::json!({"a": {"n": 1}, "b": 2});
    let digest = DisplayDigestV1::for_json_value(&left);
    let duplicate = DuplicateKeyFindingV1::new("$", "a", Some(2), Some(9));
    let validation = SchemaValidationReportV1::new(
        Some(&serde_json::json!({"type":"object"})),
        &left,
        Vec::new(),
    );
    let repair = JsonBoundaryRepairDisplayReportV1 {
        receipt_id: display_only_unstable_id("json-repair"),
        kind: ArtifactKindV1::BoundaryRepair,
        changed: true,
        repair_kind: "json-substring-extracted".into(),
        degraded: true,
        before_raw_digest: Some(non_authoritative_text_display_digest("prefix {\"a\":1}")),
        after_raw_digest: Some(non_authoritative_text_display_digest("{\"a\":1}")),
        before_display_digest: None,
        after_display_digest: Some(DisplayDigestV1::for_json_value(&serde_json::json!({"a":1}))),
        treatment_critical_fields: vec!["a".into()],
        treatment_integrity_warnings: vec!["treatment-integrity-unverifiable:a".into()],
        hard_failed: false,
        warnings: vec!["extracted first JSON object or array before boundary parse".into()],
        reason_codes: vec!["json-repair:json-substring-extracted".into()],
        canonical_repair_record_ids: Vec::new(),
        canonical_backpointers: canonical_owner_backpointer(
            "verification-control",
            "BoundaryRepairRecord",
            "canonical-boundary-repair-owner",
        ),
    };
    let outcome = BoundaryCompileOutcomeV1 {
        outcome_id: display_only_unstable_id("boundary-compile-outcome"),
        request_id: display_only_unstable_id("boundary-compile-request"),
        accepted: false,
        degraded: true,
        value: Some(left.clone()),
        display_digest: Some(digest.clone()),
        duplicate_key_findings: vec![duplicate],
        schema_validation: Some(validation),
        repair_receipt: Some(repair),
        reason_codes: vec!["schema-validation-failed".into()],
        compiled_at: Utc::now(),
    };

    assert_eq!(digest.algorithm, "blake3");
    assert!(digest.non_authoritative);
    assert_eq!(digest, DisplayDigestV1::for_json_value(&right));
    assert!(digest.digest.starts_with("blake3:"));
    assert_eq!(
        outcome.schema_validation.unwrap().kind,
        ArtifactKindV1::SchemaValidation
    );
    assert!(!outcome.accepted);
}

#[test]
fn plan_has_visible_risk_summary() {
    let plan = AiDENsAppPlanV1 {
        app_id: "app".into(),
        profile_id: "coding".into(),
        provider_required: true,
        memory_mode: MemoryModeV1::Optional,
        receipt_level: ReportLevelV1::Full,
        dangerous_auto_approval: false,
        risk_disclosures: vec![RiskDisclosureV1 {
            risk_class: CanonicalToolSideEffectClass::Admin,
            granted_by_default: false,
            permit_required: true,
            reason: "shell requires explicit operator permit".into(),
        }],
        enabled_tool_bundles: vec![],
        disabled_tool_bundles: vec![],
    };
    assert!(plan.human_summary().contains("coding"));
    assert!(plan.risk_summary().contains("granted_by_default=false"));
}

#[test]
fn p00_source_basis_lock_names_current_snapshot() {
    let lock = SourceBasisLockV1::current_20260426();

    assert_eq!(lock.snapshot_date, "2026-04-26");
    assert_eq!(lock.workspace_crates, 31);
    assert_eq!(lock.scaffold_only_files, 15);
    assert!(lock.source_archive.contains("20260426"));
}

#[test]
fn p00_scaffold_report_allows_markers_only_for_scaffold_status() {
    let report = ScaffoldSurfaceReportV1::new(vec![
        CrateSurfaceStatusV1::new(
            "aidens-daemon-kit",
            CrateImplementationStatusV1::ScaffoldOnly,
            1,
            18,
            "deferred to P11",
        ),
        CrateSurfaceStatusV1::new(
            "runner-crate",
            CrateImplementationStatusV1::Partial,
            2,
            306,
            "provider boundary exists; turn executor is later",
        ),
    ]);

    assert_eq!(report.scaffold_only_crates(), vec!["aidens-daemon-kit"]);
    assert!(report.allows_scaffold_marker_for("aidens-daemon-kit"));
    assert!(!report.allows_scaffold_marker_for("runner-crate"));
    assert!(!report.allows_scaffold_marker_for("unknown-crate"));
}

#[test]
fn p00_super_pass_status_blocks_on_fake_ready_findings() {
    let status = SuperPassStatusV1::new(
        "P00",
        SuperPassDispositionV1::InProgress,
        SourceBasisLockV1::current_20260426(),
        ScaffoldSurfaceReportV1::new(Vec::new()),
        vec![FakeReadyFindingV1::blocking(
            "README.md",
            "scaffold surface promoted",
            "P00 must fail closed",
        )],
        Vec::new(),
    );

    assert!(status.is_blocked());
}

#[test]
fn p00_source_basis_golden_fixture_deserializes() {
    let fixture = include_str!("../../../tests/fixtures/p00/source_basis_lock_v1.json");
    let lock: SourceBasisLockV1 = serde_json::from_str(fixture).unwrap();

    assert_eq!(lock.snapshot_date, "2026-04-26");
    assert_eq!(lock.rust_files, 37);
    assert_eq!(lock.approximate_rust_loc, 5126);
}

#[test]
fn p01_golden_fixtures_deserialize() {
    let honesty = include_str!("../../../tests/fixtures/p01/api_honesty_receipt_v1.json");
    let honesty: ApiHonestyReportV1 = serde_json::from_str(honesty).unwrap();
    assert_eq!(honesty.kind, ArtifactKindV1::ApiHonesty);
    assert_eq!(honesty.outcome, ApiHonestyOutcomeV1::Blocked);

    let config_apply = include_str!("../../../tests/fixtures/p01/config_apply_receipt_v1.json");
    let config_apply: ConfigApplyReportV1 = serde_json::from_str(config_apply).unwrap();
    assert_eq!(config_apply.kind, ArtifactKindV1::ConfigApply);
    assert_eq!(config_apply.provider_route.route_label, "mock");

    let parity = include_str!("../../../tests/fixtures/p01/plan_runtime_parity_report_v1.json");
    let parity: PlanRuntimeParityReportV1 = serde_json::from_str(parity).unwrap();
    assert!(parity.is_passing());
}

#[test]
fn p02_golden_fixtures_deserialize() {
    let matrix = include_str!("../../../tests/fixtures/p02/provider_backend_matrix_v1.json");
    let matrix: ProviderBackendMatrixV1 = serde_json::from_str(matrix).unwrap();
    assert!(matrix.entry_for("ollama").is_some());

    let readiness = include_str!("../../../tests/fixtures/p02/provider_readiness_receipt_v1.json");
    let readiness: ProviderReadinessReportV1 = serde_json::from_str(readiness).unwrap();
    assert_eq!(readiness.kind, ArtifactKindV1::ProviderReadiness);
    assert!(!readiness.native_tool_loop_executable);

    let route = include_str!("../../../tests/fixtures/p02/provider_route_receipt_v2.json");
    let route: ProviderRouteReportV2 = serde_json::from_str(route).unwrap();
    assert_eq!(route.route_label, "ollama-chat");
    assert!(!route.native_tool_loop);

    let fixture =
        include_str!("../../../tests/fixtures/p02/provider_certification_fixture_v1.json");
    let fixture: ProviderCertificationFixtureV1 = serde_json::from_str(fixture).unwrap();
    assert!(!fixture.expected_native_tool_loop);
}

#[test]
fn p03_golden_fixtures_deserialize() {
    let plan = include_str!("../../../tests/fixtures/p03/turn_execution_plan_v1.json");
    let plan: TurnExecutionPlanV1 = serde_json::from_str(plan).unwrap();
    assert_eq!(plan.mode, TurnModeV1::ParserFallback);

    let request = include_str!("../../../tests/fixtures/p03/tool_call_request_v1.json");
    let request: ToolCallRequestV1 = serde_json::from_str(request).unwrap();
    assert_eq!(request.source, ToolCallSourceV1::ParserFallback);
    assert!(request.degraded);

    let result = include_str!("../../../tests/fixtures/p03/tool_call_result_v1.json");
    let result: ToolCallResultV1 = serde_json::from_str(result).unwrap();
    assert!(result.succeeded);

    let turn = include_str!("../../../tests/fixtures/p03/turn_receipt_v1.json");
    let turn: TurnReportV1 = serde_json::from_str(turn).unwrap();
    assert_eq!(turn.kind, ArtifactKindV1::Turn);

    let stop = include_str!("../../../tests/fixtures/p03/stop_rule_receipt_v1.json");
    let stop: StopRuleReportV1 = serde_json::from_str(stop).unwrap();
    assert_eq!(stop.rule, StopRuleV1::FinalOutput);

    let budget = include_str!("../../../tests/fixtures/p03/budget_exhaustion_receipt_v1.json");
    let budget: BudgetExhaustionReportV1 = serde_json::from_str(budget).unwrap();
    assert_eq!(budget.kind, ArtifactKindV1::BudgetExhaustion);
}

#[test]
fn p04_golden_fixtures_deserialize() {
    let decision = include_str!("../../../tests/fixtures/p04/capability_gate_decision_v1.json");
    let decision: CapabilityGateDecisionV1 = serde_json::from_str(decision).unwrap();
    assert_eq!(decision.outcome, CapabilityGateOutcomeV1::Blocked);

    let exposure = include_str!("../../../tests/fixtures/p04/tool_exposure_plan_v2.json");
    let exposure: ToolExposurePlanV2 = serde_json::from_str(exposure).unwrap();
    assert_eq!(exposure.blocked_tool_ids, vec!["aidens:file-write:1"]);
    assert_eq!(exposure.approval_requests.len(), 1);

    let request = include_str!("../../../tests/fixtures/p04/approval_request_v1.json");
    let request: ApprovalRequestV1 = serde_json::from_str(request).unwrap();
    assert_eq!(request.sandbox_root, "/repo");

    let decision = include_str!("../../../tests/fixtures/p04/approval_decision_v1.json");
    let decision: ApprovalDecisionV1 = serde_json::from_str(decision).unwrap();
    assert!(decision.approved);

    let grant = include_str!("../../../tests/fixtures/p04/permit_grant_v1.json");
    let grant: PermitGrantV1 = serde_json::from_str(grant).unwrap();
    assert_eq!(grant.tool_id, "aidens:file-write:1");

    let use_receipt = include_str!("../../../tests/fixtures/p04/permit_use_receipt_v1.json");
    let use_receipt: PermitUseReportV1 = serde_json::from_str(use_receipt).unwrap();
    assert_eq!(use_receipt.kind, ArtifactKindV1::PermitUse);
    assert!(use_receipt.allowed);
}

#[test]
fn p05_golden_fixtures_deserialize() {
    let poison = include_str!("../../../tests/fixtures/p05/poison_receipt_record_v1.json");
    let poison: PoisonReportEntryV1 = serde_json::from_str(poison).unwrap();
    assert_eq!(poison.line_number, 1);
    assert!(poison.raw_digest.starts_with("blake3:"));

    let graph = include_str!("../../../tests/fixtures/p05/execution_lineage_graph_v1.json");
    let graph: ExecutionLineageGraphV1 = serde_json::from_str(graph).unwrap();
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn p06_golden_fixtures_deserialize() {
    let request = include_str!("../../../tests/fixtures/p06/boundary_compile_request_v1.json");
    let request: BoundaryCompileRequestV1 = serde_json::from_str(request).unwrap();
    assert_eq!(request.schema_dialect, "json-schema-2020-12-subset");

    let outcome = include_str!("../../../tests/fixtures/p06/boundary_compile_outcome_v1.json");
    let outcome: BoundaryCompileOutcomeV1 = serde_json::from_str(outcome).unwrap();
    assert!(!outcome.accepted);
    assert_eq!(
        outcome.schema_validation.unwrap().kind,
        ArtifactKindV1::SchemaValidation
    );

    let validation = include_str!("../../../tests/fixtures/p06/schema_validation_receipt_v1.json");
    let validation: SchemaValidationReportV1 = serde_json::from_str(validation).unwrap();
    assert!(!validation.valid);

    let repair = include_str!("../../../tests/fixtures/p06/json_repair_receipt_v2.json");
    let repair: JsonBoundaryRepairDisplayReportV1 = serde_json::from_str(repair).unwrap();
    assert!(repair.degraded);
    assert!(!repair.treatment_integrity_warnings.is_empty());

    let duplicate = include_str!("../../../tests/fixtures/p06/duplicate_key_finding_v1.json");
    let duplicate: DuplicateKeyFindingV1 = serde_json::from_str(duplicate).unwrap();
    assert_eq!(duplicate.key, "path");

    let digest = include_str!("../../../tests/fixtures/p06/display_digest_v1.json");
    let digest: DisplayDigestV1 = serde_json::from_str(digest).unwrap();
    assert_eq!(digest.algorithm, "blake3");
    assert!(digest.non_authoritative);
}

#[test]
fn p07_registry_manifest_and_migration_constructors_are_typed() {
    let registry = current_artifact_family_registry();
    let documents = generated_schema_documents();
    let manifest = generated_schema_manifest();
    let plan =
        MigrationPlanV1::expand_backfill_flip_contract("schema-compatibility-report", 1, 1, false);
    let backfill = BackfillReportV1::succeeded(
        &plan,
        1,
        vec!["tests/fixtures/p07/schema_compatibility_report_v1.json".into()],
    );

    assert!(registry.contains_family_version("schema-compatibility-report", 1));
    assert_eq!(registry.families.len(), documents.len());
    assert_eq!(manifest.schemas.len(), documents.len());
    assert!(manifest.registry_digest.starts_with("blake3:"));
    assert_eq!(
        registry.governance_status,
        SchemaRegistryGovernanceStatusV1::LocalDisplayIndexOnly
    );
    assert_eq!(registry.canonical_truth_owner, "contract-schema-gen");
    assert!(registry.families.iter().all(|family| {
        family.admission == SchemaFamilyAdmissionV1::LocalDisplayOnly
            && !family.admission.allows_truth_ownership_claim()
    }));
    assert!(manifest
        .schemas
        .iter()
        .all(|schema| schema.schema_identity.starts_with("schema:")
            && schema.schema_identity.contains("blake3:")));
    assert_eq!(
        plan.phases,
        vec![
            MigrationPhaseV1::Expand,
            MigrationPhaseV1::Backfill,
            MigrationPhaseV1::FlipRead,
            MigrationPhaseV1::Contract,
        ]
    );
    assert!(backfill.succeeded);
    assert_eq!(backfill.kind, ArtifactKindV1::Backfill);
}

#[test]
fn p11_schema_governance_quarantines_external_families_and_classifies_compatibility() {
    let external = ArtifactFamilyRegistrationV1::quarantined_external(
        "external-schema",
        1,
        "external-schema/v1.schema.json",
        "not admitted by canonical owner",
    );
    assert_eq!(
        external.admission,
        SchemaFamilyAdmissionV1::QuarantinedExternal
    );
    assert!(!external.admission.allows_local_generation());
    assert!(!external.admission.allows_truth_ownership_claim());
    assert!(external
        .compatibility_policy
        .contains("not admitted by canonical owner"));

    let exact = SchemaCompatibilityCheckV1::exact(
        "schema-compatibility-report",
        1,
        SchemaCompatibilityModeV1::Full,
    );
    assert_eq!(exact.change_class, SchemaChangeClassV1::Exact);
    assert!(!exact.requires_major_bump);
    assert!(exact.compatible);

    let incompatible = SchemaCompatibilityCheckV1::incompatible(
        "schema-compatibility-report",
        1,
        SchemaCompatibilityModeV1::Full,
        vec!["schema-content-drift-without-major-bump".into()],
    );
    assert_eq!(
        incompatible.change_class,
        SchemaChangeClassV1::UnknownIncompatible
    );
    assert!(incompatible.requires_major_bump);
    assert!(!incompatible.compatible);

    let collision = SchemaPathCollisionFindingV1::new(
        "agent-spec/v1.schema.json",
        vec![
            "agent-spec/v1.schema.json".into(),
            "Agent-Spec/v1.schema.json".into(),
        ],
    );
    let report = SchemaCompatibilityReportV1::new(
        &current_artifact_family_registry(),
        1,
        vec![incompatible],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![collision],
    );
    assert!(!report.compatible);
    assert!(report
        .reason_codes
        .contains(&"schema-path-case-fold-collision".into()));
}

#[test]
fn p07_golden_fixtures_deserialize() {
    let registry = include_str!("../../../tests/fixtures/p07/artifact_family_registry_v1.json");
    let registry: ArtifactFamilyRegistryV1 = serde_json::from_str(registry).unwrap();
    assert!(registry.contains_family_version("schema-compatibility-report", 1));

    let manifest = include_str!("../../../tests/fixtures/p07/generated_schema_manifest_v1.json");
    let manifest: GeneratedSchemaManifestV1 = serde_json::from_str(manifest).unwrap();
    assert_eq!(manifest.manifest_id.0, "generated-schema-manifest:v1");

    let report = include_str!("../../../tests/fixtures/p07/schema_compatibility_report_v1.json");
    let report: SchemaCompatibilityReportV1 = serde_json::from_str(report).unwrap();
    assert!(report.compatible);

    let plan = include_str!("../../../tests/fixtures/p07/migration_plan_v1.json");
    let plan: MigrationPlanV1 = serde_json::from_str(plan).unwrap();
    assert_eq!(plan.phases[0], MigrationPhaseV1::Expand);

    let backfill = include_str!("../../../tests/fixtures/p07/backfill_receipt_v1.json");
    let backfill: BackfillReportV1 = serde_json::from_str(backfill).unwrap();
    assert_eq!(backfill.kind, ArtifactKindV1::Backfill);
    assert!(backfill.succeeded);
}

#[test]
fn p07_migration_path_keeps_old_fixtures_readable() {
    let fixture_paths = vec![
        "tests/fixtures/p00/source_basis_lock_v1.json",
        "tests/fixtures/p00/scaffold_surface_report_v1.json",
        "tests/fixtures/p00/fake_ready_finding_v1.json",
        "tests/fixtures/p00/super_pass_status_v1.json",
        "tests/fixtures/p01/api_honesty_receipt_v1.json",
        "tests/fixtures/p01/config_apply_receipt_v1.json",
        "tests/fixtures/p01/plan_runtime_parity_report_v1.json",
        "tests/fixtures/p02/provider_backend_matrix_v1.json",
        "tests/fixtures/p02/provider_readiness_receipt_v1.json",
        "tests/fixtures/p02/provider_route_receipt_v2.json",
        "tests/fixtures/p02/provider_certification_fixture_v1.json",
        "tests/fixtures/p03/turn_execution_plan_v1.json",
        "tests/fixtures/p03/tool_call_request_v1.json",
        "tests/fixtures/p03/tool_call_result_v1.json",
        "tests/fixtures/p03/turn_receipt_v1.json",
        "tests/fixtures/p03/tool_invocation_receipt_v1.json",
        "tests/fixtures/p03/stop_rule_receipt_v1.json",
        "tests/fixtures/p03/budget_exhaustion_receipt_v1.json",
        "tests/fixtures/p04/capability_gate_decision_v1.json",
        "tests/fixtures/p04/tool_exposure_plan_v2.json",
        "tests/fixtures/p04/approval_request_v1.json",
        "tests/fixtures/p04/approval_decision_v1.json",
        "tests/fixtures/p04/permit_grant_v1.json",
        "tests/fixtures/p04/permit_use_receipt_v1.json",
        "tests/fixtures/p05/execution_context_v1.json",
        "tests/fixtures/p05/run_receipt_v1.json",
        "tests/fixtures/p05/poison_receipt_record_v1.json",
        "tests/fixtures/p05/execution_lineage_graph_v1.json",
        "tests/fixtures/p06/boundary_compile_request_v1.json",
        "tests/fixtures/p06/boundary_compile_outcome_v1.json",
        "tests/fixtures/p06/schema_validation_receipt_v1.json",
        "tests/fixtures/p06/json_repair_receipt_v2.json",
        "tests/fixtures/p06/duplicate_key_finding_v1.json",
        "tests/fixtures/p06/display_digest_v1.json",
    ];

    let _: SourceBasisLockV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p00/source_basis_lock_v1.json"
    ))
    .unwrap();
    let _: ScaffoldSurfaceReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p00/scaffold_surface_report_v1.json"
    ))
    .unwrap();
    let _: FakeReadyFindingV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p00/fake_ready_finding_v1.json"
    ))
    .unwrap();
    let _: SuperPassStatusV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p00/super_pass_status_v1.json"
    ))
    .unwrap();
    let _: ApiHonestyReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p01/api_honesty_receipt_v1.json"
    ))
    .unwrap();
    let _: ConfigApplyReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p01/config_apply_receipt_v1.json"
    ))
    .unwrap();
    let _: PlanRuntimeParityReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p01/plan_runtime_parity_report_v1.json"
    ))
    .unwrap();
    let _: ProviderBackendMatrixV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p02/provider_backend_matrix_v1.json"
    ))
    .unwrap();
    let _: ProviderReadinessReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p02/provider_readiness_receipt_v1.json"
    ))
    .unwrap();
    let _: ProviderRouteReportV2 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p02/provider_route_receipt_v2.json"
    ))
    .unwrap();
    let _: ProviderCertificationFixtureV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p02/provider_certification_fixture_v1.json"
    ))
    .unwrap();
    let _: TurnExecutionPlanV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p03/turn_execution_plan_v1.json"
    ))
    .unwrap();
    let _: ToolCallRequestV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p03/tool_call_request_v1.json"
    ))
    .unwrap();
    let _: ToolCallResultV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p03/tool_call_result_v1.json"
    ))
    .unwrap();
    let _: TurnReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p03/turn_receipt_v1.json"
    ))
    .unwrap();
    let _: ToolInvocationReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p03/tool_invocation_receipt_v1.json"
    ))
    .unwrap();
    let _: StopRuleReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p03/stop_rule_receipt_v1.json"
    ))
    .unwrap();
    let _: BudgetExhaustionReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p03/budget_exhaustion_receipt_v1.json"
    ))
    .unwrap();
    let _: CapabilityGateDecisionV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p04/capability_gate_decision_v1.json"
    ))
    .unwrap();
    let _: ToolExposurePlanV2 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p04/tool_exposure_plan_v2.json"
    ))
    .unwrap();
    let _: ApprovalRequestV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p04/approval_request_v1.json"
    ))
    .unwrap();
    let _: ApprovalDecisionV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p04/approval_decision_v1.json"
    ))
    .unwrap();
    let _: PermitGrantV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p04/permit_grant_v1.json"
    ))
    .unwrap();
    let _: PermitUseReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p04/permit_use_receipt_v1.json"
    ))
    .unwrap();
    let _: AidensRunContextV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p05/execution_context_v1.json"
    ))
    .unwrap();
    let _: RunReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p05/run_receipt_v1.json"
    ))
    .unwrap();
    let _: PoisonReportEntryV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p05/poison_receipt_record_v1.json"
    ))
    .unwrap();
    let _: ExecutionLineageGraphV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p05/execution_lineage_graph_v1.json"
    ))
    .unwrap();
    let _: BoundaryCompileRequestV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p06/boundary_compile_request_v1.json"
    ))
    .unwrap();
    let _: BoundaryCompileOutcomeV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p06/boundary_compile_outcome_v1.json"
    ))
    .unwrap();
    let _: SchemaValidationReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p06/schema_validation_receipt_v1.json"
    ))
    .unwrap();
    let _: JsonBoundaryRepairDisplayReportV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p06/json_repair_receipt_v2.json"
    ))
    .unwrap();
    let _: DuplicateKeyFindingV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p06/duplicate_key_finding_v1.json"
    ))
    .unwrap();
    let _: DisplayDigestV1 = serde_json::from_str(include_str!(
        "../../../tests/fixtures/p06/display_digest_v1.json"
    ))
    .unwrap();

    let plan = MigrationPlanV1::expand_backfill_flip_contract("historical-fixtures", 1, 1, false);
    let backfill = BackfillReportV1::succeeded(
        &plan,
        fixture_paths.len(),
        fixture_paths.into_iter().map(str::to_string).collect(),
    );

    assert!(backfill.succeeded);
    assert_eq!(backfill.migrated_fixture_count, 34);
}

#[test]
fn p24_run_bundle_v2_fixture_deserializes_with_canonical_context() {
    let fixture = include_str!("../../../tests/fixtures/p24/aidens_run_bundle_v2.json");
    let bundle: AiDENsRunBundleV2 = serde_json::from_str(fixture).unwrap();

    assert_eq!(bundle.schema, AiDENsRunBundleV2::SCHEMA);
    assert_eq!(
        bundle.canonical_execution_context.trace_ctx,
        bundle.trace_ctx
    );
    assert_eq!(
        bundle
            .canonical_execution_context
            .attempt_id
            .as_ref()
            .unwrap(),
        &bundle.attempt_id
    );
    assert_eq!(bundle.event_log.digest.hex().len(), 64);
    assert!(bundle
        .canonical_backpointers
        .iter()
        .any(|backpointer| backpointer.owner_crate == "semantic-memory-forge"));
}

#[test]
fn p26_run_bundle_v3_fixture_deserializes_with_evidence_fields() {
    let fixture = include_str!("../../../tests/fixtures/p26/aidens_run_bundle_v3.json");
    let bundle: AiDENsRunBundleV3 = serde_json::from_str(fixture).unwrap();

    assert_eq!(bundle.schema, AiDENsRunBundleV3::SCHEMA);
    assert_eq!(
        bundle.canonical_execution_context.trace_ctx,
        bundle.trace_ctx
    );
    assert_eq!(
        bundle.attempt_id,
        bundle
            .canonical_execution_context
            .attempt_id
            .as_ref()
            .expect("fixture execution context must include attempt_id")
            .clone()
    );
    assert_eq!(bundle.canonical_backpointers.len(), 6);
    assert_eq!(bundle.support_labels.len(), 2);
    assert_eq!(bundle.memory_grounding_receipts.len(), 1);
    assert_eq!(bundle.verification_receipts.len(), 2);
    assert!(bundle.agent_spec_digest.non_authoritative);
    assert_eq!(bundle.replay_instructions.len(), 2);
    assert_eq!(bundle.repair_plan_receipts.len(), 1);
    assert_eq!(bundle.abstention_receipts.len(), 1);
}

#[test]
fn p26_agent_spec_v1_fixture_is_valid() {
    let fixture = include_str!("../../../tests/fixtures/p26/agent_spec_v1.json");
    let spec: AgentSpecV1 = serde_json::from_str(fixture).unwrap();

    assert_eq!(spec.schema, AgentSpecV1::SCHEMA);
    assert_eq!(spec.support_label.to_string(), "supported-local");
    assert_eq!(spec.profile, "coding");
    assert!(spec.validate().is_ok());
}

#[test]
fn p26_agent_spec_v1_fixture_rejects_invalid_policy() {
    let fixture = include_str!("../../../tests/fixtures/p26/agent_spec_v1_invalid.json");
    let spec: AgentSpecV1 = serde_json::from_str(fixture).unwrap();
    let reasons = spec.validate().expect_err("invalid policy should fail");

    assert!(reasons.contains(&"agent-id-required".into()));
    assert!(reasons.contains(&"display-name-required".into()));
    assert!(reasons.contains(&"unsupported-profile".into()));
    assert!(reasons.contains(&"cloud-providers-unsupported-for-p26-local-agent".into()));
    assert!(reasons.contains(&"unsupported-tool:repo.write".into()));
    assert!(reasons.contains(&"write-tools-must-require-permit".into()));
    assert!(reasons.contains(&"write-permit-policy-must-be-operator-approved".into()));
    assert!(reasons.contains(&"network-access-must-be-forbidden-for-local-run".into()));
    assert!(reasons.contains(&"canonical-memory-grounding-must-disclose-or-be-disabled".into(),));
    assert!(reasons.contains(&"verification-required-checks-missing".into()));
    assert!(reasons.contains(&"verification-fail-closed-required".into()));
    assert!(reasons.contains(&"must-emit-tool-receipts".into()));
    assert!(reasons.contains(&"must-emit-permit-receipts".into()));
    assert!(reasons.contains(&"must-emit-abstention-receipts".into()));
    assert!(reasons.contains(&"must-emit-run-bundle".into()));
    assert!(reasons.contains(&"max-turns-must-be-positive".into()));
    assert!(reasons.contains(&"max-tool-calls-must-be-positive".into()));
    assert!(reasons.contains(&"deadline-seconds-must-be-positive".into()));
}

#[test]
fn p08_reference_artifact_constructors_are_typed() {
    let case = ReferenceCaseV1::new(
        ReferenceDomainV1::Permit,
        "read-only permit default",
        serde_json::json!({"risk_class":"read_only","matching_permit":false}),
        serde_json::json!({"decision":"allow","permit_required":false}),
    )
    .with_risk_class(CanonicalToolSideEffectClass::ReadOnly)
    .with_memory_mode(MemoryModeV1::Disabled)
    .with_receipt_level(ReportLevelV1::Standard)
    .with_tool_lifecycle_state(ToolLifecycleStateV1::Declared)
    .with_source_fixture("tests/fixtures/reference/reference_case_v1.json");

    let finding = DifferentialConformanceFindingV1::mismatch(
        &case,
        "unit-test",
        "$.decision",
        serde_json::json!("allow"),
        serde_json::json!("requires-approval"),
    );
    let report = ReferenceInterpreterReportV1::new(
        "reference:test",
        1,
        vec![finding],
        std::slice::from_ref(&case),
    );
    let manifest = GoldenFixtureManifestV1::new(
        vec!["tests/fixtures/reference/reference_case_v1.json".into()],
        &[case],
    );

    assert_eq!(
        report.report_id.0.split(':').next(),
        Some("reference-interpreter-report")
    );
    assert!(!report.passed);
    assert!(report
        .reason_codes
        .contains(&"reference-conformance-failed".into()));
    assert!(manifest
        .risk_classes
        .contains(&CanonicalToolSideEffectClass::ReadOnly));
}

#[test]
fn p08_golden_fixtures_deserialize() {
    let case = include_str!("../../../tests/fixtures/reference/reference_case_v1.json");
    let case: ReferenceCaseV1 = serde_json::from_str(case).unwrap();
    assert_eq!(case.domain, ReferenceDomainV1::ProviderRoute);

    let report =
        include_str!("../../../tests/fixtures/reference/reference_interpreter_report_v1.json");
    let report: ReferenceInterpreterReportV1 = serde_json::from_str(report).unwrap();
    assert!(report.passed);

    let finding =
        include_str!("../../../tests/fixtures/reference/differential_conformance_finding_v1.json");
    let finding: DifferentialConformanceFindingV1 = serde_json::from_str(finding).unwrap();
    assert_eq!(finding.domain, ReferenceDomainV1::Permit);

    let manifest =
        include_str!("../../../tests/fixtures/reference/golden_fixture_manifest_v1.json");
    let manifest: GoldenFixtureManifestV1 = serde_json::from_str(manifest).unwrap();
    assert!(manifest.provider_kinds.contains(&"openai".into()));
    assert!(manifest
        .risk_classes
        .contains(&CanonicalToolSideEffectClass::Admin));
    assert!(manifest.memory_modes.contains(&MemoryModeV1::Required));
    assert!(manifest.receipt_levels.contains(&ReportLevelV1::Full));
    assert!(manifest
        .tool_lifecycle_states
        .contains(&ToolLifecycleStateV1::Blocked));
}

#[test]
fn p10_coding_artifact_constructors_are_receipt_bearing_and_sandboxed() {
    let read = RepoReadReportV1::allowed("/repo", "README.md", "README.md", 5, "hello");
    let list = RepoListReportV1::allowed(
        "/repo",
        ".",
        vec![RepoListEntryV1 {
            path: "README.md".into(),
            entry_kind: "file".into(),
            bytes: Some(5),
        }],
    );
    let proposal = PatchProposalV1::new(
        "update readme",
        "--- a/README.md\n+++ b/README.md\n@@\n-hello\n+hello world\n",
        vec!["README.md".into()],
    );
    let patch_input = serde_json::json!({"diff": proposal.unified_diff});
    let apply = PatchApplyReportV1::new(
        "/repo",
        &patch_input,
        vec!["README.md".into()],
        BTreeMap::new(),
        BTreeMap::new(),
        Some(ArtifactId("permit:fixture".into())),
        Some(ArtifactId("permit-use:fixture".into())),
    );
    let command = CommandRunReportV1::completed(
        "/repo",
        vec!["cargo".into(), "check".into(), "--workspace".into()],
        Some(ArtifactId("permit:fixture".into())),
        Some(ArtifactId("permit-use:fixture".into())),
        Some(0),
        "ok",
        "",
    );
    let sandbox = SandboxCapabilityTruthV1::coding_default("/repo");
    let packet = CodexPacketV1::new(CodexPacketInputV1 {
        current_pass: "P10".into(),
        next_pass: "P11".into(),
        issue: "coding tools".into(),
        source_map: vec!["crates/aidens-tool-kit/src/lib.rs".into()],
        changed_files: vec!["crates/aidens-tool-kit/src/lib.rs".into()],
        commands_run: vec![command.clone()],
        receipt_ids: vec![read.receipt_id.clone(), command.receipt_id.clone()],
        blockers: Vec::new(),
        notes: vec!["resume from P11".into()],
    });

    assert_eq!(read.kind, ArtifactKindV1::RepoRead);
    assert_eq!(list.kind, ArtifactKindV1::RepoList);
    assert_eq!(proposal.kind, ArtifactKindV1::PatchProposal);
    assert!(!proposal.mutates_files);
    assert_eq!(apply.kind, ArtifactKindV1::PatchApply);
    assert!(apply.applied);
    assert_eq!(command.kind, ArtifactKindV1::CommandRun);
    assert!(command.succeeded);
    assert_eq!(sandbox.kind, ArtifactKindV1::SandboxTruth);
    assert!(sandbox.write_requires_permit);
    assert!(sandbox.shell_requires_permit);
    assert_eq!(packet.kind, ArtifactKindV1::CodexPacket);
    assert!(packet.has_resume_context());
}

#[test]
fn p10_golden_fixtures_deserialize() {
    let read = include_str!("../../../tests/fixtures/p10/repo_read_receipt_v1.json");
    let read: RepoReadReportV1 = serde_json::from_str(read).unwrap();
    assert_eq!(read.kind, ArtifactKindV1::RepoRead);

    let list = include_str!("../../../tests/fixtures/p10/repo_list_receipt_v1.json");
    let list: RepoListReportV1 = serde_json::from_str(list).unwrap();
    assert_eq!(list.kind, ArtifactKindV1::RepoList);

    let proposal = include_str!("../../../tests/fixtures/p10/patch_proposal_v1.json");
    let proposal: PatchProposalV1 = serde_json::from_str(proposal).unwrap();
    assert!(!proposal.mutates_files);

    let apply = include_str!("../../../tests/fixtures/p10/patch_apply_receipt_v1.json");
    let apply: PatchApplyReportV1 = serde_json::from_str(apply).unwrap();
    assert_eq!(apply.kind, ArtifactKindV1::PatchApply);

    let command = include_str!("../../../tests/fixtures/p10/command_run_receipt_v1.json");
    let command: CommandRunReportV1 = serde_json::from_str(command).unwrap();
    assert_eq!(command.kind, ArtifactKindV1::CommandRun);

    let sandbox = include_str!("../../../tests/fixtures/p10/sandbox_capability_truth_v1.json");
    let sandbox: SandboxCapabilityTruthV1 = serde_json::from_str(sandbox).unwrap();
    assert_eq!(sandbox.network_policy, "disabled");

    let packet = include_str!("../../../tests/fixtures/p10/codex_packet_v1.json");
    let packet: CodexPacketV1 = serde_json::from_str(packet).unwrap();
    assert!(packet.has_resume_context());
}

#[test]
fn p11_queue_daemon_artifact_constructors_preserve_idempotency_and_safe_mode() {
    let namespace = DaemonNamespaceV1::new("p11", "/tmp/aidens-p11", "daemon-a");
    let occurrence = ScheduleOccurrenceV1::new(
        namespace.namespace_id.clone(),
        "once",
        "occurrence-1",
        Utc::now(),
        serde_json::json!({"task":"queue"}),
        CanonicalToolSideEffectClass::ReadOnly,
    );
    let signal = WakeSignalV1::new(
        namespace.namespace_id.clone(),
        "filesystem",
        "README.md",
        serde_json::json!({"path":"README.md"}),
        CanonicalToolSideEffectClass::ReadOnly,
    );
    let job = JobV1::new(
        namespace.namespace_id.clone(),
        occurrence.idempotency_key.clone(),
        "schedule",
        occurrence.payload.clone(),
        CanonicalToolSideEffectClass::ReadOnly,
        Some(occurrence.occurrence_id.clone()),
        None,
    );
    let lease = QueueLeaseV1::new(&job, "daemon-a", 30, None);
    let safe = SafeModeReportV1::new(
        namespace.namespace_id.clone(),
        SafeModeOperationV1::Entered,
        true,
        None,
        "operator-safe-mode",
    );
    let duplicate = DuplicateSuppressionReportV1::new(
        namespace.namespace_id.clone(),
        job.idempotency_key.clone(),
        job.job_id.clone(),
        "schedule",
    );
    let hop = QueueHopReportV1::new(
        namespace.namespace_id.clone(),
        job.job_id.clone(),
        Some(lease.lease_id.clone()),
        QueueHopKindV1::LeaseAcquired,
        Some(JobStateV1::Queued),
        JobStateV1::Leased,
        "lease-acquired",
    );

    assert_eq!(namespace.kind, ArtifactKindV1::DaemonNamespace);
    assert!(occurrence.identity_is_not_timestamp_only());
    assert!(signal.idempotency_key.contains("filesystem"));
    assert_eq!(job.kind, ArtifactKindV1::Job);
    assert!(job.attempt_family_id.0.starts_with("attempt-family:"));
    assert_eq!(lease.kind, ArtifactKindV1::QueueLease);
    let expected_lease_material = format!(
        "{}|{}|{}|{}",
        job.namespace_id.0,
        job.job_id.0,
        "daemon-a",
        lease
            .acquired_at
            .to_rfc3339_opts(SecondsFormat::Nanos, true)
    );
    assert_eq!(
        lease.lease_id,
        local_artifact_id_from_stack_digest("queue-lease", &expected_lease_material)
    );
    let fallback_material = format!("{}|{}|{}|0", job.namespace_id.0, job.job_id.0, "daemon-a");
    assert_ne!(
        lease.lease_id,
        local_artifact_id_from_stack_digest("queue-lease", &fallback_material)
    );
    assert!(safe.blocks_new_risky_jobs_but_allows_drain());
    assert_eq!(duplicate.kind, ArtifactKindV1::DuplicateSuppression);
    assert_eq!(hop.kind, ArtifactKindV1::QueueHop);
}

#[test]
fn p11_golden_fixtures_deserialize() {
    let namespace = include_str!("../../../tests/fixtures/p11/daemon_namespace_v1.json");
    let namespace: DaemonNamespaceV1 = serde_json::from_str(namespace).unwrap();
    assert_eq!(namespace.kind, ArtifactKindV1::DaemonNamespace);

    let occurrence = include_str!("../../../tests/fixtures/p11/schedule_occurrence_v1.json");
    let occurrence: ScheduleOccurrenceV1 = serde_json::from_str(occurrence).unwrap();
    assert!(occurrence.identity_is_not_timestamp_only());

    let signal = include_str!("../../../tests/fixtures/p11/wake_signal_v1.json");
    let signal: WakeSignalV1 = serde_json::from_str(signal).unwrap();
    assert_eq!(signal.kind, ArtifactKindV1::WakeSignal);

    let job = include_str!("../../../tests/fixtures/p11/job_v1.json");
    let job: JobV1 = serde_json::from_str(job).unwrap();
    assert_eq!(job.kind, ArtifactKindV1::Job);

    let lease = include_str!("../../../tests/fixtures/p11/queue_lease_v1.json");
    let lease: QueueLeaseV1 = serde_json::from_str(lease).unwrap();
    assert_eq!(lease.kind, ArtifactKindV1::QueueLease);

    let safe = include_str!("../../../tests/fixtures/p11/safe_mode_receipt_v1.json");
    let safe: SafeModeReportV1 = serde_json::from_str(safe).unwrap();
    assert!(safe.blocks_new_risky_jobs_but_allows_drain());

    let duplicate =
        include_str!("../../../tests/fixtures/p11/duplicate_suppression_receipt_v1.json");
    let duplicate: DuplicateSuppressionReportV1 = serde_json::from_str(duplicate).unwrap();
    assert_eq!(duplicate.kind, ArtifactKindV1::DuplicateSuppression);

    let hop = include_str!("../../../tests/fixtures/p11/queue_hop_receipt_v1.json");
    let hop: QueueHopReportV1 = serde_json::from_str(hop).unwrap();
    assert_eq!(hop.kind, ArtifactKindV1::QueueHop);
}

#[test]
fn p13_view_artifact_constructors_disclose_policy_events() {
    let valid_at = "2026-04-27T00:00:00Z".parse().unwrap();
    let recorded_at = "2026-04-27T01:00:00Z".parse().unwrap();
    let policy = RetrievalPolicyV1::time_scoped(RuntimeViewModeV1::Temporal, valid_at, recorded_at)
        .with_alias_expansion()
        .with_scope("memory.claims");
    let request = RuntimeViewRequestV1::new("repo status", policy.clone())
        .subject("repo")
        .predicate("status")
        .with_alias("repository");
    let widening = QueryWideningReportV1::alias_expansion(
        &request,
        vec!["repo".into()],
        vec!["repo".into(), "repository".into()],
    );
    let degradation = DegradationEventV1::new(
        &request,
        "time-scoped-query-no-results-no-timeless-fallback",
        false,
        false,
        Vec::new(),
    );
    let projection = ProjectionDigestV1::new(
        RuntimeViewModeV1::Temporal,
        policy.policy_id.clone(),
        vec![ArtifactId("episode:p13".into())],
        vec![ArtifactId("claim:p13".into())],
        vec![ArtifactId("evidence:p13".into())],
        serde_json::json!({
            "policy_id": policy.policy_id,
            "claims": ["claim:p13"],
            "evidence": ["evidence:p13"],
        }),
    );
    let disclosure = ViewDisclosureReportV1::new(
        &request,
        projection.clone(),
        vec![ArtifactId("claim:p13".into())],
        vec![widening.receipt_id.clone()],
        vec![degradation.event_id.clone()],
    );

    assert_eq!(policy.kind, ArtifactKindV1::RetrievalPolicy);
    assert!(policy.forbids_silent_timeless_fallback());
    assert_eq!(request.kind, ArtifactKindV1::RuntimeViewRequest);
    assert!(request.is_time_scoped());
    assert_eq!(widening.kind, ArtifactKindV1::QueryWidening);
    assert!(widening.is_alias_expansion());
    assert!(widening.allowed_by_policy);
    assert_eq!(degradation.kind, ArtifactKindV1::DegradationEvent);
    assert!(degradation.proves_no_silent_timeless_fallback());
    assert_eq!(projection.kind, ArtifactKindV1::ProjectionDigest);
    assert!(projection.rebuilt_from_authoritative_memory);
    assert_eq!(disclosure.kind, ArtifactKindV1::ViewDisclosure);
    assert!(disclosure.discloses_policy_events());
    assert!(disclosure.has_visible_widening_or_degradation());
}

#[test]
fn p13_golden_fixtures_deserialize() {
    let policy = include_str!("../../../tests/fixtures/p13/retrieval_policy_v1.json");
    let policy: RetrievalPolicyV1 = serde_json::from_str(policy).unwrap();
    assert_eq!(policy.kind, ArtifactKindV1::RetrievalPolicy);
    assert!(policy.forbids_silent_timeless_fallback());

    let request = include_str!("../../../tests/fixtures/p13/runtime_view_request_v1.json");
    let request: RuntimeViewRequestV1 = serde_json::from_str(request).unwrap();
    assert_eq!(request.kind, ArtifactKindV1::RuntimeViewRequest);
    assert_eq!(request.view_mode, RuntimeViewModeV1::Temporal);

    let widening = include_str!("../../../tests/fixtures/p13/query_widening_receipt_v1.json");
    let widening: QueryWideningReportV1 = serde_json::from_str(widening).unwrap();
    assert!(widening.is_alias_expansion());
    assert!(widening.allowed_by_policy);

    let degradation = include_str!("../../../tests/fixtures/p13/degradation_event_v1.json");
    let degradation: DegradationEventV1 = serde_json::from_str(degradation).unwrap();
    assert!(degradation.proves_no_silent_timeless_fallback());

    let projection = include_str!("../../../tests/fixtures/p13/projection_digest_v1.json");
    let projection: ProjectionDigestV1 = serde_json::from_str(projection).unwrap();
    assert_eq!(projection.kind, ArtifactKindV1::ProjectionDigest);

    let disclosure = include_str!("../../../tests/fixtures/p13/view_disclosure_receipt_v1.json");
    let disclosure: ViewDisclosureReportV1 = serde_json::from_str(disclosure).unwrap();
    assert_eq!(disclosure.kind, ArtifactKindV1::ViewDisclosure);
    assert!(disclosure.discloses_policy_events());
}

#[test]
fn p14_release_artifact_constructors_block_on_false_public_docs() {
    let example = ExampleAppEntryV1::new(
        "examples/aidens.mock.toml",
        "coding-agent",
        "mock",
        MemoryModeV1::Disabled,
        ReleaseSurfaceStateV1::Supported,
    )
    .with_command("aidens run --config examples/aidens.mock.toml hello");
    let manifest = ExampleAppManifestV1::new(vec![example], vec!["aidens-kernel-kit".into()]);
    let smoke = InstallSmokeReportV1::new(vec![InstallSmokeStepV1::passed(
        "provider-check",
        "aidens provider-check --config examples/aidens.mock.toml",
        "provider-check ok",
    )]);
    let surfaces = vec![
        ReleaseSurfaceV1::new(
            "cli:provider-check",
            ReleaseSurfaceStateV1::Supported,
            "provider-check exposes executable provider truth",
        )
        .with_command("aidens provider-check --config examples/aidens.mock.toml"),
        ReleaseSurfaceV1::new(
            "crate:aidens-kernel-kit",
            ReleaseSurfaceStateV1::Deferred,
            "P15 horizon surface",
        ),
    ];
    let ready = ReleaseReadinessReportV1::new(
        surfaces.clone(),
        Vec::new(),
        manifest.clone(),
        smoke.clone(),
    );
    assert_eq!(manifest.kind, ArtifactKindV1::ExampleAppManifest);
    let blocked = ReleaseReadinessReportV1::new(
        surfaces,
        vec![PublicDocFindingV1::scaffold_claim(
            "README.md",
            7,
            "crate:aidens-kernel-kit",
            "aidens-kernel-kit is complete",
        )],
        manifest,
        smoke,
    );

    assert!(ready.example_manifest.covers_profile("coding-agent"));
    assert_eq!(ready.kind, ArtifactKindV1::ReleaseReadinessReport);
    assert!(!ready.blocks_release());
    assert!(blocked.blocks_release());
    assert_eq!(blocked.public_doc_findings.len(), 1);
}

#[test]
fn p14_operator_status_exposes_degraded_modes() {
    let mut sections = BTreeMap::new();
    sections.insert(
        "provider".into(),
        vec![RuntimeCapabilityTruthV1 {
            capability_id: "provider:openai".into(),
            states: vec![
                CapabilityStateV1::Configured,
                CapabilityStateV1::Unavailable,
            ],
            reason: Some("provider-boundary-unavailable".into()),
        }],
    );
    let doctor = AiDENsDoctorReportV1::new("p14-operator", sections);
    let status = OperatorStatusReportV1::new(
        "p14-operator",
        "loaded examples/aidens.openai-unavailable.toml",
        "unavailable",
        MemoryModeV1::Disabled,
        true,
        doctor,
    );

    assert_eq!(status.kind, ArtifactKindV1::OperatorStatusReport);
    assert!(status.exposes_degraded_modes());
    assert!(status.blocked_modes.contains(&"provider:openai".into()));
}

#[test]
fn p14_golden_fixtures_deserialize() {
    let manifest = include_str!("../../../tests/fixtures/p14/example_app_manifest_v1.json");
    let manifest: ExampleAppManifestV1 = serde_json::from_str(manifest).unwrap();
    assert_eq!(manifest.kind, ArtifactKindV1::ExampleAppManifest);
    assert!(manifest.covers_profile("coding-agent"));

    let smoke = include_str!("../../../tests/fixtures/p14/install_smoke_receipt_v1.json");
    let smoke: InstallSmokeReportV1 = serde_json::from_str(smoke).unwrap();
    assert_eq!(smoke.kind, ArtifactKindV1::InstallSmokeReport);
    assert!(smoke.passed);

    let status = include_str!("../../../tests/fixtures/p14/operator_status_report_v1.json");
    let status: OperatorStatusReportV1 = serde_json::from_str(status).unwrap();
    assert_eq!(status.kind, ArtifactKindV1::OperatorStatusReport);

    let readiness = include_str!("../../../tests/fixtures/p14/release_readiness_report_v1.json");
    let readiness: ReleaseReadinessReportV1 = serde_json::from_str(readiness).unwrap();
    assert_eq!(readiness.kind, ArtifactKindV1::ReleaseReadinessReport);
    assert!(!readiness.blocks_release());
}

#[test]
fn p15_kernel_artifact_constructors_preserve_right_graph_and_stop_rules() {
    let graph_id = ArtifactId("compiled-region-graph:test".into());
    let claim_a = ArtifactId("claim:a".into());
    let claim_b = ArtifactId("claim:b".into());
    let node_a = RegionNodeV1::new(
        RegionNodeKindV1::Claim,
        "repo/status/a",
        Some(claim_a.clone()),
    );
    let node_b = RegionNodeV1::new(
        RegionNodeKindV1::Claim,
        "repo/status/b",
        Some(claim_b.clone()),
    );
    let edge = RegionEdgeV1::new(
        node_a.node_id.clone(),
        node_b.node_id.clone(),
        "same-subject-predicate",
    );
    let factor = RegionFactorV1::new(
        vec![node_a.node_id.clone(), node_b.node_id.clone()],
        "claim-consistency",
        1.0,
    );
    let region_id = ArtifactId("region:test".into());
    let contract = RegionContractV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Inference,
        region_id.clone(),
        vec![node_a.node_id.clone(), node_b.node_id.clone()],
        vec![node_a.node_id.clone()],
        vec![factor.factor_id.clone()],
        4,
    );
    let graph = CompiledRegionGraphV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Inference,
        Some(RegionGraphKindV1::Storage),
        4,
        vec![node_a, node_b],
        vec![edge],
        Vec::new(),
        vec![factor],
        Vec::new(),
        vec![contract.boundary_node_ids[0].clone()],
        vec![contract.clone()],
        vec![claim_a.clone(), claim_b.clone()],
        Vec::new(),
    );
    let stop =
        KernelStopRuleReportV1::new(CanonicalKernelStopReason::FixedPoint, 3, 8, 0.05, 0.5, 0.01);
    let residual = KernelResidualReportV1::new(
        graph_id.clone(),
        region_id.clone(),
        3,
        0.07,
        0.01,
        0.05,
        stop.clone(),
    );
    let report = ConvergenceReportV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Inference,
        CanonicalKernelStopReason::FixedPoint,
        3,
        8,
        0.05,
        0.5,
        0.01,
        vec![residual.residual_id.clone()],
        stop,
        false,
    );
    let syndrome = KernelSyndromeReportV1::contradiction(
        graph_id.clone(),
        region_id.clone(),
        ArtifactId("contradiction-witness:test".into()),
        vec![claim_a, claim_b],
    );
    let digest = DisplayDigestV1::for_json_value(&serde_json::json!({
        "region": "test",
        "claims": 2
    }));
    let oracle = OracleSliceRequestV1::new(
        graph_id,
        region_id,
        contract.node_ids.clone(),
        4,
        digest.clone(),
        digest,
        None,
    );
    let run = KernelRunDisplayReportV1::new(
        &graph,
        &report,
        std::slice::from_ref(&syndrome),
        std::slice::from_ref(&oracle),
    );

    assert_eq!(graph.kind, ArtifactKindV1::CompiledRegionGraph);
    assert!(graph.is_bounded_region_graph());
    assert_eq!(contract.kind, ArtifactKindV1::RegionContract);
    assert!(contract.is_bounded());
    assert_eq!(residual.kind, ArtifactKindV1::Residual);
    assert!(report.converged);
    assert!(report.has_explicit_stop_rule_evidence());
    assert!(!report.degraded);
    assert!(report.converged);
    assert!(syndrome.requires_canonical_repair());
    assert_eq!(oracle.agreement, OracleAgreementV1::Agrees);
    assert_eq!(run.kind, ArtifactKindV1::KernelRunReport);
    assert!(run.convergence_is_evidenced());
    assert!(run.used_global_recompute);
}

#[test]
fn p28_convergence_degrades_when_residual_or_oscillation_breaks_exactness() {
    let graph_id = ArtifactId("region-graph:p28".into());
    let stop =
        KernelStopRuleReportV1::new(CanonicalKernelStopReason::FixedPoint, 3, 8, 0.05, 0.5, 0.09);
    let residual_degraded = ConvergenceReportV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Inference,
        CanonicalKernelStopReason::FixedPoint,
        3,
        8,
        0.05,
        0.5,
        0.09,
        Vec::new(),
        stop.clone(),
        false,
    );
    assert!(residual_degraded.converged);
    assert!(residual_degraded.degraded);
    assert!(residual_degraded
        .reason_codes
        .contains(&"kernel-residual-above-threshold".into()));

    let oscillating = ConvergenceReportV1::new(
        graph_id,
        RegionGraphKindV1::Inference,
        CanonicalKernelStopReason::FixedPoint,
        3,
        8,
        0.05,
        0.5,
        0.01,
        Vec::new(),
        stop,
        true,
    );
    assert!(oscillating.converged);
    assert!(oscillating.degraded);
    assert!(oscillating
        .reason_codes
        .contains(&"kernel-oscillation-detected".into()));
}

#[test]
fn p15_golden_fixtures_deserialize() {
    let graph = include_str!("../../../tests/fixtures/p15/compiled_region_graph_v1.json");
    let graph: CompiledRegionGraphV1 = serde_json::from_str(graph).unwrap();
    assert_eq!(graph.kind, ArtifactKindV1::CompiledRegionGraph);
    assert!(graph.is_bounded_region_graph());

    let contract = include_str!("../../../tests/fixtures/p15/region_contract_v1.json");
    let contract: RegionContractV1 = serde_json::from_str(contract).unwrap();
    assert_eq!(contract.kind, ArtifactKindV1::RegionContract);
    assert!(contract.is_bounded());

    let residual = include_str!("../../../tests/fixtures/p15/residual_v1.json");
    let residual: KernelResidualReportV1 = serde_json::from_str(residual).unwrap();
    assert_eq!(residual.kind, ArtifactKindV1::Residual);

    let syndrome = include_str!("../../../tests/fixtures/p15/syndrome_v1.json");
    let syndrome: KernelSyndromeReportV1 = serde_json::from_str(syndrome).unwrap();
    assert_eq!(syndrome.kind, ArtifactKindV1::Syndrome);
    assert!(syndrome.requires_canonical_repair());

    let oracle = include_str!("../../../tests/fixtures/p15/oracle_slice_request_v1.json");
    let oracle: OracleSliceRequestV1 = serde_json::from_str(oracle).unwrap();
    assert_eq!(oracle.kind, ArtifactKindV1::OracleSliceRequest);

    let report = include_str!("../../../tests/fixtures/p15/convergence_report_v1.json");
    let report: ConvergenceReportV1 = serde_json::from_str(report).unwrap();
    assert_eq!(report.kind, ArtifactKindV1::ConvergenceReport);
    assert!(report.has_explicit_stop_rule_evidence());

    let run = include_str!("../../../tests/fixtures/p15/kernel_run_receipt_v1.json");
    let run: KernelRunDisplayReportV1 = serde_json::from_str(run).unwrap();
    assert_eq!(run.kind, ArtifactKindV1::KernelRunReport);
    assert!(run.convergence_is_evidenced());
}

#[test]
fn p16_subtraction_artifacts_block_support_loss_and_record_compaction() {
    let accepted_claim = ArtifactId("claim:accepted".into());
    let evidence = ArtifactId("evidence:accepted".into());
    let budget = InvariantBudgetV1::full_history().with_removal_limits(1, 1, 0);
    let support = SupportCoreV1::new(
        vec![accepted_claim.clone()],
        Vec::new(),
        vec![evidence.clone()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let blocked_frontier = RemovalFrontierV1::new(
        &support,
        vec![accepted_claim.clone()],
        vec![evidence],
        Vec::new(),
        &budget,
    );
    let blocked_plan = SubtractionPlanV1::dry_run(
        SubtractionOperatorV1::Compact,
        &support,
        &blocked_frontier,
        &budget,
    );

    assert!(blocked_frontier.is_blocked());
    assert!(blocked_plan.blocked);
    assert!(!blocked_plan.is_lawful_append_only_reduction());
    assert!(!blocked_plan.destructive_deletion);

    let superseded_support = SupportCoreV1::new(
        vec![accepted_claim.clone()],
        vec![accepted_claim.clone()],
        Vec::new(),
        Vec::new(),
        vec![accepted_claim.clone()],
        Vec::new(),
    );
    let lawful_frontier = RemovalFrontierV1::new(
        &superseded_support,
        vec![accepted_claim.clone()],
        Vec::new(),
        Vec::new(),
        &budget,
    );
    let lawful_plan = SubtractionPlanV1::dry_run(
        SubtractionOperatorV1::Compact,
        &superseded_support,
        &lawful_frontier,
        &budget,
    );
    let digest = DisplayDigestV1::for_json_value(&serde_json::json!({
        "claims": [accepted_claim]
    }));
    let report =
        HistoryPreservationReportV1::new(&lawful_plan, &budget, digest.clone(), digest, Vec::new());
    let receipt = CompactionReportV1::append_only(&lawful_plan, &report, Vec::new());

    assert!(!lawful_frontier.is_blocked());
    assert!(lawful_plan.is_lawful_append_only_reduction());
    assert!(report.preserves_declared_history());
    assert!(receipt.compacted);
    assert!(!receipt.destructive_deletion);
}

#[test]
fn p28_history_preservation_uses_invariant_evidence_not_digest_equality() {
    let accepted_claim = ArtifactId("claim:p28".into());
    let support = SupportCoreV1::new(
        vec![accepted_claim],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let budget = InvariantBudgetV1::full_history().with_removal_limits(1, 1, 1);
    let frontier = RemovalFrontierV1::new(&support, Vec::new(), Vec::new(), Vec::new(), &budget);
    let plan =
        SubtractionPlanV1::dry_run(SubtractionOperatorV1::Compact, &support, &frontier, &budget);
    let before = DisplayDigestV1::for_json_value(&serde_json::json!({"claims": ["before"]}));
    let after = DisplayDigestV1::for_json_value(&serde_json::json!({"claims": ["after"]}));
    let report = HistoryPreservationReportV1::new(
        &plan,
        &budget,
        before,
        after,
        vec![ArtifactId("query-receipt:p28".into())],
    );

    assert_ne!(report.before_digest, report.after_digest);
    assert!(report.as_of_queries_preserved);
    assert!(report.preserves_declared_history());
}

#[test]
fn p16_golden_fixtures_deserialize() {
    let budget = include_str!("../../../tests/fixtures/p16/invariant_budget_v1.json");
    let budget: InvariantBudgetV1 = serde_json::from_str(budget).unwrap();
    assert_eq!(budget.kind, ArtifactKindV1::InvariantBudget);
    assert!(budget.preserves_as_of_queries());

    let support = include_str!("../../../tests/fixtures/p16/support_core_v1.json");
    let support: SupportCoreV1 = serde_json::from_str(support).unwrap();
    assert_eq!(support.kind, ArtifactKindV1::SupportCore);

    let frontier = include_str!("../../../tests/fixtures/p16/removal_frontier_v1.json");
    let frontier: RemovalFrontierV1 = serde_json::from_str(frontier).unwrap();
    assert_eq!(frontier.kind, ArtifactKindV1::RemovalFrontier);

    let plan = include_str!("../../../tests/fixtures/p16/subtraction_plan_v1.json");
    let plan: SubtractionPlanV1 = serde_json::from_str(plan).unwrap();
    assert_eq!(plan.kind, ArtifactKindV1::SubtractionPlan);
    assert!(plan.is_lawful_append_only_reduction());

    let report = include_str!("../../../tests/fixtures/p16/history_preservation_report_v1.json");
    let report: HistoryPreservationReportV1 = serde_json::from_str(report).unwrap();
    assert_eq!(report.kind, ArtifactKindV1::HistoryPreservationReport);
    assert!(report.preserves_declared_history());

    let receipt = include_str!("../../../tests/fixtures/p16/compaction_receipt_v1.json");
    let receipt: CompactionReportV1 = serde_json::from_str(receipt).unwrap();
    assert_eq!(receipt.kind, ArtifactKindV1::CompactionReport);
    assert!(receipt.compacted);
    assert!(!receipt.destructive_deletion);
}

#[test]
fn p17_attestation_and_settlement_names_are_canonical_reexports() {
    let envelope = AttestationEnvelopeV1::new(
        stack_ids::AttestationEnvelopeId::new("attestation:remote-a"),
        "claim-record",
        "1",
        StackContentDigest::compute_str("remote-ready"),
        "claim-record-v1",
        "remote-a",
        "2026-04-29T00:00:00Z",
        stack_ids::TrustRootSetId::new("trust-root-set:remote-a"),
        "producer remote-a signed claim-record artifact",
        stack_ids::DisclosurePolicyId::new("disclosure:claims"),
        Some(stack_ids::ArtifactAdmissionPolicyId::new(
            "artifact-admission:claims",
        )),
        attestation_exchange::AttestationReplayabilityClassV1::Replayable,
        Vec::new(),
        Vec::new(),
    )
    .unwrap();

    let shared = SharedDispositionV1 {
        schema_version: federated_settlement::SHARED_DISPOSITION_V1_SCHEMA.to_string(),
        shared_disposition_id: stack_ids::SharedDispositionId::new("shared:local-first"),
        disposition_label: "local-authority-preserved".into(),
        treatment: "remote advisory only".into(),
        advisory_only: true,
    };
    let case = SettlementCaseV1 {
        schema_version: federated_settlement::SETTLEMENT_CASE_V1_SCHEMA.to_string(),
        settlement_case_id: stack_ids::SettlementCaseId::new("settlement:remote-a"),
        treaty_bundle_id: stack_ids::TreatyBundleId::new("treaty:claims"),
        equivalence_bundle_id: stack_ids::CrossRuntimeEquivalenceBundleId::new(
            "equivalence:claims",
        ),
        citation: stack_ids::V25ConstitutionCitation::default(),
        shared_disposition: shared.clone(),
        replay_requirements: Vec::new(),
        local_dissent: Vec::new(),
        advisory_only: true,
        non_admitted: false,
    };

    assert_eq!(envelope.artifact_family, "claim-record");
    assert_eq!(
        envelope.schema_version,
        attestation_exchange::ATTESTATION_ENVELOPE_V1_SCHEMA
    );
    assert_eq!(shared.disposition_label, "local-authority-preserved");
    assert!(case.advisory_only);
    assert_eq!(case.shared_disposition, shared);
}

#[test]
fn p17_canonical_reexport_shapes_roundtrip() {
    let shared = SharedDispositionV1 {
        schema_version: federated_settlement::SHARED_DISPOSITION_V1_SCHEMA.to_string(),
        shared_disposition_id: stack_ids::SharedDispositionId::new("shared:roundtrip"),
        disposition_label: "advisory".into(),
        treatment: "display only".into(),
        advisory_only: true,
    };
    let encoded = serde_json::to_string(&shared).unwrap();
    let decoded: SharedDispositionV1 = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.shared_disposition_id.as_str(), "shared:roundtrip");
    assert!(decoded.advisory_only);
}

#[test]
fn p18_mechanism_names_are_canonical_reexports() {
    let mechanism_id = stack_ids::MechanismBundleId::new("mechanism:linear");
    let theory = TheoryVersionV1 {
        schema_version: mechanism_runtime::THEORY_VERSION_V1_SCHEMA.to_string(),
        theory_version_id: stack_ids::TheoryVersionId::new("theory:linear:v1"),
        mechanism_bundle_id: mechanism_id.clone(),
        theory_name: "linear response".into(),
        semantic_version: "1.0.0".into(),
        advisory_only: true,
    };
    let refuters = TheoryRefuterSuiteV1 {
        schema_version: mechanism_runtime::THEORY_REFUTER_SUITE_V1_SCHEMA.to_string(),
        theory_refuter_suite_id: stack_ids::TheoryRefuterSuiteId::new("refuters:linear:v1"),
        theory_version_id: theory.theory_version_id.clone(),
        required_refuters: vec!["intervention-shift".into()],
        available_refuters: Vec::new(),
        failing_refuters: Vec::new(),
        horizon_only: true,
    };
    let library = HypothesisLibraryV1 {
        schema_version: mechanism_runtime::HYPOTHESIS_LIBRARY_V1_SCHEMA.to_string(),
        hypothesis_library_id: stack_ids::HypothesisLibraryId::new("hypothesis-library:p18"),
        hypothesis_refs: vec![theory.theory_version_id.to_string()],
        publication_status: stack_ids::SurfaceStatus::AdvisoryOnly,
    };

    assert!(theory.advisory_only);
    assert!(refuters.horizon_only);
    assert_eq!(
        library.publication_status,
        stack_ids::SurfaceStatus::AdvisoryOnly
    );
}

#[test]
fn p18_canonical_reexport_shapes_roundtrip() {
    let theory = TheoryVersionV1 {
        schema_version: mechanism_runtime::THEORY_VERSION_V1_SCHEMA.to_string(),
        theory_version_id: stack_ids::TheoryVersionId::new("theory:roundtrip"),
        mechanism_bundle_id: stack_ids::MechanismBundleId::new("mechanism:roundtrip"),
        theory_name: "roundtrip".into(),
        semantic_version: "1.0.0".into(),
        advisory_only: true,
    };
    let encoded = serde_json::to_string(&theory).unwrap();
    let decoded: TheoryVersionV1 = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.theory_name, "roundtrip");
    assert!(decoded.advisory_only);
}

#[test]
fn p19_completion_audit_discloses_deferred_horizon_and_blocks_release_bar() {
    let example = ExampleAppEntryV1::new(
        "examples/aidens.mock.toml",
        "coding-agent",
        "mock",
        MemoryModeV1::Disabled,
        ReleaseSurfaceStateV1::Supported,
    );
    let readiness = ReleaseReadinessReportV1::new(
        vec![
            ReleaseSurfaceV1::new(
                "cli:provider-check",
                ReleaseSurfaceStateV1::Supported,
                "provider-check exposes executable provider truth",
            ),
            ReleaseSurfaceV1::new(
                "crate:aidens-profile-daemon",
                ReleaseSurfaceStateV1::Deferred,
                "scaffold-only; deferred until daemon profile wiring",
            ),
        ],
        Vec::new(),
        ExampleAppManifestV1::new(
            vec![example],
            vec!["aidens-profile-daemon: deferred until daemon profile wiring".into()],
        ),
        InstallSmokeReportV1::new(vec![InstallSmokeStepV1::passed(
            "verify-script-present",
            "test -f scripts/verify.sh",
            "true",
        )]),
    );
    let traceability = CrossPassTraceabilityMatrixV1::new(vec![CrossPassTraceabilityRowV1::new(
        "AIDENS-P19-01",
        "P19",
        "run required gates",
        PassCompletionStateV1::Done,
    )
    .with_crates(vec!["workspace root".into()])
    .with_artifacts(vec!["completion-audit-report".into()])
    .with_tests(vec!["cargo test --workspace".into()])
    .with_docs(vec!["STATUS.md".into()])
    .with_acceptance_gates(vec!["cargo fmt/check/test/clippy".into()])
    .with_evidence(vec![
        "handoffs/P19_FINAL_INTEGRATION_RELEASE_BAR_AND_COMPLETION_AUDIT.md".into(),
    ])]);
    let manifest = ReleaseArtifactManifestV1::new(
        "aidens-p19",
        "libraries-source-clean-20260426.zip",
        vec![ReleaseArtifactEntryV1::present(
            "schemas/generated_schema_manifest_v1.json",
            ReleaseArtifactKindV1::Manifest,
            true,
            "blake3:fixture",
            12,
        )],
    );
    let limitations = KnownLimitationsRegisterV1::new(vec![KnownLimitationV1::deferred(
        "crate:aidens-profile-daemon",
        "daemon profile wrapper remains scaffold-only",
        "operators use queue/daemon CLI surfaces directly",
    )
    .with_review_after_pass("post-P19-roadmap")]);
    let debt = RegressionDebtLedgerV1::new(vec![RegressionDebtItemV1::guarded(
        "release:docs",
        "public-doc scaffold promotion can regress",
        "scripts/assert_no_scaffold_promoted.sh",
        vec!["bash scripts/verify.sh".into()],
    )]);
    let gates = vec![
        GateCommandResultV1::passed("cargo fmt --all --check", "p19-handoff"),
        GateCommandResultV1::passed(
            "cargo check --workspace --all-targets --all-features",
            "p19-handoff",
        ),
        GateCommandResultV1::passed(
            "cargo test --workspace --all-targets --all-features",
            "p19-handoff",
        ),
        GateCommandResultV1::passed(
            "cargo clippy --workspace --all-targets --all-features -- -D warnings",
            "p19-handoff",
        ),
        GateCommandResultV1::passed(
            "P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh",
            "p19-handoff",
        ),
    ];
    let report = CompletionAuditReportV1::new(
        "libraries-source-clean-20260426.zip",
        gates,
        readiness,
        traceability,
        manifest,
        limitations,
        debt,
    );

    assert_eq!(report.kind, ArtifactKindV1::CompletionAuditReport);
    assert!(!report.release_bar_passed);
    assert_eq!(
        report.completion_state,
        CompletionAuditStateV1::DeferredHorizon
    );
    assert!(report
        .deferred_surfaces
        .contains(&"crate:aidens-profile-daemon".into()));
    assert!(report
        .reason_codes
        .contains(&"deferred-horizon-surfaces-disclosed".into()));
    assert!(!report
        .reason_codes
        .contains(&"unsubstantiated-healthy-claim".into()));
}

#[test]
fn p28_waiver_does_not_satisfy_blocked_traceability_row_unless_state_is_waived() {
    let blocked = CrossPassTraceabilityRowV1::new(
        "P28-C55",
        "P28",
        "blocked work",
        PassCompletionStateV1::Blocked,
    )
    .with_waiver("waiver:p28");
    assert!(!blocked.is_satisfied());

    let waived = CrossPassTraceabilityRowV1::new(
        "P28-C55-WAIVED",
        "P28",
        "lawfully waived work",
        PassCompletionStateV1::Waived,
    );
    assert!(waived.is_satisfied());
}

#[test]
fn p28_empty_known_limitations_register_does_not_block_completion() {
    let limitations = KnownLimitationsRegisterV1::new(Vec::new());
    assert!(limitations.current);
    assert!(limitations.limitations.is_empty());
    assert!(!limitations.blocks_completion());
    assert!(limitations
        .reason_codes
        .contains(&"known-limitations-empty-register-current".into()));
}

#[test]
fn p28_artifact_lifecycle_requires_receipted_transitions_and_blocks_early_promotion() {
    let payload = serde_json::json!({"operator": "aidens.agent.validate", "status": "ok"});
    let mut envelope = ArtifactEnvelopeV1::from_json(
        "agent-validation-report",
        1,
        &payload,
        ArtifactAuthorityClassV1::AiDENsExecutionAuthoritative,
        "p28-local-agent",
        "aidens",
    )
    .with_canonical_backpointer(CanonicalBackpointerV1::owner_type(
        "verification-control",
        "VerificationCase",
        "canonical-proof-owner",
    ));

    assert_eq!(envelope.lifecycle_state, ArtifactLifecycleStateV1::Created);
    assert!(envelope.content_digest.is_some());
    assert!(!envelope.is_promotable());

    let rejected = ArtifactTransitionReceiptV1::new(
        envelope.artifact_ref.clone(),
        ArtifactLifecycleStateV1::Created,
        ArtifactLifecycleStateV1::Promoted,
        "aidens.report.final_done",
        "test",
        None,
    );
    assert!(rejected.is_err());

    let validate = envelope
        .apply_transition(
            ArtifactLifecycleStateV1::Validated,
            "aidens.agent.validate",
            "test",
            Some(ArtifactId("execution-context:p28".into())),
        )
        .unwrap();
    assert_eq!(validate.previous_state, ArtifactLifecycleStateV1::Created);
    assert_eq!(validate.new_state, ArtifactLifecycleStateV1::Validated);
    assert!(validate
        .canonical_backpointers
        .iter()
        .any(|backpointer| backpointer.owner_crate == "verification-control"));

    envelope
        .apply_transition(
            ArtifactLifecycleStateV1::Admitted,
            "aidens.agent.doctor",
            "test",
            None,
        )
        .unwrap();
    envelope
        .apply_transition(
            ArtifactLifecycleStateV1::Projected,
            "aidens.runner.turn",
            "test",
            None,
        )
        .unwrap();
    envelope
        .apply_transition(
            ArtifactLifecycleStateV1::Verified,
            "aidens.tool.run_checks",
            "test",
            None,
        )
        .unwrap();
    assert!(envelope.is_promotable());
    let promote = envelope
        .apply_transition(
            ArtifactLifecycleStateV1::Promoted,
            "aidens.report.final_done",
            "test",
            None,
        )
        .unwrap();
    assert_eq!(promote.previous_state, ArtifactLifecycleStateV1::Verified);
    assert_eq!(promote.new_state, ArtifactLifecycleStateV1::Promoted);
    envelope
        .apply_transition(
            ArtifactLifecycleStateV1::Superseded,
            "aidens.package.generate",
            "test",
            None,
        )
        .unwrap();
    envelope
        .apply_transition(
            ArtifactLifecycleStateV1::Retired,
            "aidens.package.validate",
            "test",
            None,
        )
        .unwrap();
    assert_eq!(envelope.lifecycle_state, ArtifactLifecycleStateV1::Retired);
}

#[test]
fn p28_artifact_manifest_records_inputs_outputs_and_missing_refs() {
    let input = ArtifactEnvelopeV1::from_json(
        "agent-spec",
        1,
        &serde_json::json!({"id": "agent"}),
        ArtifactAuthorityClassV1::AdmittedFacade,
        "p28",
        "aidens",
    );
    let mut output = ArtifactEnvelopeV1::from_json(
        "doctor-report",
        1,
        &serde_json::json!({"valid": true}),
        ArtifactAuthorityClassV1::AiDENsExecutionAuthoritative,
        "p28",
        "aidens",
    );
    output
        .apply_transition(
            ArtifactLifecycleStateV1::Validated,
            "aidens.agent.doctor",
            "test",
            None,
        )
        .unwrap();
    let mut schema_identities = BTreeMap::new();
    schema_identities.insert("doctor-report".into(), "schema:doctor-report:v1".into());
    let manifest = ArtifactManifestV1::new(
        vec![ArtifactManifestEntryV1::from(&input)],
        vec![ArtifactManifestEntryV1::from(&output)],
        "stack-ids-json-c14n-v1",
        schema_identities,
    );

    assert!(manifest.complete());
    assert_eq!(manifest.inputs[0].family, "agent-spec");
    assert_eq!(manifest.outputs[0].family, "doctor-report");
    assert!(manifest
        .reason_codes
        .contains(&"artifact-manifest-created".into()));

    let degraded = manifest.clone().with_missing_or_opaque_ref(
        "opaque:external-input",
        "external input was not available for replay",
        ArtifactId("degradation:p12-missing-ref".into()),
    );
    assert!(!degraded.complete());
    assert_eq!(degraded.missing_or_opaque_ref_records.len(), 1);
    assert_eq!(
        degraded.missing_or_opaque_ref_records[0].degradation_record_id,
        ArtifactId("degradation:p12-missing-ref".into())
    );
}

#[test]
fn p28_material_done_requires_execution_context_manifests_and_receipts() {
    let context = ExecutionContextEnvelopeV1::local_started(
        "aidens.tool.repo_read",
        ArtifactId("attempt-family:p28".into()),
        "mock",
        "aidens:repo-read:1",
    )
    .complete(ExecutionCompletionStateV1::Succeeded, 12);
    let input = ArtifactEnvelopeV1::from_json(
        "repo-read-input",
        1,
        &serde_json::json!({"path": "README.md"}),
        ArtifactAuthorityClassV1::AiDENsExecutionAuthoritative,
        "p28",
        "aidens",
    );
    let output = ArtifactEnvelopeV1::from_json(
        "repo-read-output",
        1,
        &serde_json::json!({"bytes": 10}),
        ArtifactAuthorityClassV1::AiDENsExecutionAuthoritative,
        "p28",
        "aidens",
    );
    let input_manifest = ArtifactManifestV1::new(
        vec![ArtifactManifestEntryV1::from(&input)],
        Vec::new(),
        "stack-ids-json-c14n-v1",
        BTreeMap::new(),
    );
    let output_manifest = ArtifactManifestV1::new(
        Vec::new(),
        vec![ArtifactManifestEntryV1::from(&output)],
        "stack-ids-json-c14n-v1",
        BTreeMap::new(),
    );

    let rejected = OperatorInvocationReceiptV1::material_done(
        "aidens.tool.repo_read",
        &context,
        input_manifest.clone(),
        output_manifest.clone(),
        Vec::new(),
    );
    assert!(rejected.is_err());

    let incomplete_input_manifest = input_manifest.clone().with_missing_or_opaque_ref(
        "opaque:repo-root",
        "repo root digest unavailable",
        ArtifactId("degradation:p12-repo-root".into()),
    );
    let rejected_incomplete = OperatorInvocationReceiptV1::material_done(
        "aidens.tool.repo_read",
        &context,
        incomplete_input_manifest,
        output_manifest.clone(),
        vec![ArtifactId("tool-call:p12".into())],
    );
    assert!(rejected_incomplete.is_err());

    let tool_receipt = ToolCallReceiptV1::new(
        &context,
        "aidens:repo-read:1",
        &serde_json::json!({"path": "README.md"}),
        &serde_json::json!({"content": "hello"}),
        ExecutionCompletionStateV1::Succeeded,
    );
    let done = OperatorInvocationReceiptV1::material_done(
        "aidens.tool.repo_read",
        &context,
        input_manifest,
        output_manifest,
        vec![tool_receipt.receipt_id.clone()],
    )
    .unwrap();

    assert!(done.material_done);
    assert_eq!(done.tool_call_receipt_refs, vec![tool_receipt.receipt_id]);
    assert_eq!(
        done.reason_codes,
        vec!["material-operation-done-with-receipts".to_string()]
    );
}

#[test]
fn p28_timeout_tool_receipt_marks_partial_output() {
    let context = ExecutionContextEnvelopeV1::local_started(
        "aidens.tool.run_checks",
        ArtifactId("attempt-family:p28-timeout".into()),
        "local",
        "aidens:run-checks:1",
    )
    .complete(ExecutionCompletionStateV1::TimedOut, 1000);
    let receipt = ToolCallReceiptV1::new(
        &context,
        "aidens:run-checks:1",
        &serde_json::json!({"command": ["cargo", "test"]}),
        &serde_json::json!({"stdout_tail": "truncated"}),
        ExecutionCompletionStateV1::TimedOut,
    );

    assert!(receipt.partial_output);
    assert_eq!(context.deadline_status, "partial-or-timeout");
    assert!(receipt
        .reason_codes
        .contains(&"tool-output-partial-or-timeout".into()));
}

#[test]
fn p29_execution_context_fingerprint_is_environment_scoped() {
    let context = ExecutionContextEnvelopeV1::local_started(
        "aidens.tool.repo_read",
        ArtifactId("attempt-family:p29-fingerprint".into()),
        "local",
        "aidens:repo-read:1",
    );

    assert_ne!(context.environment_fingerprint, env!("CARGO_PKG_VERSION"));
    assert!(context
        .environment_fingerprint
        .contains(env!("CARGO_PKG_NAME")));
    assert!(context
        .environment_fingerprint
        .contains(std::env::consts::OS));
    assert!(context
        .environment_fingerprint
        .contains(std::env::consts::ARCH));
}

#[test]
fn p29_repeated_identical_tool_calls_get_distinct_receipts() {
    let context = ExecutionContextEnvelopeV1::local_started(
        "aidens.tool.repo_read",
        ArtifactId("attempt-family:p29-tool-receipt".into()),
        "local",
        "aidens:repo-read:1",
    );
    let input = serde_json::json!({"path": "README.md"});
    let output = serde_json::json!({"content": "same"});
    let first = ToolCallReceiptV1::new(
        &context,
        "aidens:repo-read:1",
        &input,
        &output,
        ExecutionCompletionStateV1::Succeeded,
    );
    std::thread::sleep(std::time::Duration::from_millis(1));
    let second = ToolCallReceiptV1::new(
        &context,
        "aidens:repo-read:1",
        &input,
        &output,
        ExecutionCompletionStateV1::Succeeded,
    );

    assert_eq!(first.started_at, context.started_at);
    assert_eq!(second.started_at, context.started_at);
    assert!(first.completed_at >= context.started_at);
    assert!(second.completed_at >= first.completed_at);
    assert_ne!(first.receipt_id, second.receipt_id);
}

#[test]
fn p28_required_material_operator_contracts_are_registered() {
    let registry = p28_declared_material_operation_registry();
    let required = p28_required_operator_ids();
    let report = OperationConformanceReportV1::for_required_operators(&registry, &required);

    assert!(report.passed);
    assert!(report.missing_operator_ids.is_empty());
    for operator_id in required {
        let contract = registry.contract(operator_id).expect("required contract");
        assert!(!contract.input_families.is_empty());
        assert!(!contract.output_families.is_empty());
        assert!(!contract.proof_obligations.is_empty());
        assert!(!contract.replay_requirements.is_empty());
        assert!(!contract.failure_taxonomy.is_empty());
        assert!(!contract.boundary_profile.is_empty());
    }
}

#[test]
fn p28_material_operator_registry_blocks_undeclared_effects() {
    let registry = p28_declared_material_operation_registry();
    assert!(registry
        .authorize_effects(
            "aidens.tool.repo_read",
            BTreeSet::from([
                OperatorEffectV1::ReadsRepository,
                OperatorEffectV1::EmitsReceipt
            ])
        )
        .is_ok());

    let denied = registry.authorize_effects(
        "aidens.tool.repo_read",
        BTreeSet::from([OperatorEffectV1::WritesRepository]),
    );
    assert!(denied.is_err());
    assert!(denied.unwrap_err().contains("forbidden"));

    let missing = registry.authorize_effects(
        "aidens.tool.not_declared",
        BTreeSet::from([OperatorEffectV1::EmitsReceipt]),
    );
    assert!(missing.is_err());
    assert!(missing.unwrap_err().contains("contract missing"));
}

#[test]
fn p12_operator_invocation_authorization_requires_effects_manifests_receipts_and_finite_taxonomy() {
    let registry = p28_declared_material_operation_registry();
    let input = ArtifactEnvelopeV1::from_json(
        "repo-read-input",
        1,
        &serde_json::json!({"path": "README.md"}),
        ArtifactAuthorityClassV1::AiDENsExecutionAuthoritative,
        "p12",
        "aidens",
    );
    let output = ArtifactEnvelopeV1::from_json(
        "repo-read-output",
        1,
        &serde_json::json!({"bytes": 10}),
        ArtifactAuthorityClassV1::AiDENsExecutionAuthoritative,
        "p12",
        "aidens",
    );
    let input_manifest = ArtifactManifestV1::new(
        vec![ArtifactManifestEntryV1::from(&input)],
        Vec::new(),
        "stack-ids-json-c14n-v1",
        BTreeMap::new(),
    );
    let output_manifest = ArtifactManifestV1::new(
        Vec::new(),
        vec![ArtifactManifestEntryV1::from(&output)],
        "stack-ids-json-c14n-v1",
        BTreeMap::new(),
    );
    let read_effects = BTreeSet::from([
        OperatorEffectV1::ReadsRepository,
        OperatorEffectV1::EmitsReceipt,
    ]);
    let mut execution_context = ExecutionContextEnvelopeV1::local_started(
        "aidens.tool.repo_read",
        display_only_unstable_id("p12-attempt"),
        "mock",
        "aidens.tool.repo_read",
    );
    execution_context.budget_millis_allocated = 10;
    let execution_context = execution_context.complete(ExecutionCompletionStateV1::Succeeded, 5);

    assert!(registry
        .contract("aidens.tool.repo_read")
        .unwrap()
        .failure_taxonomy_is_finite());
    assert!(registry
        .authorize_material_invocation(
            "aidens.tool.repo_read",
            read_effects.clone(),
            &execution_context,
            &input_manifest,
            &output_manifest,
            &[ArtifactId("tool-call:p12".into())],
        )
        .is_ok());

    let missing_receipt = registry.authorize_material_invocation(
        "aidens.tool.repo_read",
        read_effects.clone(),
        &execution_context,
        &input_manifest,
        &output_manifest,
        &[],
    );
    assert!(missing_receipt
        .unwrap_err()
        .contains("missing material receipts"));

    let mut over_budget_success = ExecutionContextEnvelopeV1::local_started(
        "aidens.tool.repo_read",
        display_only_unstable_id("p12-attempt-over-budget"),
        "mock",
        "aidens.tool.repo_read",
    );
    over_budget_success.budget_millis_allocated = 10;
    let over_budget_success =
        over_budget_success.complete(ExecutionCompletionStateV1::Succeeded, 11);
    let budget_violation = registry.authorize_material_invocation(
        "aidens.tool.repo_read",
        read_effects.clone(),
        &over_budget_success,
        &input_manifest,
        &output_manifest,
        &[ArtifactId("tool-call:p12".into())],
    );
    assert!(budget_violation.unwrap_err().contains("budget enforcement"));

    let opaque_input = input_manifest.with_missing_or_opaque_ref(
        "opaque:repo-root",
        "repo root digest unavailable",
        ArtifactId("degradation:p12-operator".into()),
    );
    let incomplete = registry.authorize_material_invocation(
        "aidens.tool.repo_read",
        read_effects,
        &execution_context,
        &opaque_input,
        &output_manifest,
        &[ArtifactId("tool-call:p12".into())],
    );
    assert!(incomplete.unwrap_err().contains("manifests incomplete"));

    let denied = registry.authorize_material_invocation(
        "aidens.tool.repo_read",
        BTreeSet::from([OperatorEffectV1::WritesRepository]),
        &execution_context,
        &opaque_input,
        &output_manifest,
        &[ArtifactId("tool-call:p12".into())],
    );
    assert!(denied.unwrap_err().contains("forbidden"));
}

#[test]
fn p28_proof_waiver_is_not_proof_and_debt_blocks_promotion() {
    let mut obligation = ProofObligationV1::new("cargo test passes", "test-log");
    let waiver = ProofWaiverReceiptV1::new(
        obligation.obligation_id.clone(),
        "operator",
        "temporary local waiver",
    );
    obligation.waived_by.push(waiver.receipt_id.clone());
    let profile = LocalProofProfileV1::local_exact(vec![obligation]);
    let artifact_ref = ArtifactId("artifact:p28-proof".into());
    let debt = ProofDebtLedgerV1::from_profile(artifact_ref.clone(), &profile);
    let eligibility = PromotionEligibilityReportV1::new(artifact_ref, &profile, &debt);

    assert!(waiver.waiver_is_not_proof);
    assert!(!profile.proof_satisfied());
    assert!(profile.has_waiver_without_proof());
    assert!(debt.blocks_promotion());
    assert!(!eligibility.eligible);
    assert_eq!(
        debt.items[0].restriction,
        ProofUseRestrictionV1::AdvisoryOnly
    );
}

#[test]
fn p09_proof_debt_is_queryable_and_expired_waiver_escalates() {
    let now = DateTime::parse_from_rfc3339("2026-05-07T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let expired_at = DateTime::parse_from_rfc3339("2026-05-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let active_until = DateTime::parse_from_rfc3339("2026-06-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let mut obligation = ProofObligationV1::new("phase 09 proof", "hostile-fixture");
    obligation
        .waived_by
        .push(ArtifactId("proof-waiver:phase09".into()));
    let profile = LocalProofProfileV1::local_exact(vec![obligation]);
    let artifact_ref = ArtifactId("artifact:p09-proof-debt".into());
    let mut debt = ProofDebtLedgerV1::from_profile(artifact_ref, &profile);

    debt.items[0] = debt.items[0].clone().with_expiry(active_until);
    assert!(debt.allows_use_at("local-advisory-display", now));
    assert!(!debt.allows_use_at("release-promotion", now));
    assert!(debt.expired_items_at(now).is_empty());

    debt.items[0] = debt.items[0].clone().with_expiry(expired_at);
    assert_eq!(debt.expired_items_at(now).len(), 1);
    assert_eq!(debt.escalated_items_at(now).len(), 1);
    assert!(!debt.allows_use_at("local-advisory-display", now));
}

#[test]
fn p28_proof_evidence_satisfies_profile_and_allows_promotion() {
    let mut obligation = ProofObligationV1::new("cargo test passes", "test-log");
    obligation
        .satisfied_by
        .push(ArtifactId("test-log:p28".into()));
    let profile = LocalProofProfileV1::local_exact(vec![obligation]);
    let artifact_ref = ArtifactId("artifact:p28-proof-pass".into());
    let debt = ProofDebtLedgerV1::from_profile(artifact_ref.clone(), &profile);
    let eligibility = PromotionEligibilityReportV1::new(artifact_ref, &profile, &debt);

    assert!(profile.proof_satisfied());
    assert!(debt.items.is_empty());
    assert!(eligibility.eligible);
}

#[test]
fn p28_degraded_release_surface_blocks_readiness() {
    let manifest = ExampleAppManifestV1::new(
        vec![ExampleAppEntryV1::new(
            "examples/aidens.mock.toml",
            "coding-agent",
            "mock",
            MemoryModeV1::Disabled,
            ReleaseSurfaceStateV1::Supported,
        )],
        Vec::new(),
    );
    let smoke = InstallSmokeReportV1::new(vec![InstallSmokeStepV1::passed(
        "provider-check",
        "aidens provider-check --config examples/aidens.mock.toml",
        "provider-check ok",
    )]);
    let readiness = ReleaseReadinessReportV1::new(
        vec![ReleaseSurfaceV1::new(
            "package:self-replay",
            ReleaseSurfaceStateV1::Degraded,
            "skip-cargo replay is degraded",
        )],
        Vec::new(),
        manifest,
        smoke,
    );

    assert!(readiness.blocks_release());
    assert!(!readiness.ready);
}

#[test]
fn p28_semantic_state_degradation_cannot_answer_as_exact() {
    let artifact_ref = ArtifactId("artifact:p28-semantic".into());
    let degradation = LocalDegradationRecordV1::new(
        artifact_ref.clone(),
        "skip-cargo-replay",
        "cargo-backed replay proof absent",
    );
    let disclosure = ViewDisclosureV1::widening(
        "package-self-replay",
        "supported-local",
        Some(ArtifactId("view-report:p28".into())),
    );
    let state = SemanticStateV1::exact_supported(artifact_ref, "proof-profile:p28")
        .with_degradation(&degradation)
        .with_view_disclosure(&disclosure);

    assert_eq!(state.exactness, SemanticExactnessV1::Degraded);
    assert!(!state.can_answer_as_exact());
    assert!(degradation.blocks_readiness_without_waiver());
    assert!(disclosure.widening);
    assert_eq!(disclosure.exactness, SemanticExactnessV1::Degraded);
}

#[test]
fn p09_semantic_state_blocks_exactness_for_contradiction_and_execution_contamination() {
    let artifact_ref = ArtifactId("artifact:p09-semantic".into());
    let contradiction = SemanticContradictionRecordV1::new(
        artifact_ref.clone(),
        vec![ArtifactId("witness:p09".into())],
        "refuted-by-hostile-fixture",
    );
    let contamination = ExecutionContaminationRecordV1::new(
        artifact_ref.clone(),
        ArtifactId("execution-context:p09".into()),
        "execution-output-used-as-domain-truth",
    );
    let state = SemanticStateV1::exact_supported(artifact_ref, "proof-profile:p09")
        .with_execution_contamination(&contamination)
        .with_contradiction(&contradiction);

    assert_eq!(contradiction.exactness_after, SemanticExactnessV1::Refuted);
    assert!(contradiction.promotion_blocking);
    assert!(contamination.domain_truth_contaminated);
    assert_eq!(state.exactness, SemanticExactnessV1::Refuted);
    assert!(!state.can_answer_as_exact());
    assert!(state.blocks_promotion());
    assert_eq!(state.contradiction_record_ids.len(), 1);
    assert_eq!(state.execution_contamination_ids.len(), 1);
}

#[test]
fn p28_v11b_region_and_subtraction_surfaces_remain_reserved_or_advisory() {
    let node = RegionNodeV1::new(RegionNodeKindV1::Claim, "claim", None);
    let graph_id = ArtifactId("graph:p28-reserved".into());
    let region_id = ArtifactId("region:p28-reserved".into());
    let contract = RegionContractV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Inference,
        region_id,
        vec![node.node_id.clone()],
        Vec::new(),
        Vec::new(),
        4,
    );
    let graph = CompiledRegionGraphV1::new(
        graph_id,
        RegionGraphKindV1::Inference,
        Some(RegionGraphKindV1::Storage),
        4,
        vec![node],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![contract.clone()],
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(
        contract.activation_level,
        V11ActivationLevelV1::ReservedDraft
    );
    assert!(contract.advisory_only);
    assert_eq!(graph.activation_level, V11ActivationLevelV1::ReservedDraft);
    assert!(graph.advisory_only);
    assert!(!graph.can_claim_active_v11b_runtime());

    let accepted_claim = ArtifactId("claim:p28-accepted".into());
    let support = SupportCoreV1::new(
        vec![accepted_claim.clone()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let budget = InvariantBudgetV1::full_history();
    let frontier = RemovalFrontierV1::new(
        &support,
        vec![accepted_claim],
        Vec::new(),
        Vec::new(),
        &budget,
    );
    let plan = SubtractionPlanV1::dry_run(
        SubtractionOperatorV1::SupportCoreExtraction,
        &support,
        &frontier,
        &budget,
    );
    assert_eq!(plan.activation_level, V11ActivationLevelV1::AdvisoryOnly);
    assert!(plan.advisory_only);
    assert!(plan.blocked);
    assert!(!plan.can_mutate_runtime_state());
}

#[test]
fn p29_right_graph_misuse_is_blocked_for_storage_or_unbounded_regions() {
    let node_a = RegionNodeV1::new(RegionNodeKindV1::Claim, "a", None);
    let node_b = RegionNodeV1::new(RegionNodeKindV1::Evidence, "b", None);
    let graph_id = ArtifactId("graph:p29-right-graph".into());
    let region = RegionContractV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Storage,
        ArtifactId("region:p29-right-graph".into()),
        vec![node_a.node_id.clone(), node_b.node_id.clone()],
        Vec::new(),
        Vec::new(),
        1,
    );
    let graph = CompiledRegionGraphV1::new(
        graph_id,
        RegionGraphKindV1::Storage,
        Some(RegionGraphKindV1::Storage),
        1,
        vec![node_a, node_b],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![region],
        Vec::new(),
        Vec::new(),
    );

    assert!(!graph.right_graph_law_satisfied);
    assert!(!graph.is_bounded_region_graph());
    assert!(!graph.can_claim_active_v11b_runtime());
    assert!(graph
        .reason_codes
        .contains(&"right-graph-law-blocked".into()));
}

#[test]
fn p10_right_graph_misuse_blocks_retrieval_and_control_kernel_execution() {
    for graph_kind in [RegionGraphKindV1::Retrieval, RegionGraphKindV1::Control] {
        let node = RegionNodeV1::new(RegionNodeKindV1::Claim, format!("{graph_kind}:node"), None);
        let graph_id = ArtifactId(format!("graph:p10-{graph_kind}"));
        let region = RegionContractV1::new(
            graph_id.clone(),
            graph_kind,
            ArtifactId(format!("region:p10-{graph_kind}")),
            vec![node.node_id.clone()],
            Vec::new(),
            Vec::new(),
            4,
        );
        let graph = CompiledRegionGraphV1::new(
            graph_id,
            graph_kind,
            Some(RegionGraphKindV1::Storage),
            4,
            vec![node],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![region],
            Vec::new(),
            Vec::new(),
        );

        assert!(!graph_kind.can_execute_kernel());
        assert!(!graph.right_graph_law_satisfied);
        assert!(!graph.is_bounded_region_graph());
        assert!(graph
            .reason_codes
            .contains(&"right-graph-law-blocked".into()));
    }
}

#[test]
fn p29_region_boundary_message_and_receipt_are_executable_seed_only() {
    let digest = DisplayDigestV1::for_json_value(&serde_json::json!({
        "claim": "bounded handoff"
    }));
    let message = RegionBoundaryMessageV1::seed(
        ArtifactId("region:p29-source".into()),
        ArtifactId("region:p29-destination".into()),
        "claim-projection",
        ArtifactId("artifact:p29-payload".into()),
        digest,
    );
    let receipt = RegionBoundaryReceiptV1::seed(&message, true);
    let rejected = RegionBoundaryReceiptV1::seed(&message, false);
    let quarantined = RegionBoundaryReceiptV1::quarantined(&message, "missing replay witness");

    assert_eq!(message.kind, ArtifactKindV1::BoundaryMessage);
    assert_eq!(receipt.kind, ArtifactKindV1::BoundaryReceipt);
    assert_eq!(receipt.message_id, message.message_id);
    assert_eq!(
        receipt.disposition,
        RegionBoundaryReceiptDispositionV1::Accepted
    );
    assert!(receipt.accepted);
    assert_eq!(
        rejected.disposition,
        RegionBoundaryReceiptDispositionV1::Rejected
    );
    assert!(!rejected.accepted);
    assert_eq!(
        quarantined.disposition,
        RegionBoundaryReceiptDispositionV1::Quarantined
    );
    assert_eq!(
        quarantined.quarantine_reason.as_deref(),
        Some("missing replay witness")
    );
    assert!(receipt.replay_required);
    assert_eq!(message.activation_level, V11ActivationLevelV1::AdvisoryOnly);
    assert_eq!(receipt.activation_level, V11ActivationLevelV1::AdvisoryOnly);
    assert!(message.advisory_only);
    assert!(receipt.advisory_only);
    assert!(!message.can_cross_runtime_boundary());
    assert!(!receipt.can_admit_runtime_payload());
    assert!(!rejected.can_admit_runtime_payload());
    assert!(!quarantined.can_admit_runtime_payload());
    assert!(message
        .reason_codes
        .contains(&"v11b-boundary-message-executable-seed".into()));
    assert!(receipt
        .reason_codes
        .contains(&"v11b-boundary-receipt-executable-seed".into()));
}

#[test]
fn p10_minimal_v11b_region_seed_covers_failure_repair_support_and_oracle_diff() {
    let graph_id = ArtifactId("graph:p10-minimal-region".into());
    let region_id = ArtifactId("region:p10-minimal-region".into());
    let accepted_claim = ArtifactId("claim:p10-accepted".into());
    let node = RegionNodeV1::new(
        RegionNodeKindV1::Claim,
        "accepted claim",
        Some(accepted_claim.clone()),
    );
    let region = RegionContractV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Inference,
        region_id.clone(),
        vec![node.node_id.clone()],
        Vec::new(),
        Vec::new(),
        4,
    );
    let graph = CompiledRegionGraphV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Inference,
        Some(RegionGraphKindV1::Storage),
        4,
        vec![node],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![region.clone()],
        vec![accepted_claim.clone()],
        Vec::new(),
    );
    assert!(graph.is_bounded_region_graph());
    assert!(!graph.can_claim_active_v11b_runtime());

    let stop = KernelStopRuleReportV1::new(
        CanonicalKernelStopReason::MaxIterations,
        2,
        2,
        0.01,
        0.5,
        0.2,
    );
    let residual = KernelResidualReportV1::new(
        graph_id.clone(),
        region_id.clone(),
        2,
        0.3,
        0.2,
        0.01,
        stop.clone(),
    );
    let convergence = ConvergenceReportV1::new(
        graph_id.clone(),
        RegionGraphKindV1::Inference,
        CanonicalKernelStopReason::MaxIterations,
        2,
        2,
        0.01,
        0.5,
        residual.current_value,
        vec![residual.residual_id.clone()],
        stop,
        true,
    );
    let syndrome = KernelSyndromeReportV1::contradiction(
        graph_id.clone(),
        region_id.clone(),
        ArtifactId("witness:p10-contradiction".into()),
        vec![accepted_claim.clone()],
    );
    let approximate = DisplayDigestV1::for_json_value(&serde_json::json!({"value": "approx"}));
    let exact = DisplayDigestV1::for_json_value(&serde_json::json!({"value": "exact"}));
    let oracle = OracleSliceRequestV1::new(
        graph_id,
        region_id,
        region.node_ids,
        4,
        approximate,
        exact,
        Some(0.25),
    );
    let run = KernelRunDisplayReportV1::new(
        &graph,
        &convergence,
        std::slice::from_ref(&syndrome),
        std::slice::from_ref(&oracle),
    );
    let support = SupportCoreV1::new(
        vec![accepted_claim.clone()],
        Vec::new(),
        Vec::new(),
        vec![run.receipt_id.clone()],
        Vec::new(),
        Vec::new(),
    );
    let frontier = RemovalFrontierV1::new(
        &support,
        vec![accepted_claim],
        Vec::new(),
        vec![run.receipt_id.clone()],
        &InvariantBudgetV1::full_history(),
    );

    assert!(residual.blocks_promotion_as_exact());
    assert!(syndrome.requires_canonical_repair());
    assert!(syndrome.blocks_promotion_as_exact());
    assert!(convergence.degraded);
    assert!(convergence.blocks_promotion_as_exact());
    assert!(convergence.has_explicit_stop_rule_evidence());
    assert_eq!(oracle.agreement, OracleAgreementV1::BoundedDisagreement);
    assert!(oracle.has_bounded_semantic_diff());
    assert!(run.degraded);
    assert!(!run.can_promote_as_exact_seed_result());
    assert!(frontier.is_blocked());
}

#[test]
fn p29_residual_syndrome_convergence_seed_stays_receipt_bearing() {
    let graph_id = ArtifactId("graph:p29-kernel".into());
    let region_id = ArtifactId("region:p29-kernel".into());
    let stop =
        KernelStopRuleReportV1::new(CanonicalKernelStopReason::FixedPoint, 4, 8, 0.05, 0.5, 0.02);
    let residual = KernelResidualReportV1::new(
        graph_id.clone(),
        region_id.clone(),
        4,
        0.2,
        0.02,
        0.05,
        stop.clone(),
    );
    let syndrome = KernelSyndromeReportV1::contradiction(
        graph_id.clone(),
        region_id,
        ArtifactId("witness:p29-contradiction".into()),
        vec![ArtifactId("claim:p29-affected".into())],
    );
    let convergence = ConvergenceReportV1::new(
        graph_id,
        RegionGraphKindV1::Inference,
        CanonicalKernelStopReason::FixedPoint,
        4,
        8,
        0.05,
        0.5,
        residual.current_value,
        vec![residual.residual_id.clone()],
        stop,
        false,
    );

    assert!(residual.converged);
    assert!(convergence.converged);
    assert!(!convergence.degraded);
    assert!(convergence.has_explicit_stop_rule_evidence());
    assert!(syndrome.requires_canonical_repair());
}

#[test]
fn p29_lawful_subtraction_seed_blocks_support_loss_and_allows_safe_dry_run() {
    let accepted_claim = ArtifactId("claim:p29-accepted".into());
    let evidence = ArtifactId("evidence:p29-accepted".into());
    let support = SupportCoreV1::new(
        vec![accepted_claim.clone()],
        Vec::new(),
        vec![evidence.clone()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let budget = InvariantBudgetV1::full_history().with_removal_limits(1, 1, 0);
    let blocked_frontier = RemovalFrontierV1::new(
        &support,
        vec![accepted_claim.clone()],
        vec![evidence],
        Vec::new(),
        &budget,
    );
    let blocked_plan = SubtractionPlanV1::dry_run(
        SubtractionOperatorV1::SupportCoreExtraction,
        &support,
        &blocked_frontier,
        &budget,
    );
    assert!(blocked_frontier.is_blocked());
    assert!(blocked_plan.blocked);
    assert!(!blocked_plan.is_lawful_append_only_reduction());

    let superseded_support = SupportCoreV1::new(
        vec![accepted_claim.clone()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![accepted_claim.clone()],
        Vec::new(),
    );
    let safe_frontier = RemovalFrontierV1::new(
        &superseded_support,
        vec![accepted_claim],
        Vec::new(),
        Vec::new(),
        &budget,
    );
    let safe_plan = SubtractionPlanV1::dry_run(
        SubtractionOperatorV1::SupportCoreExtraction,
        &superseded_support,
        &safe_frontier,
        &budget,
    );
    assert!(!safe_frontier.is_blocked());
    assert!(safe_plan.is_lawful_append_only_reduction());
    assert!(!safe_plan.can_mutate_runtime_state());
}

#[test]
fn p28_v11c_external_admission_defaults_to_quarantine() {
    let source = ArtifactId("external-artifact:p28".into());
    let decision = ExternalArtifactAdmissionDecisionV1::default_quarantine(source.clone());

    assert_eq!(decision.kind, ArtifactKindV1::AdmissionDecision);
    assert_eq!(decision.source_artifact_id, source);
    assert_eq!(
        decision.disposition,
        ExternalAdmissionDispositionV1::Quarantined
    );
    assert_eq!(decision.activation_level, V11ActivationLevelV1::Quarantined);
    assert!(!decision.truth_promotion_allowed);
    assert!(!decision.proof_waiver_allowed);
    assert!(decision
        .reason_codes
        .contains(&"external-admission-defaults-to-quarantine".into()));
}

#[test]
fn p28_learned_or_advisory_system_cannot_promote_truth_or_waive_proof() {
    let learned = AdvisorySystemPromotionGuardV1::advisory(AdvisorySystemKindV1::LearnedRanker);
    let heuristic =
        AdvisorySystemPromotionGuardV1::advisory(AdvisorySystemKindV1::HeuristicAdvisor);

    for guard in [learned, heuristic] {
        assert_eq!(guard.activation_level, V11ActivationLevelV1::AdvisoryOnly);
        assert!(!guard.truth_promotion_allowed);
        assert!(!guard.proof_waiver_allowed);
        assert!(!guard.activation_level.allows_truth_promotion());
        assert!(guard
            .reason_codes
            .contains(&"advisory-system-cannot-waive-proof".into()));
    }
}

#[test]
fn p19_golden_fixtures_deserialize() {
    let manifest = include_str!("../../../tests/fixtures/p19/release_artifact_manifest_v1.json");
    let manifest: ReleaseArtifactManifestV1 = serde_json::from_str(manifest).unwrap();
    assert_eq!(manifest.kind, ArtifactKindV1::ReleaseArtifactManifest);
    assert!(!manifest.blocks_release());

    let matrix = include_str!("../../../tests/fixtures/p19/cross_pass_traceability_matrix_v1.json");
    let matrix: CrossPassTraceabilityMatrixV1 = serde_json::from_str(matrix).unwrap();
    assert_eq!(matrix.kind, ArtifactKindV1::CrossPassTraceabilityMatrix);
    assert!(!matrix.blocks_completion());

    let limitations =
        include_str!("../../../tests/fixtures/p19/known_limitations_register_v1.json");
    let limitations: KnownLimitationsRegisterV1 = serde_json::from_str(limitations).unwrap();
    assert_eq!(limitations.kind, ArtifactKindV1::KnownLimitationsRegister);
    assert!(!limitations.blocks_completion());

    let debt = include_str!("../../../tests/fixtures/p19/regression_debt_ledger_v1.json");
    let debt: RegressionDebtLedgerV1 = serde_json::from_str(debt).unwrap();
    assert_eq!(debt.kind, ArtifactKindV1::RegressionDebtLedger);
    assert!(!debt.blocks_completion());

    let report = include_str!("../../../tests/fixtures/p19/completion_audit_report_v1.json");
    let report: CompletionAuditReportV1 = serde_json::from_str(report).unwrap();
    assert_eq!(report.kind, ArtifactKindV1::CompletionAuditReport);
    assert!(report.release_bar_passed);
    assert_eq!(
        report.completion_state,
        CompletionAuditStateV1::DeferredHorizon
    );
    assert!(!report.deferred_surfaces.is_empty());
}
