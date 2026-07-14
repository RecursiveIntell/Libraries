use super::*;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct TestAgentFileV1 {
    pub(crate) agent: TestAgentSectionV1,
    pub(crate) provider: TestAgentProviderSectionV1,
    #[serde(default)]
    pub(crate) tools: TestAgentToolsSectionV1,
    #[serde(default)]
    pub(crate) receipts: TestAgentReceiptsSectionV1,
    #[serde(default)]
    pub(crate) agency: TestAgentAgencySectionV1,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct TestAgentSectionV1 {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) profile: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct TestAgentProviderSectionV1 {
    pub(crate) kind: String,
    pub(crate) fixture: String,
}

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct TestAgentToolsSectionV1 {
    #[serde(default)]
    pub(crate) expose: Vec<String>,
    #[serde(default)]
    pub(crate) enabled_bundles: Vec<String>,
    #[serde(default)]
    pub(crate) sandbox_root: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct TestAgentReceiptsSectionV1 {
    #[serde(default)]
    pub(crate) store_root: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct TestAgentAgencySectionV1 {
    #[serde(default)]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) require_receipts: bool,
}

impl Default for TestAgentAgencySectionV1 {
    fn default() -> Self {
        Self {
            enabled: true,
            require_receipts: true,
        }
    }
}

#[derive(Debug)]
pub(crate) struct TestAgentFixturePlan {
    pub(crate) user_prompt: String,
    pub(crate) mock_response: String,
    pub(crate) requested_tool_bundles: Vec<String>,
    pub(crate) seeded_files: Vec<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct TestAgentBundlePaths {
    pub(crate) final_text: PathBuf,
    pub(crate) run_bundle: PathBuf,
    pub(crate) run_report: PathBuf,
    pub(crate) turn_report: PathBuf,
    pub(crate) tool_exposure: PathBuf,
    pub(crate) agency_policy_reports: PathBuf,
    pub(crate) event_log: PathBuf,
    pub(crate) summary: PathBuf,
}

impl TestAgentBundlePaths {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self {
            final_text: root.join("final.txt"),
            run_bundle: root.join("run-bundle.json"),
            run_report: root.join("run-report.json"),
            turn_report: root.join("turn-report.json"),
            tool_exposure: root.join("tool-exposure.json"),
            agency_policy_reports: root.join("agency-policy-reports.json"),
            event_log: root.join("event-log.ndjson"),
            summary: root.join("summary.md"),
        }
    }
}

pub(crate) struct TestAgentRunBundleInput<'a> {
    pub(crate) run_id: &'a str,
    pub(crate) profile: &'a str,
    pub(crate) config_path: &'a Path,
    pub(crate) fixture_path: &'a Path,
    pub(crate) output_dir: &'a Path,
    pub(crate) receipt_root: &'a Path,
    pub(crate) output: &'a aidens_runner::AiDENsRunOutput,
    pub(crate) canonical_records: &'a [aidens_receipts::CanonicalEventLogEntry],
}

pub(crate) struct TestAgentSummaryInput<'a> {
    pub(crate) run_id: &'a str,
    pub(crate) config_path: &'a Path,
    pub(crate) fixture_path: &'a Path,
    pub(crate) output_dir: &'a Path,
    pub(crate) sandbox_root: &'a Path,
    pub(crate) receipt_root: &'a Path,
    pub(crate) seeded_files: &'a [PathBuf],
    pub(crate) final_text: &'a str,
    pub(crate) run_receipt_id: &'a str,
    pub(crate) turn_final_state: String,
    pub(crate) agency_report_count: usize,
    pub(crate) canonical_record_count: usize,
}

pub(crate) struct LocalRunBundleInput<'a> {
    pub(crate) run_id: &'a str,
    pub(crate) profile: &'a str,
    pub(crate) workload_class: &'a str,
    pub(crate) provider_route: Option<&'a str>,
    pub(crate) trace_ctx: Option<aidens_contracts::StackTraceCtx>,
    pub(crate) attempt_id: Option<aidens_contracts::StackAttemptId>,
    pub(crate) trial_id: Option<aidens_contracts::StackTrialId>,
    pub(crate) replay_command: String,
    pub(crate) fixture_path: Option<String>,
    pub(crate) output_dir: &'a Path,
    pub(crate) event_log_path: &'a Path,
    pub(crate) canonical_record_count: usize,
    pub(crate) event_count: usize,
    pub(crate) elapsed_ms: i64,
    pub(crate) degradation: Vec<String>,
    pub(crate) support: AiDENsRunSupportTierEvidenceV1,
    pub(crate) failure: AiDENsRunFailureTaxonomyV1,
    pub(crate) output_paths: Vec<String>,
    pub(crate) provider_receipts: Vec<String>,
    pub(crate) tool_receipts: Vec<String>,
    pub(crate) permit_receipts: Vec<String>,
}

pub(crate) async fn invoke_coding_agent_step(
    dispatcher: &ToolDispatcher,
    label: &str,
    tool_id: &str,
    input: serde_json::Value,
) -> serde_json::Value {
    match dispatcher.invoke(tool_id, input.clone()).await {
        Ok(outcome) => coding_agent_success_step(label, tool_id, input, outcome),
        Err(error) => {
            if let Some(tool_error) = error.downcast_ref::<ToolInvocationError>() {
                serde_json::json!({
                    "label": label,
                    "tool_id": tool_id,
                    "status": "blocked_or_failed",
                    "input": input,
                    "error": tool_error.to_string(),
                    "tool_invocation_receipt": tool_error.receipt(),
                    "approval_request": tool_error.approval_request(),
                    "schema_validation_receipt": tool_error.schema_validation_receipt(),
                })
            } else {
                serde_json::json!({
                    "label": label,
                    "tool_id": tool_id,
                    "status": "failed",
                    "input": input,
                    "error": error.to_string(),
                })
            }
        }
    }
}

pub(crate) fn coding_agent_success_step(
    label: &str,
    tool_id: &str,
    input: serde_json::Value,
    outcome: ToolInvocationOutcome,
) -> serde_json::Value {
    let status = if tool_id == "aidens:run-checks:1"
        && outcome
            .output
            .get("succeeded")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
    {
        "check_failed"
    } else if tool_id == "aidens:patch-apply:1"
        && outcome
            .output
            .get("applied")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
    {
        "checked"
    } else {
        "success"
    };
    serde_json::json!({
        "label": label,
        "tool_id": tool_id,
        "status": status,
        "input": input,
        "output": outcome.output,
        "tool_invocation_receipt": outcome.receipt,
        "permit_use_receipt": outcome.permit_use_receipt,
    })
}

pub(crate) fn append_coding_agent_step_record(
    log: &CanonicalEventLog,
    step: &serde_json::Value,
) -> Result<Option<aidens_receipts::CanonicalEventLogEntry>> {
    let Some(receipt) = step.get("tool_invocation_receipt") else {
        return Ok(None);
    };
    let receipt_id = receipt
        .get("receipt_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("coding-agent-tool-invocation");
    Ok(Some(log.append_orchestration_report(
        "tool-invocation-report-v1",
        receipt_id,
        receipt.clone(),
    )?))
}

pub(crate) fn coding_agent_read_path(sandbox_root: &Path) -> Result<String> {
    for candidate in ["README.md", "Cargo.toml", "src/lib.rs"] {
        if sandbox_root.join(candidate).is_file() {
            return Ok(candidate.into());
        }
    }
    for entry in std::fs::read_dir(sandbox_root)
        .with_context(|| format!("failed to list {}", sandbox_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                return Ok(name.into());
            }
        }
    }
    bail!("coding-agent sandbox has no readable file")
}

pub(crate) fn coding_agent_search_query(app_id: &str) -> String {
    app_id
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .find(|part| part.len() >= 3)
        .unwrap_or("AiDENs")
        .to_string()
}

pub(crate) fn coding_agent_patch_diff(sandbox_root: &Path, read_path: &str) -> Result<String> {
    let source = std::fs::read_to_string(sandbox_root.join(read_path))
        .with_context(|| format!("failed to read patch candidate {read_path}"))?;
    let removed = source
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("P24 fixture");
    let added = format!("{removed} [P24 local coding-agent proposed change]");
    Ok(format!(
        "--- a/{read_path}\n+++ b/{read_path}\n@@\n-{removed}\n+{added}\n"
    ))
}

pub(crate) fn repo_status_report(sandbox_root: &Path) -> Result<serde_json::Value> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(sandbox_root)
        .arg("status")
        .arg("--short")
        .output();
    match output {
        Ok(output) if output.status.success() => Ok(serde_json::json!({
            "tool_id": "aidens:repo-status:1",
            "status": "success",
            "sandbox_root": sandbox_root,
            "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
            "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
            "exit_code": output.status.code(),
            "read_only": true,
        })),
        Ok(output) => Ok(serde_json::json!({
            "tool_id": "aidens:repo-status:1",
            "status": "degraded",
            "sandbox_root": sandbox_root,
            "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
            "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
            "exit_code": output.status.code(),
            "read_only": true,
            "reason_codes": ["git-status-unavailable-or-not-repo"],
        })),
        Err(error) => Ok(serde_json::json!({
            "tool_id": "aidens:repo-status:1",
            "status": "degraded",
            "sandbox_root": sandbox_root,
            "read_only": true,
            "reason_codes": ["git-status-command-unavailable"],
            "error": error.to_string(),
        })),
    }
}

pub(crate) fn write_coding_agent_event_log(path: &Path, report: &serde_json::Value) -> Result<()> {
    let mut events = Vec::new();
    for step in report
        .get("steps")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        events.push(serde_json::json!({
            "event": step.get("label").and_then(serde_json::Value::as_str).unwrap_or("coding_step"),
            "tool_id": step.get("tool_id"),
            "status": step.get("status"),
            "receipt_id": step.pointer("/tool_invocation_receipt/receipt_id"),
            "reason_codes": step.pointer("/tool_invocation_receipt/reason_codes"),
        }));
    }
    events.push(serde_json::json!({
        "event": "repo_status_recorded",
        "status": report.pointer("/status/status"),
        "reason_codes": report.pointer("/status/reason_codes"),
    }));
    let mut lines = events
        .iter()
        .map(serde_json::to_string)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .join("\n");
    lines.push('\n');
    std::fs::write(path, lines).with_context(|| format!("failed to write {}", path.display()))
}

pub(crate) fn coding_agent_tool_receipt_ids(report: &serde_json::Value) -> Vec<String> {
    let mut ids = report
        .get("steps")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|step| {
            step.pointer("/tool_invocation_receipt/receipt_id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids
}

pub(crate) fn coding_agent_permit_receipt_ids(report: &serde_json::Value) -> Vec<String> {
    let mut ids = report
        .get("steps")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|step| {
            step.pointer("/permit_use_receipt/receipt_id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids
}

pub(crate) fn coding_agent_v11a_evidence(
    run_id: &str,
    config_path: &Path,
    sandbox_root: &Path,
    steps: &[serde_json::Value],
    receipt_chain: &[serde_json::Value],
    git_status: &serde_json::Value,
) -> Result<serde_json::Value> {
    let input_payload = serde_json::json!({
        "config_path": config_path,
        "sandbox_root": sandbox_root,
        "profile": "coding-agent",
        "run_id": run_id,
    });
    let output_payload = serde_json::json!({
        "steps": steps,
        "receipt_chain": receipt_chain,
        "status": git_status,
    });
    let input_artifact = aidens_contracts::ArtifactEnvelopeV1::from_json(
        "coding-agent-local-input",
        1,
        &input_payload,
        aidens_contracts::ArtifactAuthorityClassV1::AdmittedFacade,
        "p29-v11a-local",
        "aidens-cli",
    );
    let mut output_artifact = aidens_contracts::ArtifactEnvelopeV1::from_json(
        "coding-agent-local-output",
        1,
        &output_payload,
        aidens_contracts::ArtifactAuthorityClassV1::AiDENsExecutionAuthoritative,
        "p29-v11a-local",
        "aidens-cli",
    )
    .with_canonical_backpointer(CanonicalBackpointerV1::owner_type(
        "llm-tool-runtime",
        "ToolReceipt",
        "canonical-tool-receipt-owner",
    ));
    for state in [
        aidens_contracts::ArtifactLifecycleStateV1::Validated,
        aidens_contracts::ArtifactLifecycleStateV1::Admitted,
        aidens_contracts::ArtifactLifecycleStateV1::Projected,
        aidens_contracts::ArtifactLifecycleStateV1::Verified,
    ] {
        output_artifact
            .apply_transition(state, "aidens.runner.turn", "aidens-cli", None)
            .map_err(anyhow::Error::msg)?;
    }

    let mut schema_identities = BTreeMap::new();
    schema_identities.insert(
        "coding-agent-local-input".to_string(),
        "AiDENsCodingAgentLocalInputV1".to_string(),
    );
    schema_identities.insert(
        "coding-agent-local-output".to_string(),
        "AiDENsCodingAgentLocalRunV1".to_string(),
    );
    let input_manifest = aidens_contracts::ArtifactManifestV1::new(
        vec![aidens_contracts::ArtifactManifestEntryV1::from(
            &input_artifact,
        )],
        Vec::new(),
        "stack-ids-json-c14n-v1",
        schema_identities.clone(),
    );
    let output_manifest = aidens_contracts::ArtifactManifestV1::new(
        Vec::new(),
        vec![aidens_contracts::ArtifactManifestEntryV1::from(
            &output_artifact,
        )],
        "stack-ids-json-c14n-v1",
        schema_identities,
    );
    let execution_context = aidens_contracts::ExecutionContextEnvelopeV1::local_started(
        "aidens.runner.turn",
        aidens_contracts::generated_artifact_id_from_material("attempt-family", run_id),
        "local-tools-only",
        "aidens:safe-coding-tools",
    )
    .complete(aidens_contracts::ExecutionCompletionStateV1::Succeeded, 0);
    let operator_contract = aidens_contracts::p28_declared_material_operation_registry()
        .contract("aidens.runner.turn")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("missing operator contract: aidens.runner.turn"))?;
    let tool_receipt_refs = receipt_chain
        .iter()
        .filter_map(|step| {
            step.get("tool_receipt_id")
                .and_then(serde_json::Value::as_str)
                .filter(|id| !id.trim().is_empty())
                .map(ArtifactId::new)
        })
        .collect::<Vec<_>>();
    let missing_receipts = tool_receipt_refs.is_empty();
    let invocation_receipt = if missing_receipts {
        None
    } else {
        Some(
            aidens_contracts::OperatorInvocationReceiptV1::material_done(
                "aidens.runner.turn",
                &execution_context,
                input_manifest.clone(),
                output_manifest.clone(),
                tool_receipt_refs.clone(),
            )
            .map_err(anyhow::Error::msg)?,
        )
    };
    let mut proof_obligation = aidens_contracts::ProofObligationV1::new(
        "supported-local coding-agent run has complete manifests and durable receipts",
        "local-run-receipts",
    );
    if let Some(receipt) = &invocation_receipt {
        proof_obligation
            .satisfied_by
            .push(receipt.receipt_id.clone());
    }
    let proof_profile = aidens_contracts::LocalProofProfileV1::local_exact(vec![proof_obligation]);
    let proof_debt = aidens_contracts::ProofDebtLedgerV1::from_profile(
        output_artifact.artifact_ref.clone(),
        &proof_profile,
    );
    let promotion_eligibility = aidens_contracts::PromotionEligibilityReportV1::new(
        output_artifact.artifact_ref.clone(),
        &proof_profile,
        &proof_debt,
    );
    let mut degradation_records = Vec::new();
    if missing_receipts {
        degradation_records.push(aidens_contracts::LocalDegradationRecordV1::new(
            output_artifact.artifact_ref.clone(),
            "missing-tool-receipts",
            "material completion cannot be proven",
        ));
    }
    if proof_debt.blocks_promotion() {
        degradation_records.push(aidens_contracts::LocalDegradationRecordV1::new(
            output_artifact.artifact_ref.clone(),
            "proof-debt-blocks-promotion",
            "release-candidate completion blocked until proof obligations are satisfied",
        ));
    }
    let view_disclosure = aidens_contracts::ViewDisclosureV1 {
        disclosure_id: aidens_contracts::generated_artifact_id_from_material(
            "view-disclosure",
            &format!("{run_id}|coding-agent-local-report|supported-local"),
        ),
        view_family: "coding-agent-local-report".into(),
        widening: false,
        support_label: "supported-local".into(),
        exactness: if degradation_records.is_empty() {
            aidens_contracts::SemanticExactnessV1::Exact
        } else {
            aidens_contracts::SemanticExactnessV1::Degraded
        },
        source_report_id: Some(output_artifact.artifact_ref.clone()),
        reason_codes: vec!["supported-local-view-disclosed".into()],
        recorded_at: Utc::now(),
    };
    let mut semantic_state = aidens_contracts::SemanticStateV1::exact_supported(
        output_artifact.artifact_ref.clone(),
        proof_profile.profile_id.as_str().clone(),
    )
    .with_view_disclosure(&view_disclosure);
    for degradation in &degradation_records {
        semantic_state = semantic_state.with_degradation(degradation);
    }
    let completion_gate = serde_json::json!({
        "status": if invocation_receipt.is_some()
            && !proof_debt.blocks_promotion()
            && degradation_records.is_empty()
        {
            "complete"
        } else {
            "blocked_or_degraded"
        },
        "material_done": invocation_receipt.is_some(),
        "missing_receipts": missing_receipts,
        "proof_debt_blocks": proof_debt.blocks_promotion(),
        "semantic_exact": semantic_state.can_answer_as_exact(),
        "reason_codes": if invocation_receipt.is_some()
            && !proof_debt.blocks_promotion()
            && degradation_records.is_empty()
        {
            vec!["v11a-supported-local-evidence-complete"]
        } else {
            vec!["v11a-completion-blocked-by-missing-evidence"]
        },
    });

    Ok(serde_json::json!({
        "status": "supported-local-release-candidate-evidence",
        "claim_scope": "declared supported-local coding-agent path only",
        "artifact_envelope": output_artifact,
        "input_manifest": input_manifest,
        "output_manifest": output_manifest,
        "execution_context": execution_context,
        "operator_contract": operator_contract,
        "operator_invocation_receipt": invocation_receipt,
        "proof_profile": proof_profile,
        "proof_debt_ledger": proof_debt,
        "promotion_eligibility": promotion_eligibility,
        "semantic_state": semantic_state,
        "view_disclosure": view_disclosure,
        "degradation_records": degradation_records,
        "completion_gate": completion_gate,
    }))
}

pub(crate) fn coding_agent_receipt_chain(steps: &[serde_json::Value]) -> Vec<serde_json::Value> {
    steps
        .iter()
        .map(|step| {
            serde_json::json!({
                "label": step.get("label"),
                "tool_id": step.get("tool_id"),
                "status": step.get("status"),
                "tool_receipt_id": step.pointer("/tool_invocation_receipt/receipt_id"),
                "permit_use_receipt_id": step.pointer("/permit_use_receipt/receipt_id"),
                "reason_codes": step.pointer("/tool_invocation_receipt/reason_codes"),
                "changed_files": step.pointer("/output/changed_files")
                    .or_else(|| step.pointer("/tool_invocation_receipt/output/changed_files")),
                "check_succeeded": step.pointer("/output/succeeded"),
            })
        })
        .collect()
}

pub(crate) fn coding_agent_loop_summary(steps: &[serde_json::Value]) -> serde_json::Value {
    let mut blocked_steps = Vec::new();
    let mut failed_checks = Vec::new();
    let mut changed_files = BTreeSet::new();
    let mut permit_use_receipts = BTreeSet::new();
    let mut applied_patch = false;
    for step in steps {
        let label = step
            .get("label")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("step");
        let status = step
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        if status == "blocked_or_failed" {
            blocked_steps.push(label.to_string());
        }
        if status == "check_failed" {
            failed_checks.push(label.to_string());
        }
        if step
            .pointer("/output/applied")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        {
            applied_patch = true;
        }
        for path in step
            .pointer("/output/changed_files")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
        {
            changed_files.insert(path.to_string());
        }
        if let Some(id) = step
            .pointer("/permit_use_receipt/receipt_id")
            .and_then(serde_json::Value::as_str)
        {
            permit_use_receipts.insert(id.to_string());
        }
    }
    serde_json::json!({
        "total_steps": steps.len(),
        "blocked_steps": blocked_steps,
        "failed_checks": failed_checks,
        "applied_patch": applied_patch,
        "changed_files": changed_files.into_iter().collect::<Vec<_>>(),
        "permit_use_receipts": permit_use_receipts.into_iter().collect::<Vec<_>>(),
        "semantic_status": coding_agent_semantic_status(steps),
    })
}

pub(crate) fn coding_agent_semantic_status(steps: &[serde_json::Value]) -> &'static str {
    if steps
        .iter()
        .any(|step| step.get("status").and_then(serde_json::Value::as_str) == Some("check_failed"))
    {
        "degraded_exact_check"
    } else {
        "exact_check"
    }
}

pub(crate) fn coding_agent_failure_taxonomy(
    report: &serde_json::Value,
) -> AiDENsRunFailureTaxonomyV1 {
    let steps = report
        .get("steps")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let failed_check = steps
        .iter()
        .any(|step| step.get("status").and_then(serde_json::Value::as_str) == Some("check_failed"));
    let side_effect_blocked = steps.iter().any(|step| {
        matches!(
            step.get("label").and_then(serde_json::Value::as_str),
            Some("patch_apply_permit_gate" | "run_checks_permit_gate")
        ) && step.get("status").and_then(serde_json::Value::as_str) == Some("blocked_or_failed")
    });
    let applied_patch = steps.iter().any(|step| {
        step.pointer("/output/applied")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
    });
    AiDENsRunFailureTaxonomyV1 {
        class: if failed_check {
            AiDENsRunFailureClassV1::ToolFailed
        } else if side_effect_blocked || applied_patch {
            AiDENsRunFailureClassV1::None
        } else {
            AiDENsRunFailureClassV1::OperatorAbstained
        },
        reason_codes: if failed_check {
            vec!["check-command-failed".into()]
        } else if side_effect_blocked {
            vec!["side-effect-tools-require-permit-blocked-as-expected".into()]
        } else if applied_patch {
            vec!["patch-and-check-loop-succeeded".into()]
        } else {
            vec!["no-side-effect-tool-executed".into()]
        },
        degraded: failed_check,
        blocked: false,
    }
}

pub(crate) fn build_local_run_bundle_v2(
    input: LocalRunBundleInput<'_>,
) -> Result<AiDENsRunBundleV2> {
    let trace_ctx = input
        .trace_ctx
        .clone()
        .unwrap_or_else(aidens_contracts::StackTraceCtx::generate);
    let attempt_id = input
        .attempt_id
        .clone()
        .unwrap_or_else(aidens_contracts::StackAttemptId::generate);
    let trial_id = input
        .trial_id
        .clone()
        .unwrap_or_else(aidens_contracts::StackTrialId::generate);
    let mut execution_context =
        aidens_contracts::canonical_stack::ForgeExecutionContextV1::new(trace_ctx);
    execution_context.attempt_id = Some(attempt_id);
    execution_context.trial_id = Some(trial_id);
    execution_context.replay_link = Some(input.replay_command.clone());
    execution_context.workload_class = Some(input.workload_class.into());
    execution_context.deadline = Some(format!("{}ms", input.budget_max_turn_millis()));
    execution_context.cost_budget_units = Some(input.budget_max_tool_calls().into());
    execution_context.degradation_markers = input.degradation.clone();
    execution_context.provider_route = input.provider_route.map(str::to_string);
    execution_context.dispatch_outcome = if input.failure.blocked {
        aidens_contracts::canonical_stack::ForgeDispatchOutcomeV1::Failed
    } else if input.failure.degraded || !input.degradation.is_empty() {
        aidens_contracts::canonical_stack::ForgeDispatchOutcomeV1::Degraded
    } else {
        aidens_contracts::canonical_stack::ForgeDispatchOutcomeV1::Succeeded
    };
    execution_context.environment_fingerprint = Some(
        StackContentDigest::compute_str(&format!(
            "{}:{}",
            input.profile,
            input.output_dir.display()
        ))
        .hex()
        .to_string(),
    );

    let event_log = event_log_digest(
        input.event_log_path,
        input.canonical_record_count,
        input.event_count,
    )?;
    let replay = AiDENsRunReplayNormalizationV1 {
        replay_command: input.replay_command,
        fixture_path: input.fixture_path,
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
    let budget = AiDENsRunBudgetDeadlineV1 {
        max_steps: 16,
        max_tool_calls: 16,
        max_retries: 2,
        max_turn_millis: 120_000,
        elapsed_ms: input.elapsed_ms,
        deadline: Some("120000ms".into()),
        cost_budget_units: Some(16),
        degradation_markers: input.degradation,
    };
    let mut bundle = AiDENsRunBundleV2::new(
        input.run_id,
        input.profile,
        execution_context,
        event_log,
        budget,
        input.support,
        replay,
        input.failure,
    );
    bundle.provider_receipts = input.provider_receipts;
    bundle.tool_receipts = input.tool_receipts;
    bundle.permit_receipts = input.permit_receipts;
    bundle.outputs = input.output_paths;
    Ok(bundle)
}

impl<'a> LocalRunBundleInput<'a> {
    fn budget_max_tool_calls(&self) -> u32 {
        16
    }

    fn budget_max_turn_millis(&self) -> u64 {
        120_000
    }
}

pub(crate) fn event_log_digest(
    path: &Path,
    canonical_record_count: usize,
    event_count: usize,
) -> Result<AiDENsRunEventLogDigestV1> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read event log {}", path.display()))?;
    let normalized = replay_normalized_event_log(&text)?;
    let event_count = if event_count == 0 {
        text.lines().filter(|line| !line.trim().is_empty()).count()
    } else {
        event_count
    };
    Ok(AiDENsRunEventLogDigestV1 {
        event_log_path: path.display().to_string(),
        digest: StackContentDigest::compute_str(&text),
        replay_normalized_digest: StackContentDigest::compute_str(&normalized),
        canonical_record_count,
        event_count,
        reason_codes: vec!["event-log-digested-by-stack-ids".into()],
    })
}

pub(crate) fn replay_normalized_event_log(text: &str) -> Result<String> {
    let mut values = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let mut value: serde_json::Value = serde_json::from_str(line)
            .with_context(|| "event log line must be valid JSON for replay normalization")?;
        normalize_replay_value(&mut value);
        values.push(value);
    }
    serde_json::to_string(&values).with_context(|| "failed to encode normalized event log")
}

pub(crate) fn normalize_replay_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for key in [
                "receipt_id",
                "run_receipt_id",
                "turn_receipt_id",
                "report_id",
                "recorded_at",
                "started_at",
                "completed_at",
                "content_digest",
            ] {
                if map.contains_key(key) {
                    map.insert(key.into(), serde_json::Value::String("<normalized>".into()));
                }
            }
            for value in map.values_mut() {
                normalize_replay_value(value);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                normalize_replay_value(value);
            }
        }
        _ => {}
    }
}

pub(crate) fn load_test_agent_file(path: &Path) -> Result<TestAgentFileV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read test-agent config {}", path.display()))?;
    let config = toml::from_str::<TestAgentFileV1>(&contents)
        .with_context(|| format!("failed to parse test-agent config {}", path.display()))?;
    if config.agent.name.trim().is_empty() {
        bail!("test-agent [agent].name must not be empty");
    }
    if config.provider.fixture.trim().is_empty() {
        bail!("test-agent [provider].fixture must not be empty");
    }
    Ok(config)
}

pub(crate) fn load_test_agent_fixture(path: &Path) -> Result<Value> {
    serde_json::from_str(
        &std::fs::read_to_string(path)
            .with_context(|| format!("failed to read test-agent fixture {}", path.display()))?,
    )
    .with_context(|| format!("failed to parse test-agent fixture {}", path.display()))
}

pub(crate) fn test_agent_effective_config(
    config_path: &Path,
    test_agent: &TestAgentFileV1,
    sandbox_root: &Path,
    receipt_root: &Path,
    seed_fixture_files: bool,
) -> Result<(AiDENsConfigV1, PathBuf, TestAgentFixturePlan)> {
    if test_agent.provider.kind.trim() != "mock" {
        bail!("test-agent plan sources currently support only mock fixture providers");
    }
    let reference_root = test_agent_reference_root(config_path)?;
    let fixture_path = resolve_against_root(&test_agent.provider.fixture, &reference_root);
    let fixture = load_test_agent_fixture(&fixture_path)?;
    let fixture_plan = prepare_test_agent_fixture(&fixture, sandbox_root, seed_fixture_files)?;
    let enabled_bundles = test_agent_enabled_bundles(test_agent, &fixture_plan)?;
    let profile_id = test_agent_profile_id(test_agent, &enabled_bundles);
    let mut effective_config = AiDENsConfigV1::safe_default(&test_agent.agent.name);
    effective_config.profile_id = Some(profile_id);
    effective_config.provider = ProviderConfigV1 {
        kind: "mock".into(),
        model: Some("test-agent-fixture".into()),
        api_key: None,
        base_url: None,
        mock_response: Some(fixture_plan.mock_response.clone()),
    };
    effective_config.tools.sandbox_root = Some(sandbox_root.display().to_string());
    effective_config.tools.enabled_bundles = enabled_bundles;
    effective_config.receipts.store_root = Some(receipt_root.display().to_string());
    effective_config.memory_mode = MemoryModeV1::Disabled;
    effective_config.receipt_level = if test_agent.agency.require_receipts {
        ReportLevelV1::Full
    } else {
        ReportLevelV1::Standard
    };
    Ok((effective_config, fixture_path, fixture_plan))
}

pub(crate) fn prepare_test_agent_fixture(
    fixture: &Value,
    sandbox_root: &Path,
    seed_fixture_files: bool,
) -> Result<TestAgentFixturePlan> {
    let turns = fixture
        .get("turns")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("test-agent fixture must contain turns array"))?;
    let user_prompt = fixture_turn_content(turns, "user")?.to_string();
    let final_text = fixture_turn_content(turns, "assistant_final")?.to_string();
    let tool_turn = turns
        .iter()
        .find(|turn| turn.get("role").and_then(Value::as_str) == Some("assistant_tool_call"));
    let Some(tool_turn) = tool_turn else {
        return Ok(TestAgentFixturePlan {
            user_prompt,
            mock_response: final_text,
            requested_tool_bundles: Vec::new(),
            seeded_files: Vec::new(),
        });
    };
    let tool_name = required_json_str(tool_turn, "tool", "assistant_tool_call.tool")?;
    let tool_id = canonical_test_agent_tool_id(tool_name)?;
    let tool_input = tool_turn
        .get("arguments")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("assistant_tool_call.arguments missing"))?;
    let mut seeded_files = Vec::new();
    if tool_name == "repo.read" && seed_fixture_files {
        let relative =
            required_json_str(&tool_input, "path", "assistant_tool_call.arguments.path")?;
        let target = sandbox_root.join(relative);
        if !target.exists() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            std::fs::write(&target, TEST_AGENT_SEEDED_README)
                .with_context(|| format!("failed to seed {}", target.display()))?;
            seeded_files.push(target);
        }
    }
    let mock_tool_call = serde_json::json!({
        "tool_call": {
            "tool_id": tool_id,
            "input": tool_input,
        }
    });
    let mock_response = format!(
        "{}{}{} Tool evidence: {{{{last_tool_content}}}}",
        serde_json::to_string(&mock_tool_call)?,
        TEST_AGENT_MOCK_RESPONSE_DELIMITER,
        final_text
    );
    Ok(TestAgentFixturePlan {
        user_prompt,
        mock_response,
        requested_tool_bundles: vec![test_agent_bundle_for_tool(tool_name)?.into()],
        seeded_files,
    })
}

pub(crate) fn fixture_turn_content<'a>(turns: &'a [Value], role: &str) -> Result<&'a str> {
    turns
        .iter()
        .find(|turn| turn.get("role").and_then(Value::as_str) == Some(role))
        .and_then(|turn| turn.get("content").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("test-agent fixture missing {role} content"))
}

pub(crate) fn required_json_str<'a>(value: &'a Value, key: &str, label: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{label} missing or not a string"))
}

pub(crate) fn canonical_test_agent_tool_id(tool_name: &str) -> Result<&'static str> {
    match tool_name {
        "repo.read" => Ok("aidens:repo-read:1"),
        "repo.list" => Ok("aidens:repo-list:1"),
        "repo.search" => Ok("aidens:repo-search:1"),
        "file.stat" => Ok("aidens:file-stat:1"),
        _ => bail!("unsupported test-agent tool: {tool_name}"),
    }
}

pub(crate) fn test_agent_bundle_for_tool(tool_name: &str) -> Result<&'static str> {
    match tool_name {
        "repo.read" => Ok("repo-read"),
        "repo.list" => Ok("repo-list"),
        "repo.search" => Ok("repo-search"),
        "file.stat" => Ok("file-stat"),
        _ => bail!("unsupported test-agent tool: {tool_name}"),
    }
}

pub(crate) fn test_agent_enabled_bundles(
    test_agent: &TestAgentFileV1,
    fixture_plan: &TestAgentFixturePlan,
) -> Result<Vec<String>> {
    let mut bundles = BTreeSet::new();
    for bundle in &test_agent.tools.enabled_bundles {
        bundles.insert(bundle.clone());
    }
    for exposed in &test_agent.tools.expose {
        bundles.insert(test_agent_bundle_for_tool(exposed)?.into());
    }
    for bundle in &fixture_plan.requested_tool_bundles {
        bundles.insert(bundle.clone());
    }
    Ok(bundles.into_iter().collect())
}

pub(crate) fn test_agent_profile_id(
    test_agent: &TestAgentFileV1,
    enabled_bundles: &[String],
) -> String {
    test_agent
        .agent
        .profile
        .as_deref()
        .and_then(AiDENsProfile::from_id)
        .map(|profile| profile.id().to_string())
        .unwrap_or_else(|| {
            if enabled_bundles.is_empty() {
                "chat-only".into()
            } else {
                "coding-agent".into()
            }
        })
}

pub(crate) fn test_agent_run_id(
    test_agent: &TestAgentFileV1,
    config_path: &Path,
) -> Result<String> {
    let reference_root = test_agent_reference_root(config_path)?;
    let fixture_path = resolve_against_root(&test_agent.provider.fixture, &reference_root);
    let fixture_stem = fixture_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("fixture");
    Ok(format!(
        "{}-{}",
        receipt_store_segment(&test_agent.agent.name),
        receipt_store_segment(fixture_stem)
    ))
}

pub(crate) fn write_json_file(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

pub(crate) fn write_test_agent_run_bundle(
    path: &Path,
    input: &TestAgentRunBundleInput<'_>,
) -> Result<()> {
    let started_at = input.output.receipt.context.started_at;
    let completed_at = input.output.receipt.completed_at.unwrap_or(started_at);
    let elapsed_ms = (completed_at - started_at).num_milliseconds().max(0);
    let degradation = test_agent_degradation_reasons(input.output);
    let failure_class = if input.output.turn_receipt.blocked {
        AiDENsRunFailureClassV1::ToolBlocked
    } else if input.output.turn_receipt.degraded {
        AiDENsRunFailureClassV1::VerificationUnavailable
    } else {
        AiDENsRunFailureClassV1::None
    };
    let event_log_path = input.output_dir.join("event-log.ndjson");
    let provider_route_label = input
        .output
        .receipt
        .provider_route
        .as_ref()
        .map(|route| route.route_label.as_str());
    let bundle = build_local_run_bundle_v2(LocalRunBundleInput {
        run_id: input.run_id,
        profile: input.profile,
        workload_class: "fixture-test-agent",
        provider_route: provider_route_label,
        trace_ctx: Some(input.output.receipt.context.stack_trace_ctx()),
        attempt_id: Some(input.output.receipt.context.stack_attempt_id()),
        trial_id: None,
        replay_command: format!(
            "cargo run -p aidens-cli -- run-test-agent {} --out {}",
            input.config_path.display(),
            input.output_dir.display()
        ),
        fixture_path: Some(input.fixture_path.display().to_string()),
        output_dir: input.output_dir,
        event_log_path: &event_log_path,
        canonical_record_count: input.canonical_records.len(),
        event_count: 0,
        elapsed_ms,
        degradation,
        support: AiDENsRunSupportTierEvidenceV1 {
            support_tier: "fixture-supported".into(),
            supported: vec![
                "fixture-backed-mock-provider".into(),
                "repo-read-tool-route".into(),
                "durable-canonical-event-log".into(),
                "run-bundle-v2-inspect".into(),
            ],
            partial: vec!["parser-fallback-tool-loop".into()],
            deferred: vec![
                "cloud-provider-execution".into(),
                "native-provider-tool-loop".into(),
                "autonomous-daemon-readiness".into(),
            ],
            reason_codes: vec!["p24-v2-run-bundle".into()],
        },
        failure: AiDENsRunFailureTaxonomyV1 {
            class: failure_class,
            reason_codes: input.output.turn_receipt.reason_codes.clone(),
            degraded: input.output.turn_receipt.degraded,
            blocked: input.output.turn_receipt.blocked,
        },
        output_paths: vec![
            input.output_dir.join("final.txt").display().to_string(),
            input
                .output_dir
                .join("run-report.json")
                .display()
                .to_string(),
            input
                .output_dir
                .join("turn-report.json")
                .display()
                .to_string(),
            input
                .output_dir
                .join("tool-exposure.json")
                .display()
                .to_string(),
            input
                .output_dir
                .join("agency-policy-reports.json")
                .display()
                .to_string(),
            event_log_path.display().to_string(),
            input
                .receipt_root
                .join("canonical-receipts.ndjson")
                .display()
                .to_string(),
        ],
        provider_receipts: vec![input.output.receipt.receipt_id.as_str().clone()],
        tool_receipts: input
            .output
            .receipt
            .tool_invocation_receipts
            .iter()
            .map(|receipt| receipt.receipt_id.as_str().clone())
            .collect(),
        permit_receipts: input
            .output
            .receipt
            .permit_use_receipts
            .iter()
            .map(|receipt| receipt.receipt_id.as_str().clone())
            .collect(),
    })?;
    write_json_file(path, &bundle)
}

pub(crate) fn test_agent_degradation_reasons(
    output: &aidens_runner::AiDENsRunOutput,
) -> Vec<String> {
    let mut reasons = output.receipt.warnings.clone();
    if output.turn_receipt.degraded {
        reasons.push("turn-degraded".into());
    }
    if let Some(route) = output.receipt.provider_route.as_ref() {
        if route.degraded {
            reasons.push(format!("provider-route-degraded:{}", route.route_label));
        }
        reasons.extend(route.reason_codes.clone());
    }
    reasons.extend(
        output
            .tool_exposure
            .decisions
            .iter()
            .flat_map(|decision| decision.reason_codes.clone()),
    );
    reasons.sort();
    reasons.dedup();
    reasons
}

pub(crate) fn write_test_agent_event_log(
    path: &Path,
    output: &aidens_runner::AiDENsRunOutput,
    canonical_records: &[aidens_receipts::CanonicalEventLogEntry],
) -> Result<()> {
    let provider_route = output
        .receipt
        .provider_route
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("run report missing provider route"))?;
    let mut events = vec![
        serde_json::json!({
            "event": "provider_route_selected",
            "provider": &provider_route.provider_kind,
            "route": &provider_route.route_label,
            "native_tool_loop": provider_route.native_tool_loop,
            "degraded": provider_route.degraded,
            "reason_codes": &provider_route.reason_codes,
        }),
        serde_json::json!({
            "event": "tool_exposure_plan_created",
            "exposure_id": &output.tool_exposure.exposure_id,
            "exposed_tool_ids": &output.tool_exposure.exposed_tool_ids,
            "blocked_tool_ids": &output.tool_exposure.blocked_tool_ids,
        }),
    ];
    for decision in &output.tool_exposure.decisions {
        events.push(serde_json::json!({
            "event": "permit_checked",
            "tool": &decision.capability_id,
            "outcome": &decision.outcome,
            "permit_required": decision.permit_required,
            "executable_this_turn": decision.executable_this_turn,
            "reason_codes": &decision.reason_codes,
        }));
    }
    for invocation in &output.receipt.tool_invocation_receipts {
        events.push(serde_json::json!({
            "event": "tool_invocation_recorded",
            "tool": &invocation.tool_id,
            "succeeded": invocation.succeeded,
            "receipt_id": &invocation.receipt_id,
            "reason_codes": &invocation.reason_codes,
        }));
    }
    for report in &output.agency_policy_reports {
        events.push(serde_json::json!({
            "event": "agency_policy_evaluated",
            "surface": &report.surface,
            "report_id": &report.report_id,
            "outcome": &report.outcome,
            "receipt_schema_names": report.receipt_schema_names(),
        }));
    }
    events.push(serde_json::json!({
        "event": "final_response_recorded",
        "run_receipt_id": &output.receipt.receipt_id,
        "turn_receipt_id": &output.turn_receipt.receipt_id,
        "final_state": &output.turn_receipt.final_state,
        "canonical_record_count": canonical_records.len(),
    }));
    let mut lines = events
        .iter()
        .map(serde_json::to_string)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .join("\n");
    lines.push('\n');
    std::fs::write(path, lines).with_context(|| format!("failed to write {}", path.display()))
}

pub(crate) fn write_test_agent_summary(
    path: &Path,
    input: &TestAgentSummaryInput<'_>,
) -> Result<()> {
    let seeded = if input.seeded_files.is_empty() {
        "- none\n".to_string()
    } else {
        input
            .seeded_files
            .iter()
            .map(|path| format!("- {}\n", path.display()))
            .collect::<String>()
    };
    let summary = format!(
        "# AiDENs Test Agent Run\n\n\
Status: PASS\n\n\
Config: `{}`\n\
Fixture: `{}`\n\
Bundle run ID: `{}`\n\
Output directory: `{}`\n\
Sandbox root: `{}`\n\
Receipt root: `{}`\n\
Run receipt: `{}`\n\
Turn final state: `{}`\n\
Agency policy reports: `{}`\n\
Canonical event log records: `{}`\n\n\
Seeded files:\n{}\
\nFinal output:\n\n```text\n{}\n```\n",
        input.config_path.display(),
        input.fixture_path.display(),
        input.run_id,
        input.output_dir.display(),
        input.sandbox_root.display(),
        input.receipt_root.display(),
        input.run_receipt_id,
        input.turn_final_state,
        input.agency_report_count,
        input.canonical_record_count,
        seeded,
        input.final_text
    );
    std::fs::write(path, summary).with_context(|| format!("failed to write {}", path.display()))
}

pub(crate) fn resolve_cli_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()?.join(path))
}

pub(crate) fn resolve_output_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    resolve_cli_path(path)
}

pub(crate) fn resolve_against_root(path: impl AsRef<Path>, root: &Path) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

pub(crate) fn resolve_bundle_path(bundle_dir: &Path, path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        bundle_dir.join(path)
    }
}

pub(crate) fn test_agent_reference_root(config_path: &Path) -> Result<PathBuf> {
    let start = config_path.parent().unwrap_or_else(|| Path::new("."));
    for ancestor in start.ancestors() {
        if ancestor.join("Cargo.toml").exists() && ancestor.join("fixtures").exists() {
            return Ok(ancestor.to_path_buf());
        }
    }
    std::env::current_dir().context("failed to resolve test-agent reference root")
}

pub(crate) fn parse_profile(profile: &str) -> Result<AiDENsProfile> {
    AiDENsProfile::from_id(profile)
        .ok_or_else(|| anyhow::anyhow!("unknown AiDENs profile: {profile}"))
}

pub(crate) fn parse_risk_class(risk: &str) -> Result<CanonicalToolSideEffectClass> {
    match risk.trim().to_ascii_lowercase().as_str() {
        "read-only" | "read_only" | "readonly" => Ok(CanonicalToolSideEffectClass::ReadOnly),
        "memory-write" | "memory" => Ok(CanonicalToolSideEffectClass::Write),
        "file-write" | "file" | "write" => Ok(CanonicalToolSideEffectClass::Write),
        "shell" => Ok(CanonicalToolSideEffectClass::Admin),
        "network" => Ok(CanonicalToolSideEffectClass::Analysis),
        "schedule" => Ok(CanonicalToolSideEffectClass::Admin),
        "queue-or-daemon" | "queue" | "daemon" => Ok(CanonicalToolSideEffectClass::Admin),
        "external-federation" | "federation" => Ok(CanonicalToolSideEffectClass::Admin),
        _ => bail!("unknown risk class: {risk}"),
    }
}
