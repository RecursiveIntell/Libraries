//! Execution helpers for the local Plan-Act-Verify loop.
//!
//! These helpers coordinate local run evidence and memory grounding without
//! owning canonical memory or verification truth.

use super::*;

pub(super) fn build_p26_memory_fixture_envelope(
    agent_id: &str,
) -> anyhow::Result<canonical_stack::ExportEnvelopeV3> {
    use aidens_memory_kit::canonical_stack::{
        ClaimId, ClaimVersionId, EntityId, EnvelopeId, ExportClaim, ExportEnvelopeV2, ExportRecord,
        ScopeKey, TraceCtx, EXPORT_ENVELOPE_V2_SCHEMA,
    };

    let namespace = format!("p26-memory:{}", agent_id.replace(':', "_"));
    let scope_key = ScopeKey::namespace_only(namespace);
    let records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new(format!("claim:{agent_id}"))),
        claim_version_id: Some(ClaimVersionId::new(format!("claim-version:{agent_id}:v1"))),
        subject_entity_id: EntityId::new(format!("entity:{agent_id}")),
        predicate: "supports-memory-grounding".into(),
        object_anchor: serde_json::json!(
            "canonical memory fixture imported for local agent memory grounding"
        ),
        valid_from: Some("2026-05-04T00:00:00Z".into()),
        valid_to: None,
        confidence: 1.0,
        content: "agent memory grounding fixture content for P26 local loop".into(),
        projection_family: "p26_local_agent".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: Some(serde_json::json!({
            "kernel_semantics_v3": {
                "claim_family_id": format!("claim-family:{agent_id}"),
                "assertion_group_id": format!("assertion-group:{agent_id}")
            }
        })),
    })];
    let envelope = ExportEnvelopeV2 {
        envelope_id: EnvelopeId::new(format!("envelope:{agent_id}")),
        schema_version: EXPORT_ENVELOPE_V2_SCHEMA.into(),
        content_digest: ExportEnvelopeV2::compute_digest(
            "forge", &scope_key, &records, None, None,
        )?,
        source_authority: "forge".into(),
        scope_key,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-05-04T00:00:00Z".into(),
        export_meta: None,
        evidence_bundle: None,
        records,
    };
    Ok(envelope.enrich_to_v3()?)
}

pub(super) async fn run_canonical_memory_grounding(
    spec: &AgentSpecV1,
    query: &str,
) -> anyhow::Result<Vec<String>> {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| anyhow!("memory-grounding-timestamp:{error}"))?
        .as_nanos();
    let namespace = format!("p26-memory:{}", spec.agent_id.replace(':', "_"));
    let memory_root = std::env::temp_dir().join(format!("aidens-p26-memory-{now_nanos}"));
    let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(&memory_root),
        runtime_config_for_namespace(&namespace),
    )
    .map_err(|error| anyhow!("memory-grounding-open:{error}"))?;
    let envelope = build_p26_memory_fixture_envelope(&spec.agent_id)?;
    let import_result = adapter
        .import_forge_export(&envelope)
        .await
        .map_err(|error| anyhow!("memory-grounding-import:{error}"))?;
    let (query_results, query_trace) = adapter
        .query(query, None)
        .await
        .map_err(|error| anyhow!("memory-grounding-query:{error}"))?;

    let normalized_query = query.to_lowercase();

    let mut degradation = Vec::new();
    let mut widening = Vec::new();
    if query_trace.has_scope_enforcement_warning() {
        degradation.push("scope-enforced-partial".into());
    }
    if query_trace.has_temporal_downgrade() {
        degradation.push("temporal-downgrade".into());
    }
    if query_trace.has_import_staleness_warning() {
        degradation.push("import-stale".into());
    }
    if query_trace.is_degraded() {
        degradation.push("query-degraded".into());
    }
    if !query_trace.warnings.is_empty() {
        degradation.push(format!("warnings:{}", query_trace.warnings.len()));
    }
    if format!("{:?}", query_trace.widenings)
        .to_lowercase()
        .contains("widen")
    {
        widening.push("query-widened".into());
    }

    let evidence = MemoryGroundingEvidenceV1::canonical_seam(
        spec.memory_policy.mode.to_string(),
        query,
        query_results.len(),
        envelope.records.len(),
        format!("{:?}", query_trace.trace_ctx),
        degradation.clone(),
        widening.clone(),
    );

    let mut receipts = vec![
        evidence
            .to_receipt_line()
            .map_err(|error| anyhow!("memory-grounding-receipt-json:{error}"))?,
        format!(
            "memory-grounding:canonical-seam:mode={}",
            spec.memory_policy.mode
        ),
        format!(
            "memory-grounding:canonical-seam:batch-records:{}",
            envelope.records.len()
        ),
        format!("memory-grounding:canonical-seam:import:{:?}", import_result),
        format!(
            "memory-grounding:canonical-seam:results:{}",
            query_results.len()
        ),
        format!(
            "memory-grounding:canonical-seam:query-trace:{:?}",
            query_trace.trace_ctx
        ),
    ];

    if query_results.is_empty() {
        return Err(anyhow!("memory-grounding-no-results"));
    }

    if normalized_query.contains("canonical seam fixture") {
        receipts.push("memory-grounding:view-disclosure:canonical-requested".into());
    }
    if degradation
        .iter()
        .any(|label| label == "scope-enforced-partial")
    {
        receipts.push("memory-grounding:view-disclosure:scope-enforced-partial".into());
    }
    if degradation
        .iter()
        .any(|label| label == "temporal-downgrade")
    {
        receipts.push("memory-grounding:view-disclosure:temporal-downgrade".into());
    }
    if degradation.iter().any(|label| label == "import-stale") {
        receipts.push("memory-grounding:view-disclosure:import-stale".into());
    }
    if degradation.iter().any(|label| label == "query-degraded") {
        receipts.push("memory-grounding:view-disclosure:query-degraded".into());
    }
    if let Some(warning_count) = degradation
        .iter()
        .find_map(|label| label.strip_prefix("warnings:"))
    {
        receipts.push(format!(
            "memory-grounding:view-disclosure:warnings:{warning_count}"
        ));
    }

    if !widening.is_empty() {
        receipts.push("memory-grounding:view-disclosure:widened".into());
    }

    if format!("{:?}", query_trace)
        .to_lowercase()
        .contains("degrad")
    {
        receipts.push("memory-grounding:view-disclosure:degraded".into());
    }

    Ok(receipts)
}

#[derive(Debug, Clone)]
pub(super) struct AgentToolResolution {
    pub(super) canonical: Vec<String>,
    pub(super) unsupported: Vec<String>,
}

pub(super) fn map_agent_tools_to_ids(tool_aliases: &[String]) -> AgentToolResolution {
    let mut canonical = Vec::new();
    let mut unsupported = Vec::new();
    for alias in tool_aliases {
        match alias.as_str() {
            "repo.read" => canonical.push("aidens:repo-read:1".into()),
            "repo.list" => canonical.push("aidens:repo-list:1".into()),
            "repo.search" => canonical.push("aidens:repo-search:1".into()),
            "patch.propose" => canonical.push("aidens:patch-propose:1".into()),
            "patch.apply" => canonical.push("aidens:patch-apply:1".into()),
            "checks.run" => canonical.push("aidens:run-checks:1".into()),
            "run.inspect" => canonical.push("aidens:run-checks:1".into()),
            "run.replay" => unsupported.push(alias.clone()),
            _ => unsupported.push(alias.clone()),
        }
    }
    canonical.sort();
    canonical.dedup();
    unsupported.sort();
    unsupported.dedup();
    AgentToolResolution {
        canonical,
        unsupported,
    }
}

pub(super) fn verification_checks_for_loop(
    spec: &AgentSpecV1,
    output: &AiDENsRunOutput,
    allowed_tools: &[String],
) -> Vec<VerificationReceiptV1> {
    let mut checks = Vec::new();
    let supported_tools = allowed_tools.iter().cloned().collect::<BTreeSet<_>>();
    for check in &spec.verification_policy.required_checks {
        let (passed, reason_codes) = match check {
            AgentVerificationCheckV1::Schema => {
                let schema_receipt_ok = !output.receipt.receipt_id.to_string().is_empty()
                    && !output.receipt.context.run_id.as_str().is_empty()
                    && serde_json::to_string(&output.receipt).is_ok();
                (schema_receipt_ok, Vec::new())
            }
            AgentVerificationCheckV1::Sandbox => (
                output
                    .receipt
                    .tool_call_requests
                    .iter()
                    .all(|call| supported_tools.contains(&call.tool_id)),
                Vec::new(),
            ),
            AgentVerificationCheckV1::Digest => {
                (serde_json::to_string(&output.receipt).is_ok(), Vec::new())
            }
            AgentVerificationCheckV1::SupportClaim => {
                let supported = matches!(
                    spec.support_label,
                    AgentSpecSupportLabelV1::Supported | AgentSpecSupportLabelV1::SupportedLocal
                );
                let reason_codes = if supported {
                    Vec::new()
                } else {
                    vec![format!("unsupported-label:{}", spec.support_label)]
                };
                (supported, reason_codes)
            }
        };
        let mut reason_codes = reason_codes;
        if !passed && reason_codes.is_empty() {
            reason_codes.push("check-failed".into());
        }
        checks.push(VerificationReceiptV1 {
            receipt_id: display_only_unstable_id("agent-verification"),
            step: 1,
            check: format!("{check:?}"),
            passed,
            reason_codes,
        });
    }
    checks
}

pub(super) fn turn_output_block_reason_codes(output: &AiDENsRunOutput) -> Vec<String> {
    let mut reason_codes = output
        .receipt
        .stop_rule_receipts
        .iter()
        .flat_map(|receipt| receipt.reason_codes.iter().cloned())
        .collect::<Vec<_>>();
    if reason_codes.is_empty() {
        reason_codes.push(format!(
            "turn-state:{}",
            turn_final_state_code(output.turn_receipt.final_state)
        ));
    }
    reason_codes.sort();
    reason_codes.dedup();
    reason_codes
}

pub(super) fn turn_final_state_code(state: TurnFinalStateV1) -> &'static str {
    match state {
        TurnFinalStateV1::FinalOutput => "final-output",
        TurnFinalStateV1::ProviderUnavailable => "provider-unavailable",
        TurnFinalStateV1::ToolBlocked => "tool-blocked",
        TurnFinalStateV1::ToolFailed => "tool-failed",
        TurnFinalStateV1::BudgetExhausted => "budget-exhausted",
        TurnFinalStateV1::StopRuleTriggered => "stop-rule-triggered",
    }
}

pub(super) fn failed_verification_checks(checks: &[VerificationReceiptV1]) -> Vec<String> {
    checks
        .iter()
        .filter(|receipt| !receipt.passed)
        .map(|receipt| receipt.check.clone())
        .collect()
}

pub(super) fn blocked_turn_action(final_state: TurnFinalStateV1) -> &'static str {
    match final_state {
        TurnFinalStateV1::ToolFailed => "tool-invocation",
        TurnFinalStateV1::ToolBlocked | TurnFinalStateV1::BudgetExhausted => "tool-action",
        TurnFinalStateV1::ProviderUnavailable | TurnFinalStateV1::StopRuleTriggered => "provider",
        _ => "execution-blocked",
    }
}
