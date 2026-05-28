use super::*;

pub fn agent_validate_command(spec: &str) -> Result<String> {
    let (spec_path, spec_value, agent_spec) = load_agent_spec_file(spec)?;
    let schema = generated_schema_for_rust_type("AgentSpecV1")?;
    let schema_validation = validate_json_schema(&schema, &spec_value);
    let support_label = agent_spec.support_label.to_string();
    let validation = match agent_spec.validate() {
        Ok(()) => serde_json::json!({
            "valid": true,
            "reason_codes": [],
        }),
        Err(reason_codes) => serde_json::json!({
            "valid": false,
            "reason_codes": reason_codes,
        }),
    };
    let validation_valid = validation
        .get("valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut degradation = Vec::new();
    if !schema_validation.valid {
        degradation.push("agent-spec-schema-validation-failed".into());
    }
    if !validation_valid {
        degradation.push("agent-spec-policy-validation-failed".into());
    }
    let semantic_status = if schema_validation.valid && validation_valid {
        "exact_check"
    } else {
        "failed_exact_check"
    };
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "schema": "AgentSpecValidationReportV1",
        "spec_path": spec_path,
        "agent_id": agent_spec.agent_id,
        "display_name": agent_spec.display_name,
        "profile": agent_spec.profile,
        "support_label": support_label.clone(),
        "agent_spec_digest": DisplayDigestV1::for_json_value(&spec_value),
        "schema_validation": schema_validation,
        "validation": validation,
        "semantic_disclosure": semantic_disclosure_value(
            semantic_status,
            support_label,
            degradation,
            vec![
                "strict-json-parse-with-duplicate-key-rejection".into(),
                "generated-json-schema-validation".into(),
                "AgentSpecV1-policy-validation".into(),
            ],
            vec![
                "validation is an AiDENs-local operator check and does not create canonical memory, tool, or verification truth".into(),
            ],
        ),
        "canonical_ownership": {
            "aidens_role": "consumer-orchestrator",
            "memory_truth_owner": "semantic-memory",
            "tool_receipt_owner": "llm-tool-runtime",
            "trace_identity_owner": "stack-ids",
            "verification_truth_owner": "verification-control/verification-policy",
        }
    }))?)
}

pub fn agent_doctor_command(spec: &str) -> Result<String> {
    let (spec_path, spec_value, agent_spec) = load_agent_spec_file(spec)?;
    let validation_reasons = agent_spec.validate().err().unwrap_or_default();
    let mut blocked = validation_reasons.clone();
    if agent_spec.provider_policy.cloud_allowed {
        blocked.push("cloud-provider-execution-blocked-for-supported-local-agent".into());
    }
    if matches!(
        agent_spec.permit_policy.network,
        AgentPermitRuleV1::OperatorApproved
    ) {
        blocked.push("network-authority-blocked-for-p26-local-agent".into());
    }
    let support_label = agent_spec.support_label.to_string();
    let semantic_status = if blocked.is_empty() {
        "exact_check"
    } else {
        "degraded_exact_check"
    };
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "schema": "AgentSpecDoctorReportV1",
        "spec_path": spec_path,
        "agent_id": agent_spec.agent_id,
        "support_label": support_label.clone(),
        "agent_spec_digest": DisplayDigestV1::for_json_value(&spec_value),
        "valid": validation_reasons.is_empty(),
        "blocked_checks": blocked.clone(),
        "memory_policy": agent_spec.memory_policy,
        "tool_policy": agent_spec.tool_policy,
        "permit_policy": agent_spec.permit_policy,
        "verification_policy": agent_spec.verification_policy,
        "support_disclosure": {
            "supported_local": ["mock/local provider route", "sandboxed coding tools", "permit-gated writes", "run-bundle-v3 evidence"],
            "partial": ["memory grounding through canonical seam when enabled"],
            "deferred": ["cloud provider execution", "broad autonomous daemon behavior", "V10 runtime geometry"]
        },
        "semantic_disclosure": semantic_disclosure_value(
            semantic_status,
            support_label,
            blocked,
            vec![
                "AgentSpecV1-policy-validation".into(),
                "supported-local-cloud-boundary-check".into(),
                "permit-policy-network-boundary-check".into(),
            ],
            vec![
                "doctor output is an AiDENs-local operator diagnostic; canonical policy truth remains delegated".into(),
            ],
        ),
        "consumer_only": true
    }))?)
}

pub fn agent_new_command(template: &str, out: &str) -> Result<String> {
    if template != "local-coding" {
        bail!("agent new currently supports only --template local-coding");
    }
    let out_dir = resolve_output_path(out)?;
    if out_dir.exists() {
        bail!(
            "agent new refuses to overwrite existing directory: {}",
            out_dir.display()
        );
    }
    std::fs::create_dir_all(out_dir.join("sandbox"))
        .with_context(|| format!("failed to create {}", out_dir.join("sandbox").display()))?;
    let spec = local_coding_agent_spec_value("agent:local-coding-example", "Local Coding Agent");
    write_json_file(&out_dir.join("agent.json"), &spec)?;
    std::fs::write(
        out_dir.join("task.md"),
        "Read README.md, search for AiDENs, and propose a bounded local patch. Apply only with a scoped permit.\n",
    )
    .with_context(|| format!("failed to write {}", out_dir.join("task.md").display()))?;
    std::fs::write(
        out_dir.join("sandbox").join("README.md"),
        "# AiDENs Local Coding Sandbox\n\nAiDENs fixture content for local agent read/search/propose evidence.\n",
    )
    .with_context(|| format!("failed to write {}", out_dir.join("sandbox/README.md").display()))?;
    std::fs::write(
        out_dir.join("README.md"),
        "# AiDENs AgentSpecV1 Local Coding Example\n\nRun with:\n\n```bash\ncargo run -p aidens-cli -- agent validate --spec agent.json\ncargo run -p aidens-cli -- agent run --spec agent.json --task task.md --sandbox-root sandbox --out target/run\ncargo run -p aidens-cli -- agent inspect --run target/run\n```\n",
    )
    .with_context(|| format!("failed to write {}", out_dir.join("README.md").display()))?;
    Ok(format!(
        "AiDENs agent new\ntemplate: {template}\noutput: {}\nspec: {}\ntask: {}",
        out_dir.display(),
        out_dir.join("agent.json").display(),
        out_dir.join("task.md").display()
    ))
}

pub fn agent_run_command(
    spec: &str,
    task: &str,
    out: &str,
    sandbox_root: Option<String>,
    permit_json: Option<String>,
    mock_response: Option<String>,
) -> Result<String> {
    let started_at = Utc::now();
    let (spec_path, spec_value, agent_spec) = load_agent_spec_file(spec)?;
    if let Err(reason_codes) = agent_spec.validate() {
        bail!("AgentSpecV1 validation failed: {}", reason_codes.join(","));
    }
    let spec_root = spec_path.parent().unwrap_or_else(|| Path::new("."));
    let task_path = resolve_input_or_spec_relative_path(task, spec_root)?;
    let task_text = if task_path.exists() {
        std::fs::read_to_string(&task_path)
            .with_context(|| format!("failed to read task {}", task_path.display()))?
    } else {
        task.to_string()
    };
    if task_text.trim().is_empty() {
        bail!("agent run requires a non-empty task");
    }
    let out_dir = resolve_output_path(out)?;
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create output directory {}", out_dir.display()))?;
    let sandbox_root = sandbox_root
        .map(|root| resolve_input_or_spec_relative_path(root, spec_root))
        .transpose()?
        .unwrap_or_else(|| spec_root.join("sandbox"))
        .canonicalize()
        .with_context(|| "agent run sandbox_root must exist; pass --sandbox-root or create a sandbox directory next to the spec")?;
    let receipt_root = out_dir.join("receipts");
    let event_log_path = out_dir.join("event-log.ndjson");
    let permit_policy = permit_json
        .as_deref()
        .map(permit_policy_from_json)
        .transpose()?
        .unwrap_or_default();
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&sandbox_root)?;
    let mock_response = mock_response.unwrap_or_else(|| {
        agent_default_mock_response(&agent_spec, &sandbox_root, &task_text)
            .unwrap_or_else(|_| "AgentSpecV1 local agent completed without tool calls.".into())
    });
    let runtime = tokio::runtime::Runtime::new()?;
    let loop_output = runtime.block_on(async {
        PlanActVerifyLoopV1::new(agent_spec.clone())
            .app_id(receipt_store_segment(&agent_spec.agent_id))
            .tools(registry)
            .permit_policy(permit_policy)
            .provider_mock_response(mock_response)
            .canonical_receipt_log_config(CanonicalEventLogConfig::for_root(&receipt_root))
            .execute(task_text.clone())
            .await
    })?;
    write_agent_loop_artifacts(&out_dir, &event_log_path, &loop_output)?;

    let run_id = format!(
        "{}-agent-local",
        receipt_store_segment(&agent_spec.agent_id)
    );
    let elapsed_ms = (Utc::now() - started_at).num_milliseconds().max(0);
    let run_output = loop_output.run_output.as_ref();
    let trace_ctx = run_output
        .map(|output| output.receipt.context.stack_trace_ctx())
        .unwrap_or_else(aidens_contracts::StackTraceCtx::generate);
    let attempt_id = run_output
        .map(|output| output.receipt.context.stack_attempt_id())
        .unwrap_or_else(StackAttemptId::generate);
    let trial_id = StackTrialId::generate();
    let attempt_family_id = run_output
        .map(|output| output.receipt.context.attempt_family_id.clone())
        .unwrap_or_else(|| generated_artifact_id_from_material("attempt-family", &run_id));
    let mut execution_context =
        aidens_contracts::canonical_stack::ForgeExecutionContextV1::new(trace_ctx.clone());
    execution_context.attempt_id = Some(attempt_id.clone());
    execution_context.trial_id = Some(trial_id.clone());
    execution_context.replay_link = Some(format!(
        "cargo run -p aidens-cli -- agent run --spec {} --task {} --sandbox-root {} --out {}",
        spec_path.display(),
        task_path.display(),
        sandbox_root.display(),
        out_dir.display()
    ));
    execution_context.workload_class = Some("agent-spec-supported-local".into());
    execution_context.deadline = Some(format!(
        "{}ms",
        agent_spec
            .budget_policy
            .deadline_seconds
            .saturating_mul(1000)
    ));
    execution_context.cost_budget_units = Some(agent_spec.budget_policy.max_tool_calls.into());
    execution_context.provider_route = Some("mock".into());
    execution_context.degradation_markers = agent_degradation_markers(&loop_output);
    execution_context.dispatch_outcome =
        if matches!(loop_output.outcome, PlanActVerifyOutcomeV1::Success) {
            aidens_contracts::canonical_stack::ForgeDispatchOutcomeV1::Succeeded
        } else if loop_output.abstention_receipt.is_some() {
            aidens_contracts::canonical_stack::ForgeDispatchOutcomeV1::Failed
        } else {
            aidens_contracts::canonical_stack::ForgeDispatchOutcomeV1::Degraded
        };
    execution_context.environment_fingerprint = Some(
        StackContentDigest::compute_str(&format!(
            "{}:{}:{}",
            agent_spec.agent_id,
            sandbox_root.display(),
            out_dir.display()
        ))
        .hex()
        .to_string(),
    );

    let event_log = event_log_digest(
        &event_log_path,
        run_output
            .map(|output| output.durable_receipt_records.len())
            .unwrap_or(0),
        0,
    )?;
    let replay = AiDENsRunReplayNormalizationV1 {
        replay_command: execution_context.replay_link.clone().unwrap_or_default(),
        fixture_path: task_path.exists().then(|| task_path.display().to_string()),
        normalized_fields: vec![
            "created_at".into(),
            "receipt_id".into(),
            "recorded_at".into(),
            "content_digest".into(),
        ],
        deterministic_compare: true,
        normalized_digest: event_log.replay_normalized_digest.clone(),
        reason_codes: vec!["timestamp-and-id-normalized".into()],
    };
    let failure = agent_failure_taxonomy(&loop_output);
    let budget = AiDENsRunBudgetDeadlineV1 {
        max_steps: agent_spec.budget_policy.max_turns,
        max_tool_calls: agent_spec.budget_policy.max_tool_calls,
        max_retries: 2,
        max_turn_millis: agent_spec
            .budget_policy
            .deadline_seconds
            .saturating_mul(1000),
        elapsed_ms,
        deadline: execution_context.deadline.clone(),
        cost_budget_units: Some(agent_spec.budget_policy.max_tool_calls.into()),
        degradation_markers: execution_context.degradation_markers.clone(),
    };
    let support = AiDENsRunSupportTierEvidenceV1 {
        support_tier: agent_spec.support_label.to_string(),
        supported: vec![
            "AgentSpecV1 validation".into(),
            "bounded PlanActVerifyLoopV1".into(),
            "sandboxed local coding tools".into(),
            "AiDENsRunBundleV3 inspection".into(),
        ],
        partial: vec!["canonical memory grounding evidence when enabled".into()],
        deferred: vec![
            "cloud-provider-execution".into(),
            "broad-autonomous-daemon-behavior".into(),
            "V10-runtime-geometry".into(),
        ],
        reason_codes: vec!["p26-supported-local-agent-spine".into()],
    };
    let memory_grounding = if loop_output.memory_grounding_receipts.is_empty() {
        vec![if agent_spec.memory_policy.enabled {
            "memory-grounding:enabled-no-receipt".into()
        } else {
            "memory-grounding:disabled-by-agent-spec".into()
        }]
    } else {
        loop_output.memory_grounding_receipts.clone()
    };
    let outputs = vec![
        out_dir.join("final.txt").display().to_string(),
        out_dir
            .join("plan-act-verify-output.json")
            .display()
            .to_string(),
        out_dir.join("event-log.ndjson").display().to_string(),
        receipt_root
            .join("canonical-receipts.ndjson")
            .display()
            .to_string(),
    ];
    let blocked_checks = loop_output
        .verification_receipts
        .iter()
        .filter(|receipt| !receipt.passed)
        .map(|receipt| receipt.check.clone())
        .collect::<Vec<_>>();
    let mut bundle = loop_output.assemble_v3_bundle(
        run_id.clone(),
        agent_spec.profile.clone(),
        execution_context,
        event_log,
        budget,
        support,
        vec![
            agent_spec.support_label.to_string(),
            "supported-local".into(),
        ],
        replay,
        failure,
        attempt_family_id,
        attempt_id,
        trial_id,
        DisplayDigestV1::for_json_value(&spec_value),
        memory_grounding,
        outputs,
        vec![
            "re-run agent run with the same AgentSpecV1, task, sandbox root, and permit JSON"
                .into(),
            "inspect run-bundle.json and canonical receipt log before claiming replay success"
                .into(),
        ],
        blocked_checks,
    );
    if let Some(output) = run_output {
        bundle.provider_receipts = vec![output.receipt.receipt_id.to_string()];
        bundle.tool_receipts = output
            .receipt
            .tool_invocation_receipts
            .iter()
            .map(|receipt| receipt.receipt_id.to_string())
            .collect();
        let mut permit_receipts: Vec<String> = output
            .receipt
            .permit_use_receipts
            .iter()
            .map(|receipt| receipt.receipt_id.to_string())
            .collect();
        if permit_receipts.is_empty() {
            permit_receipts = output
                .receipt
                .approval_requests
                .iter()
                .map(|request| request.request_id.to_string())
                .collect();
        }
        bundle.permit_receipts = permit_receipts;
    }
    bundle.support_labels.sort();
    bundle.support_labels.dedup();
    let bundle_store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&receipt_root))?;
    let bundle_store_record = bundle_store.write_bundle(&bundle)?;
    write_json_file(&out_dir.join("run-bundle.json"), &bundle)?;
    write_json_file(
        &out_dir.join("run-bundle-store-record.json"),
        &bundle_store_record,
    )?;
    write_agent_final_artifact(&out_dir, &loop_output)?;
    Ok(format!(
        "AiDENs agent run\nspec: {}\ntask: {}\nsandbox: {}\noutput: {}\nrun_bundle: {}\nrun_bundle_store: {}\noutcome: {:?}",
        spec_path.display(),
        task_path.display(),
        sandbox_root.display(),
        out_dir.display(),
        out_dir.join("run-bundle.json").display(),
        bundle_store_record.bundle_path.display(),
        loop_output.outcome
    ))
}

fn load_agent_spec_file(path: &str) -> Result<(PathBuf, Value, AgentSpecV1)> {
    let spec_path = resolve_cli_path(path)?;
    let raw = std::fs::read_to_string(&spec_path)
        .with_context(|| format!("failed to read AgentSpecV1 {}", spec_path.display()))?;
    let value = parse_strict_json(&raw)
        .with_context(|| format!("failed strict-parse AgentSpecV1 {}", spec_path.display()))?;
    let spec: AgentSpecV1 = serde_json::from_value(value.clone())
        .with_context(|| format!("failed to decode AgentSpecV1 {}", spec_path.display()))?;
    Ok((spec_path, value, spec))
}

fn generated_schema_for_rust_type(rust_type: &str) -> Result<Value> {
    generated_schema_documents()
        .into_iter()
        .find(|document| document.registration.rust_type == rust_type)
        .map(|document| document.schema)
        .ok_or_else(|| anyhow::anyhow!("generated schema missing for {rust_type}"))
}

fn resolve_input_or_spec_relative_path(
    path: impl AsRef<Path>,
    spec_root: &Path,
) -> Result<PathBuf> {
    let path = path.as_ref();
    if path.is_absolute() || path.exists() {
        return resolve_cli_path(path);
    }
    Ok(spec_root.join(path))
}

fn local_coding_agent_spec_value(agent_id: &str, display_name: &str) -> Value {
    serde_json::json!({
        "schema": "AgentSpecV1",
        "agent_id": agent_id,
        "display_name": display_name,
        "support_label": "supported-local",
        "profile": "coding",
        "provider_policy": {
            "provider": "local",
            "cloud_allowed": false,
            "fallback_allowed": false
        },
        "memory_policy": {
            "enabled": false,
            "mode": "canonical-seam",
            "requires_view_disclosure": false
        },
        "tool_policy": {
            "allowed_tools": [
                "repo.read",
                "repo.list",
                "repo.search",
                "patch.propose",
                "patch.apply",
                "checks.run",
                "run.inspect"
            ],
            "write_tools_require_permit": true
        },
        "permit_policy": {
            "writes": "operator-approved",
            "commands": "operator-approved",
            "network": "forbidden"
        },
        "verification_policy": {
            "required_checks": ["schema", "sandbox", "digest", "support-claim"],
            "fail_closed": true
        },
        "evidence_policy": {
            "emit_run_bundle": true,
            "emit_tool_receipts": true,
            "emit_permit_receipts": true,
            "emit_abstention_receipts": true
        },
        "budget_policy": {
            "max_turns": 4,
            "max_tool_calls": 8,
            "deadline_seconds": 120
        }
    })
}

fn agent_default_mock_response(
    spec: &AgentSpecV1,
    sandbox_root: &Path,
    task_text: &str,
) -> Result<String> {
    let tools = spec
        .tool_policy
        .allowed_tools
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let lower_task = task_text.to_ascii_lowercase();
    if lower_task.contains("apply") && tools.contains("patch.apply") {
        let read_path = coding_agent_read_path(sandbox_root)?;
        let diff = coding_agent_patch_diff(sandbox_root, &read_path)?;
        return agent_tool_mock_script(
            "aidens:patch-apply:1",
            serde_json::json!({ "diff": diff }),
            "patch apply finished with permit-gated evidence: {{last_tool_output_text}}",
        );
    }
    if (lower_task.contains("check") || lower_task.contains("inspect"))
        && (tools.contains("checks.run") || tools.contains("run.inspect"))
    {
        return agent_tool_mock_script(
            "aidens:run-checks:1",
            serde_json::json!({ "command": ["bash", "scripts/verify.sh"] }),
            "check finished with permit-gated evidence: {{last_tool_output_text}}",
        );
    }
    if lower_task.contains("search") && tools.contains("repo.search") {
        return agent_tool_mock_script(
            "aidens:repo-search:1",
            serde_json::json!({
                "query": agent_search_query(task_text),
                "path": ".",
            }),
            "search finished with sandbox evidence: {{last_tool_output_text}}",
        );
    }
    if lower_task.contains("list") && tools.contains("repo.list") {
        return agent_tool_mock_script(
            "aidens:repo-list:1",
            serde_json::json!({ "path": ".", "max_entries": 50 }),
            "list finished with sandbox evidence: {{last_tool_output_text}}",
        );
    }
    if lower_task.contains("propose") && tools.contains("patch.propose") {
        let read_path = coding_agent_read_path(sandbox_root)?;
        let diff = coding_agent_patch_diff(sandbox_root, &read_path)?;
        return agent_tool_mock_script(
            "aidens:patch-propose:1",
            serde_json::json!({
                "summary": "AgentSpecV1 local patch proposal",
                "diff": diff,
            }),
            "proposal finished without mutation: {{last_tool_output_text}}",
        );
    }
    if tools.contains("repo.read") {
        let read_path = coding_agent_read_path(sandbox_root)?;
        return agent_tool_mock_script(
            "aidens:repo-read:1",
            serde_json::json!({ "path": read_path }),
            "read finished with sandbox evidence: {{last_tool_content}}",
        );
    }
    Ok("AgentSpecV1 local agent completed without tool calls.".into())
}

fn agent_tool_mock_script(tool_id: &str, input: Value, final_text: &str) -> Result<String> {
    Ok(format!(
        "{}{}{}",
        serde_json::to_string(&serde_json::json!({
            "tool_call": {
                "tool_id": tool_id,
                "input": input,
            }
        }))?,
        TEST_AGENT_MOCK_RESPONSE_DELIMITER,
        final_text
    ))
}

fn agent_search_query(task_text: &str) -> String {
    task_text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .find(|part| part.len() >= 3)
        .unwrap_or("AiDENs")
        .to_string()
}

fn write_agent_loop_artifacts(
    out_dir: &Path,
    event_log_path: &Path,
    output: &PlanActVerifyLoopV1Output,
) -> Result<()> {
    write_json_file(
        &out_dir.join("plan-act-verify-output.json"),
        &agent_loop_output_json(output),
    )?;
    if let Some(run) = output.run_output.as_ref() {
        write_json_file(&out_dir.join("run-report.json"), &run.receipt)?;
        write_json_file(&out_dir.join("turn-report.json"), &run.turn_receipt)?;
        write_json_file(&out_dir.join("tool-exposure.json"), &run.tool_exposure)?;
        write_json_file(
            &out_dir.join("agency-policy-reports.json"),
            &run.agency_policy_reports,
        )?;
    }
    if let Some(abstention) = output.abstention_receipt.as_ref() {
        write_json_file(
            &out_dir.join("abstention.json"),
            &agent_abstention_json(abstention),
        )?;
    }
    if let Some(repair) = output.repair_plan.as_ref() {
        write_json_file(
            &out_dir.join("repair-plan.json"),
            &agent_repair_json(repair),
        )?;
    }
    write_agent_event_log(event_log_path, output)
}

fn write_agent_final_artifact(out_dir: &Path, output: &PlanActVerifyLoopV1Output) -> Result<()> {
    let final_text = output
        .run_output
        .as_ref()
        .map(|run| run.text.clone())
        .or_else(|| {
            output
                .abstention_receipt
                .as_ref()
                .map(|receipt| format!("abstained: {}", receipt.reason_code))
        })
        .unwrap_or_else(|| format!("outcome: {:?}", output.outcome));
    std::fs::write(out_dir.join("final.txt"), final_text)
        .with_context(|| format!("failed to write {}", out_dir.join("final.txt").display()))
}

fn agent_loop_output_json(output: &PlanActVerifyLoopV1Output) -> Value {
    let degradation = agent_degradation_markers(output);
    let semantic_status = if degradation.is_empty() {
        "exact_check"
    } else {
        "degraded_exact_check"
    };
    serde_json::json!({
        "schema": "PlanActVerifyLoopV1OutputDisplay",
        "agent_id": output.agent_id,
        "app_id": output.app_id,
        "profile": output.profile,
        "support_label": output.support_label,
        "outcome": format!("{:?}", output.outcome),
        "turns_used": output.turns_used,
        "max_turns": output.max_turns,
        "plan_receipts": output.plan_receipts.iter().map(|receipt| serde_json::json!({
            "receipt_id": receipt.receipt_id,
            "plan_id": receipt.plan_id,
            "step": receipt.step,
            "action": receipt.action,
            "reason_codes": receipt.reason_codes,
        })).collect::<Vec<_>>(),
        "tool_route_receipts": output.tool_route_receipts.iter().map(|receipt| serde_json::json!({
            "receipt_id": receipt.receipt_id,
            "plan_id": receipt.plan_id,
            "step": receipt.step,
            "requested_tool_ids": receipt.requested_tool_ids,
            "exposed_tool_ids": receipt.exposed_tool_ids,
            "reason_codes": receipt.reason_codes,
        })).collect::<Vec<_>>(),
        "tool_call_receipts": output.tool_call_receipts.iter().map(|receipt| serde_json::json!({
            "receipt_id": receipt.receipt_id,
            "plan_id": receipt.plan_id,
            "step": receipt.step,
            "run_id": receipt.run_id,
            "permitted_tool_calls": receipt.permitted_tool_calls,
            "blocked_tool_calls": receipt.blocked_tool_calls,
            "succeeded_tool_calls": receipt.succeeded_tool_calls,
            "failed_tool_calls": receipt.failed_tool_calls,
            "reason_codes": receipt.reason_codes,
        })).collect::<Vec<_>>(),
        "verification_receipts": output.verification_receipts.iter().map(|receipt| serde_json::json!({
            "receipt_id": receipt.receipt_id,
            "step": receipt.step,
            "check": receipt.check,
            "passed": receipt.passed,
            "reason_codes": receipt.reason_codes,
        })).collect::<Vec<_>>(),
        "memory_grounding_receipts": output.memory_grounding_receipts,
        "abstention_receipt": output.abstention_receipt.as_ref().map(agent_abstention_json),
        "repair_plan": output.repair_plan.as_ref().map(agent_repair_json),
        "finalization": output.finalization.as_ref().map(|receipt| serde_json::json!({
            "receipt_id": receipt.receipt_id,
            "step": receipt.step,
            "outcome": receipt.outcome,
            "final_state": receipt.final_state,
            "blocked": receipt.blocked,
            "support_label": receipt.support_label,
            "reason_codes": receipt.reason_codes,
        })),
        "semantic_disclosure": semantic_disclosure_value(
            semantic_status,
            output.support_label.clone(),
            degradation,
            vec![
                "plan-receipts-emitted".into(),
                "tool-route-and-call-receipts-emitted".into(),
                "verification-receipts-emitted".into(),
                "canonical-owner-disclosure-emitted".into(),
            ],
            vec![
                "display output summarizes local runner receipts; receipt truth remains in canonical owner artifacts".into(),
            ],
        ),
        "canonical_ownership": {
            "memory_truth_owner": "semantic-memory/knowledge-runtime through aidens-memory-kit",
            "tool_receipt_owner": "llm-tool-runtime through aidens-tool-kit",
            "verification_truth_owner": "verification-control and verification-policy",
            "repair_truth_owner": "canonical repair/governance crates; this is display evidence only"
        }
    })
}

fn agent_abstention_json(receipt: &aidens_runner::AbstentionReceiptV1) -> Value {
    serde_json::json!({
        "receipt_id": receipt.receipt_id,
        "step": receipt.step,
        "reason_code": receipt.reason_code,
        "blocked_action": receipt.blocked_action,
        "evidence": receipt.evidence,
        "required_permits": receipt.required_permits,
        "can_resume": receipt.can_resume,
        "support_impact": receipt.support_impact,
    })
}

fn agent_repair_json(receipt: &aidens_runner::RepairPlanDisplayReceiptV1) -> Value {
    serde_json::json!({
        "repair_id": receipt.repair_id,
        "source_run_id": receipt.source_run_id,
        "failure_kind": receipt.failure_kind,
        "candidate_repair_actions": receipt.candidate_repair_actions,
        "required_verification": receipt.required_verification,
        "required_permits": receipt.required_permits,
        "risk_level": receipt.risk_level,
        "canonical_owner": receipt.canonical_owner,
        "display_only": true,
    })
}

fn write_agent_event_log(path: &Path, output: &PlanActVerifyLoopV1Output) -> Result<()> {
    let mut events = vec![serde_json::json!({
        "event": "agent_spec_loaded",
        "agent_id": output.agent_id,
        "support_label": output.support_label,
    })];
    for receipt in &output.plan_receipts {
        events.push(serde_json::json!({
            "event": "plan_receipt",
            "receipt_id": receipt.receipt_id,
            "step": receipt.step,
            "action": receipt.action,
            "reason_codes": receipt.reason_codes,
        }));
    }
    for receipt in &output.tool_route_receipts {
        events.push(serde_json::json!({
            "event": "tool_route_receipt",
            "receipt_id": receipt.receipt_id,
            "requested_tool_ids": receipt.requested_tool_ids,
            "exposed_tool_ids": receipt.exposed_tool_ids,
        }));
    }
    for receipt in &output.tool_call_receipts {
        events.push(serde_json::json!({
            "event": "tool_call_receipt",
            "receipt_id": receipt.receipt_id,
            "permitted_tool_calls": receipt.permitted_tool_calls,
            "blocked_tool_calls": receipt.blocked_tool_calls,
            "succeeded_tool_calls": receipt.succeeded_tool_calls,
            "failed_tool_calls": receipt.failed_tool_calls,
            "reason_codes": receipt.reason_codes,
        }));
    }
    for receipt in &output.verification_receipts {
        events.push(serde_json::json!({
            "event": "verification_receipt",
            "receipt_id": receipt.receipt_id,
            "check": receipt.check,
            "passed": receipt.passed,
            "reason_codes": receipt.reason_codes,
        }));
    }
    for receipt in &output.memory_grounding_receipts {
        events.push(serde_json::json!({
            "event": "memory_grounding_receipt",
            "receipt": receipt,
        }));
    }
    if let Some(receipt) = output.abstention_receipt.as_ref() {
        events.push(serde_json::json!({
            "event": "abstention_receipt",
            "receipt_id": receipt.receipt_id,
            "reason_code": receipt.reason_code,
            "blocked_action": receipt.blocked_action,
            "required_permits": receipt.required_permits,
        }));
    }
    if let Some(receipt) = output.repair_plan.as_ref() {
        events.push(serde_json::json!({
            "event": "repair_display_receipt",
            "repair_id": receipt.repair_id,
            "failure_kind": receipt.failure_kind,
            "canonical_owner": receipt.canonical_owner,
        }));
    }
    if let Some(receipt) = output.finalization.as_ref() {
        events.push(serde_json::json!({
            "event": "finalization_receipt",
            "receipt_id": receipt.receipt_id,
            "outcome": receipt.outcome,
            "final_state": receipt.final_state,
            "blocked": receipt.blocked,
            "reason_codes": receipt.reason_codes,
        }));
    }
    let mut lines = events
        .iter()
        .map(serde_json::to_string)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .join("\n");
    lines.push('\n');
    std::fs::write(path, lines).with_context(|| format!("failed to write {}", path.display()))
}

fn agent_degradation_markers(output: &PlanActVerifyLoopV1Output) -> Vec<String> {
    let mut markers = Vec::new();
    if !matches!(output.outcome, PlanActVerifyOutcomeV1::Success) {
        markers.push(format!("agent-outcome:{:?}", output.outcome));
    }
    if let Some(receipt) = output.abstention_receipt.as_ref() {
        markers.push(format!("abstention:{}", receipt.reason_code));
    }
    if let Some(receipt) = output.finalization.as_ref() {
        for reason in &receipt.reason_codes {
            if reason.contains("provider-policy-local-routed-to-explicit-mock-fixture") {
                markers.push(reason.clone());
            }
        }
    }
    for receipt in &output.verification_receipts {
        if !receipt.passed {
            markers.push(format!("verification-failed:{}", receipt.check));
        }
    }
    markers.sort();
    markers.dedup();
    markers
}

fn agent_failure_taxonomy(output: &PlanActVerifyLoopV1Output) -> AiDENsRunFailureTaxonomyV1 {
    let class = match output.outcome {
        PlanActVerifyOutcomeV1::Success => AiDENsRunFailureClassV1::None,
        PlanActVerifyOutcomeV1::Abstained => AiDENsRunFailureClassV1::OperatorAbstained,
        PlanActVerifyOutcomeV1::RepairNeeded => AiDENsRunFailureClassV1::VerificationUnavailable,
        PlanActVerifyOutcomeV1::Failed => AiDENsRunFailureClassV1::ToolFailed,
    };
    let mut reason_codes = output
        .finalization
        .as_ref()
        .map(|receipt| receipt.reason_codes.clone())
        .unwrap_or_default();
    if let Some(receipt) = output.abstention_receipt.as_ref() {
        reason_codes.push(receipt.reason_code.clone());
    }
    if reason_codes.is_empty() {
        reason_codes.push(format!("agent-outcome:{:?}", output.outcome));
    }
    reason_codes.sort();
    reason_codes.dedup();
    AiDENsRunFailureTaxonomyV1 {
        class,
        reason_codes,
        degraded: !matches!(output.outcome, PlanActVerifyOutcomeV1::Success),
        blocked: output
            .finalization
            .as_ref()
            .is_some_and(|receipt| receipt.blocked),
    }
}

pub fn inspect_run_bundle_command(dir: &str) -> Result<String> {
    let target = resolve_cli_path(dir)?;
    let bundle_path = if target.is_file() {
        target
    } else if target.join("run-bundle.json").exists() {
        target.join("run-bundle.json")
    } else if target.join("run-bundles").join("index.ndjson").exists() {
        RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&target))?
            .single_bundle_path()?
    } else {
        target.join("run-bundle.json")
    };
    let dir = bundle_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let bundle_text = std::fs::read_to_string(&bundle_path)
        .with_context(|| format!("failed to read {}", bundle_path.display()))?;
    let bundle: Value = parse_strict_json(&bundle_text)
        .with_context(|| format!("failed strict-parse {}", bundle_path.display()))?;
    let bundle_schema = bundle
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !matches!(
        bundle_schema,
        schema if schema == AiDENsRunBundleV2::SCHEMA || schema == AiDENsRunBundleV3::SCHEMA
    ) {
        bail!("inspect-run requires AiDENsRunBundleV2 or AiDENsRunBundleV3");
    }
    let schema_rust_type = if bundle_schema == AiDENsRunBundleV3::SCHEMA {
        "AiDENsRunBundleV3"
    } else {
        "AiDENsRunBundleV2"
    };
    let schema = generated_schema_for_rust_type(schema_rust_type)?;
    let schema_validation = validate_json_schema(&schema, &bundle);
    if !schema_validation.valid {
        bail!(
            "run bundle schema validation failed: {}",
            schema_validation.errors.join("; ")
        );
    }
    let mut required_pointers = vec![
        "/canonical_execution_context",
        "/canonical_execution_context/trace_ctx",
        "/trace_ctx",
        "/attempt_id",
        "/trial_id",
        "/event_log/digest",
        "/event_log/replay_normalized_digest",
        "/support/support_tier",
        "/replay/replay_command",
        "/failure/class",
    ];
    if bundle_schema == AiDENsRunBundleV3::SCHEMA {
        required_pointers.extend([
            "/attempt_family_id",
            "/agent_spec_digest",
            "/support_labels",
            "/memory_grounding_receipts",
            "/verification_receipts",
            "/replay_instructions",
        ]);
    }
    for pointer in required_pointers {
        if bundle.pointer(pointer).is_none() {
            bail!("run bundle missing required field {pointer}");
        }
    }
    let event_log_declared = bundle
        .pointer("/event_log/event_log_path")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("run bundle missing event_log_path"))?;
    let event_log_path = resolve_bundle_path(&dir, event_log_declared);
    let event_log_text = std::fs::read_to_string(&event_log_path)
        .with_context(|| format!("failed to read {}", event_log_path.display()))?;
    let event_log_digest = StackContentDigest::compute_str(&event_log_text);
    let event_log_digest_verified = bundle
        .pointer("/event_log/digest")
        .and_then(Value::as_str)
        .is_some_and(|expected| expected == event_log_digest.hex());
    let output_dir_receipt_log_path = dir.join("receipts").join("canonical-receipts.ndjson");
    let store_root = receipt_store_root_for_bundle_dir(&dir);
    let store_root_receipt_log_path = store_root.join("canonical-receipts.ndjson");
    let canonical_log_path = if output_dir_receipt_log_path.exists() {
        output_dir_receipt_log_path
    } else {
        store_root_receipt_log_path
    };
    let canonical_record_count = if canonical_log_path.exists() {
        std::fs::read_to_string(&canonical_log_path)
            .with_context(|| format!("failed to read {}", canonical_log_path.display()))?
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count()
    } else {
        0
    };
    let run_bundle_store_record_path = dir.join("run-bundle-store-record.json");
    let store_record = if run_bundle_store_record_path.exists() {
        Some(
            parse_strict_json(
                &std::fs::read_to_string(&run_bundle_store_record_path).with_context(|| {
                    format!("failed to read {}", run_bundle_store_record_path.display())
                })?,
            )
            .with_context(|| {
                format!(
                    "failed strict-parse {}",
                    run_bundle_store_record_path.display()
                )
            })?,
        )
    } else if bundle_schema == AiDENsRunBundleV3::SCHEMA {
        let run_id = bundle
            .get("run_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let store = RunBundleStore::open(RunBundleStoreConfig::for_receipt_root(&store_root))?;
        store
            .inspect(run_id)
            .ok()
            .and_then(|inspection| serde_json::to_value(inspection.record).ok())
    } else {
        None
    };
    let support_tier = bundle
        .pointer("/support/support_tier")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let mut semantic_degradation = bundle
        .pointer("/budget/degradation_markers")
        .and_then(Value::as_array)
        .map(|markers| {
            markers
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if bundle
        .pointer("/failure/degraded")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        semantic_degradation.push("failure-taxonomy-degraded".into());
    }
    if !event_log_digest_verified {
        semantic_degradation.push("event-log-digest-mismatch".into());
    }
    semantic_degradation.sort();
    semantic_degradation.dedup();
    let store_semantic_status = store_record
        .as_ref()
        .and_then(|record| record.get("semantic_status"))
        .and_then(Value::as_str)
        .unwrap_or("exact_check");
    let semantic_status = if !schema_validation.valid || !event_log_digest_verified {
        "failed_exact_check"
    } else if !semantic_degradation.is_empty() || store_semantic_status.contains("degraded") {
        "degraded_exact_check"
    } else {
        store_semantic_status
    };
    let semantic_disclosure = semantic_disclosure_value(
        semantic_status,
        support_tier.clone(),
        semantic_degradation,
        vec![
            "strict-json-parse-with-duplicate-key-rejection".into(),
            "generated-run-bundle-schema-validation".into(),
            "event-log-digest-verification".into(),
            "durable-run-bundle-store-record-inspection".into(),
        ],
        vec![
            "inspect-run is an AiDENs-local operator report; canonical receipt semantics remain in owner crates".into(),
        ],
    );
    let report = serde_json::json!({
        "schema": if bundle_schema == AiDENsRunBundleV3::SCHEMA {
            "AiDENsRunInspectReportV3"
        } else {
            "AiDENsRunInspectReportV2"
        },
        "bundle_dir": dir,
        "bundle_path": bundle_path,
        "bundle_schema": bundle.get("schema"),
        "run_id": bundle.get("run_id"),
        "support_tier": support_tier,
        "support": bundle.get("support"),
        "schema_validation": schema_validation,
        "support_labels": bundle.get("support_labels"),
        "trace_ctx": bundle.get("trace_ctx"),
        "attempt_family_id": bundle.get("attempt_family_id"),
        "attempt_id": bundle.get("attempt_id"),
        "trial_id": bundle.get("trial_id"),
        "agent_spec_digest": bundle.get("agent_spec_digest"),
        "provider_route": bundle.pointer("/canonical_execution_context/provider_route"),
        "budget": bundle.get("budget"),
        "degradation": bundle.pointer("/budget/degradation_markers"),
        "failure": bundle.get("failure"),
        "tool_receipts": bundle.get("tool_receipts"),
        "permit_receipts": bundle.get("permit_receipts"),
        "memory_grounding_receipts": bundle.get("memory_grounding_receipts"),
        "verification_receipts": bundle.get("verification_receipts"),
        "abstention_receipts": bundle.get("abstention_receipts"),
        "repair_plan_receipts": bundle.get("repair_plan_receipts"),
        "event_log_digest_verified": event_log_digest_verified,
        "event_log_digest": event_log_digest,
        "replay": bundle.get("replay"),
        "replay_instructions": bundle.get("replay_instructions"),
        "blocked_checks": bundle.get("blocked_checks"),
        "canonical_record_count": canonical_record_count,
        "canonical_record_log_path": canonical_log_path,
        "run_bundle_store_record": store_record,
        "semantic_disclosure": semantic_disclosure,
        "inspection_scope": "AiDENs-local operator report; canonical receipt semantics remain in owner crates.",
    });
    Ok(serde_json::to_string_pretty(&report)?)
}

fn receipt_store_root_for_bundle_dir(dir: &Path) -> PathBuf {
    for ancestor in dir.ancestors() {
        if ancestor.file_name().and_then(|name| name.to_str()) == Some("run-bundles") {
            return ancestor
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
        }
    }
    dir.parent()
        .unwrap_or_else(|| Path::new("."))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}
