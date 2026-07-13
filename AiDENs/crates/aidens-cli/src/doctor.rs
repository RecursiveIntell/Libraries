use super::*;

pub(crate) fn receipt_store_truth_for_config(cfg: &AiDENsConfigV1) -> RuntimeCapabilityTruthV1 {
    if let Some(root) = cfg
        .receipts
        .store_root
        .clone()
        .or_else(|| default_receipt_store_root(&cfg.app_id, &cfg.receipt_level))
    {
        let health = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&root))
            .and_then(|log| log.verify_chain());
        return match health {
            Ok(true) => truth(
                "receipts:canonical-log",
                vec![
                    CapabilityStateV1::Configured,
                    CapabilityStateV1::Available,
                    CapabilityStateV1::Healthy,
                ],
                Some(format!(
                    "canonical-receipt-log: verified; root={root}; receipt_level={}",
                    cfg.receipt_level
                )),
            ),
            Ok(false) => truth(
                "receipts:canonical-log",
                vec![
                    CapabilityStateV1::Configured,
                    CapabilityStateV1::Unavailable,
                    CapabilityStateV1::Degraded,
                ],
                Some(format!(
                    "canonical-receipt-log: chain-verification-failed; root={root}; receipt_level={}",
                    cfg.receipt_level
                )),
            ),
            Err(error) => truth(
                "receipts:canonical-log",
                vec![
                    CapabilityStateV1::Configured,
                    CapabilityStateV1::Unavailable,
                    CapabilityStateV1::Degraded,
                ],
                Some(format!(
                    "canonical-receipt-log: health-probe-failed: {error}; root={root}; receipt_level={}",
                    cfg.receipt_level
                )),
            ),
        };
    }
    truth(
        "receipts:minimal-no-durable-store",
        vec![CapabilityStateV1::Configured, CapabilityStateV1::Disabled],
        Some("receipt_level=minimal; no durable store configured".into()),
    )
}

pub(crate) fn provider_capability_truth(cfg: &AiDENsConfigV1) -> RuntimeCapabilityTruthV1 {
    let spec = provider_spec_from_config(&cfg.provider);
    let readiness = provider_readiness_for_spec(&spec);
    let route = route_for_config(cfg);
    let mut states = Vec::new();
    if readiness.configured {
        states.push(CapabilityStateV1::Configured);
    }
    if readiness.executable {
        states.extend([
            CapabilityStateV1::Available,
            CapabilityStateV1::ExecutableThisTurn,
        ]);
        if cfg.provider.kind.trim().eq_ignore_ascii_case("mock") {
            states.push(CapabilityStateV1::Healthy);
        }
    } else if is_disabled_provider(&cfg.provider.kind) {
        states.push(CapabilityStateV1::Disabled);
    } else if readiness
        .reason_codes
        .iter()
        .any(|reason| reason.contains("api-key"))
    {
        states.push(CapabilityStateV1::BlockedByPolicy);
    } else {
        states.push(CapabilityStateV1::Unavailable);
    }
    if route.degraded {
        states.push(CapabilityStateV1::Degraded);
    }
    if route.route_label == "parser-fallback" {
        states.push(CapabilityStateV1::FallbackOnly);
    }
    truth(
        format!("provider:{}", cfg.provider.kind),
        states,
        Some(merged_reason_codes(readiness.reason_codes, route.reason_codes).join(",")),
    )
}

pub(crate) fn provider_capability_matrix_truths() -> Vec<RuntimeCapabilityTruthV1> {
    provider_backend_matrix()
        .entries
        .into_iter()
        .map(|entry| {
            let mut states = vec![CapabilityStateV1::Declared];
            match entry.status {
                ProviderBackendStatusV1::Disabled => {
                    states.push(CapabilityStateV1::Disabled);
                }
                ProviderBackendStatusV1::Executable => {
                    states.push(CapabilityStateV1::Available);
                    states.push(CapabilityStateV1::ExecutableThisTurn);
                    if entry.provider_kind == "mock" {
                        states.push(CapabilityStateV1::Healthy);
                    }
                }
                ProviderBackendStatusV1::BoundaryUnavailable | ProviderBackendStatusV1::Unsupported => {
                    states.push(CapabilityStateV1::Unavailable);
                    states.push(CapabilityStateV1::Deferred);
                }
            }
            if entry.route_label == ProviderRouteKindV1::ParserFallback.to_string() {
                states.push(CapabilityStateV1::FallbackOnly);
            }
            truth(
                format!("provider-matrix:{}", entry.provider_kind),
                states,
                Some(format!(
                    "status={}; route={}; chat_completion_executable={}; native_tool_loop_executable={}; streaming_executable={}; structured_output_executable={}; support_label={}; reason_codes={}",
                    entry.status,
                    entry.route_label,
                    entry.chat_completion_executable,
                    entry.native_tool_loop_executable,
                    entry.streaming_executable,
                    entry.structured_output_executable,
                    provider_matrix_support_label(&entry.provider_kind, entry.status),
                    entry.reason_codes.join(",")
                )),
            )
        })
        .collect()
}

pub(crate) fn provider_matrix_support_label(
    provider_kind: &str,
    status: ProviderBackendStatusV1,
) -> &'static str {
    match (provider_kind, status) {
        ("mock", ProviderBackendStatusV1::Executable) => "fixture-supported-not-cloud",
        ("ollama", ProviderBackendStatusV1::Executable) => "partial-local-chat",
        (_, ProviderBackendStatusV1::Disabled) => "blocked/tested",
        (_, ProviderBackendStatusV1::BoundaryUnavailable) => "deferred/unavailable",
        (_, ProviderBackendStatusV1::Unsupported) => "unsupported",
        _ => "executable-test-backed",
    }
}

pub(crate) fn provider_support_tier(
    provider_kind: &str,
    status: ProviderBackendStatusV1,
) -> &'static str {
    match (provider_kind, status) {
        ("mock", ProviderBackendStatusV1::Executable) => "supported",
        ("ollama", ProviderBackendStatusV1::Executable) => "partial",
        (_, ProviderBackendStatusV1::Executable) => "partial",
        (_, ProviderBackendStatusV1::Disabled) => "deferred",
        (_, ProviderBackendStatusV1::BoundaryUnavailable) => "deferred",
        (_, ProviderBackendStatusV1::Unsupported) => "failed",
    }
}

pub(crate) fn semantic_disclosure_value(
    semantic_status: &str,
    support_tier: impl Into<String>,
    degradation: Vec<String>,
    proof_checks: Vec<String>,
    known_limits: Vec<String>,
) -> Value {
    serde_json::json!({
        "semantic_status": semantic_status,
        "exactness": semantic_status,
        "support_tier": support_tier.into(),
        "degradation": degradation,
        "proof_checks": proof_checks,
        "known_limits": known_limits,
        "reference_semantics": {
            "status": "deferred-unless-canonical-owner-proves-reference-semantics",
            "promotion_rule": "do-not-promote-display-or-advisory-results-without-canonical-owner-proof"
        }
    })
}

pub(crate) fn report_json_with_support_tiers(
    report: &impl serde::Serialize,
    support_tiers: serde_json::Value,
) -> Result<String> {
    let mut value = serde_json::to_value(report)?;
    let Some(object) = value.as_object_mut() else {
        bail!("report JSON must encode as an object");
    };
    object.insert("operator_support_tiers".into(), support_tiers);
    object.insert(
        "semantic_disclosure".into(),
        semantic_disclosure_value(
            "display_only",
            "mixed-operator-report",
            Vec::new(),
            vec!["support-tier-buckets-emitted".into()],
            vec![
                "AiDENs-local operator report; canonical truth remains delegated to owner crates"
                    .into(),
            ],
        ),
    );
    Ok(serde_json::to_string_pretty(&value)?)
}

pub(crate) fn empty_support_tiers() -> BTreeMap<String, Vec<String>> {
    ["supported", "partial", "scaffold", "deferred", "failed"]
        .into_iter()
        .map(|tier| (tier.to_string(), Vec::new()))
        .collect()
}

pub(crate) fn push_support_tier(
    tiers: &mut BTreeMap<String, Vec<String>>,
    tier: &str,
    capability_id: impl Into<String>,
) {
    tiers
        .entry(tier.to_string())
        .or_default()
        .push(capability_id.into());
}

pub(crate) fn finalize_support_tiers(
    mut tiers: BTreeMap<String, Vec<String>>,
) -> serde_json::Value {
    for values in tiers.values_mut() {
        values.sort();
        values.dedup();
    }
    serde_json::to_value(tiers).unwrap_or(serde_json::Value::Null)
}

pub(crate) fn support_tiers_from_doctor(report: &AiDENsDoctorReportV1) -> serde_json::Value {
    let mut tiers = empty_support_tiers();
    for (section, truths) in &report.sections {
        for truth in truths {
            let tier = capability_support_tier(section, truth);
            push_support_tier(&mut tiers, tier, truth.capability_id.clone());
        }
    }
    finalize_support_tiers(tiers)
}

pub(crate) fn support_tiers_from_examples(manifest: &ExampleAppManifestV1) -> serde_json::Value {
    let mut tiers = empty_support_tiers();
    for example in &manifest.examples {
        let tier = release_surface_state_support_tier(&example.status, false);
        push_support_tier(&mut tiers, tier, example.path.clone());
    }
    for feature in &manifest.unsupported_advanced_features {
        push_support_tier(&mut tiers, "scaffold", feature.clone());
    }
    finalize_support_tiers(tiers)
}

pub(crate) fn support_tiers_from_surfaces(surfaces: &[ReleaseSurfaceV1]) -> serde_json::Value {
    let mut tiers = empty_support_tiers();
    for surface in surfaces {
        let is_scaffold = surface
            .reason
            .to_ascii_lowercase()
            .contains("scaffold-only");
        let tier = release_surface_state_support_tier(&surface.state, is_scaffold);
        push_support_tier(&mut tiers, tier, surface.surface_id.clone());
    }
    finalize_support_tiers(tiers)
}

pub(crate) fn release_surface_state_support_tier(
    state: &ReleaseSurfaceStateV1,
    scaffold: bool,
) -> &'static str {
    if scaffold {
        return "scaffold";
    }
    match state {
        ReleaseSurfaceStateV1::Supported => "supported",
        ReleaseSurfaceStateV1::Partial | ReleaseSurfaceStateV1::Degraded => "partial",
        ReleaseSurfaceStateV1::Deferred => "deferred",
        ReleaseSurfaceStateV1::Blocked => "failed",
    }
}

pub(crate) fn capability_support_tier(
    section: &str,
    truth: &RuntimeCapabilityTruthV1,
) -> &'static str {
    let reason = truth
        .reason
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if section == "scaffold_surfaces" || reason.contains("scaffold-only") {
        return "scaffold";
    }
    if truth.states.contains(&CapabilityStateV1::Failed) {
        return "failed";
    }
    if truth.states.contains(&CapabilityStateV1::Deferred)
        || reason.contains("deferred")
        || reason.contains("provider-boundary-unavailable")
    {
        return "deferred";
    }
    if truth.states.contains(&CapabilityStateV1::Unavailable) {
        return "failed";
    }
    if truth.states.contains(&CapabilityStateV1::Degraded)
        || truth.states.contains(&CapabilityStateV1::FallbackOnly)
        || truth.states.contains(&CapabilityStateV1::BlockedByPolicy)
        || truth.states.contains(&CapabilityStateV1::RequiresApproval)
        || truth.states.contains(&CapabilityStateV1::Hidden)
        || reason.contains("delegated")
    {
        return "partial";
    }
    if truth.states.contains(&CapabilityStateV1::Disabled) {
        return "deferred";
    }
    if matches!(
        section,
        "daemon" | "governance" | "memory" | "queue" | "repair" | "runtime" | "schedule" | "wake"
    ) {
        return "partial";
    }
    "supported"
}

pub(crate) fn tool_support_tier(
    executable: bool,
    exposed: bool,
    hidden: bool,
    blocked: bool,
    requires_permit: bool,
    registered: bool,
) -> &'static str {
    if blocked || hidden || requires_permit {
        return "partial";
    }
    if executable && exposed {
        return "supported";
    }
    if registered || executable {
        return "partial";
    }
    "deferred"
}

pub(crate) fn provider_route_truth(cfg: &AiDENsConfigV1) -> RuntimeCapabilityTruthV1 {
    let route = route_for_config(cfg);
    let mut states = vec![CapabilityStateV1::Configured];
    match route.route {
        ProviderRouteKindV1::Disabled => {
            states = vec![CapabilityStateV1::Disabled, CapabilityStateV1::Deferred];
        }
        ProviderRouteKindV1::Unavailable => {
            states.push(CapabilityStateV1::Unavailable);
        }
        ProviderRouteKindV1::ParserFallback | ProviderRouteKindV1::Degraded => {
            states.push(CapabilityStateV1::Degraded);
        }
        _ => {
            states.extend([
                CapabilityStateV1::Available,
                CapabilityStateV1::ExecutableThisTurn,
            ]);
        }
    }
    truth(
        format!("provider-route:{}", route.route_label),
        states,
        Some(route.reason_codes.join(",")),
    )
}

pub(crate) fn memory_truth_for_config(cfg: &AiDENsConfigV1) -> RuntimeCapabilityTruthV1 {
    let store_root = cfg.memory.store_root.as_deref();
    let (states, reason) = match (&cfg.memory_mode, store_root) {
        (MemoryModeV1::Disabled, Some(root)) => (
            vec![CapabilityStateV1::Disabled],
            format!(
                "memory_mode=disabled; durable memory store configured but unused; root={root}"
            ),
        ),
        (MemoryModeV1::Disabled, None) => (
            vec![CapabilityStateV1::Disabled],
            "memory_mode=disabled; durable memory store not configured".to_string(),
        ),
        (MemoryModeV1::Optional, Some(root)) => (
            vec![
                CapabilityStateV1::Configured,
                CapabilityStateV1::Available,
                CapabilityStateV1::Healthy,
            ],
            format!("memory_mode=optional; durable memory store configured; root={root}"),
        ),
        (MemoryModeV1::Optional, None) => (
            vec![CapabilityStateV1::Configured, CapabilityStateV1::Degraded],
            "memory_mode=optional; durable memory store not configured".to_string(),
        ),
        (MemoryModeV1::Required, Some(root)) => (
            vec![
                CapabilityStateV1::Configured,
                CapabilityStateV1::Available,
                CapabilityStateV1::Healthy,
            ],
            format!("memory_mode=required; durable memory store configured; root={root}"),
        ),
        (MemoryModeV1::Required, None) => (
            vec![
                CapabilityStateV1::Configured,
                CapabilityStateV1::BlockedByPolicy,
            ],
            "memory_mode=required; memory-required-without-durable-store".to_string(),
        ),
    };
    truth("memory:runtime", states, Some(reason))
}

pub(crate) fn doctor_report_for_config(
    config_status: &str,
    cfg: &AiDENsConfigV1,
) -> AiDENsDoctorReportV1 {
    let registry = tool_registry_for_config(cfg);
    let exposure = tool_exposure_for_config(cfg);
    let mut sections = BTreeMap::new();

    sections.insert(
        "config".into(),
        vec![truth(
            "config:file",
            vec![
                CapabilityStateV1::Configured,
                CapabilityStateV1::Available,
                CapabilityStateV1::Healthy,
            ],
            Some(config_status.into()),
        )],
    );
    sections.insert("provider".into(), vec![provider_capability_truth(cfg)]);
    sections.insert("provider_route".into(), vec![provider_route_truth(cfg)]);
    sections.insert(
        "provider_capability_matrix".into(),
        provider_capability_matrix_truths(),
    );

    let tool_truth = exposure
        .declared_tool_ids
        .iter()
        .cloned()
        .map(|tool_id| {
            let mut states = vec![CapabilityStateV1::Declared];
            if registry.contains_tool_id(&tool_id) {
                states.push(CapabilityStateV1::Registered);
            }
            if exposure.exposed_tool_ids.contains(&tool_id) {
                states.push(CapabilityStateV1::ExposedThisTurn);
            }
            if registry.can_execute(&tool_id) {
                states.push(CapabilityStateV1::ExecutableThisTurn);
            } else if registry.contains_tool_id(&tool_id) {
                states.push(CapabilityStateV1::Deferred);
            }
            if exposure.hidden_tool_ids.contains(&tool_id) {
                states.push(CapabilityStateV1::Hidden);
            }
            if exposure.blocked_tool_ids.contains(&tool_id) {
                states.push(CapabilityStateV1::BlockedByPolicy);
            }
            let reason = exposure
                .decisions
                .iter()
                .find(|decision| decision.capability_id == tool_id)
                .map(|decision| decision.reason_codes.join(","))
                .filter(|reason| !reason.is_empty())
                .unwrap_or_else(|| "read-only policy exposure".into());
            if reason.contains("permit-required") {
                states.push(CapabilityStateV1::RequiresApproval);
            }
            truth(tool_id, states, Some(reason))
        })
        .collect::<Vec<_>>();
    sections.insert("tools".into(), tool_truth);

    sections.insert(
        "security".into(),
        vec![
            truth(
                "security:approval",
                vec![CapabilityStateV1::Configured, CapabilityStateV1::Healthy],
                Some(format!(
                    "approval_mode={}, write_policy={}",
                    cfg.security.approval_mode, cfg.security.write_policy
                )),
            ),
            truth(
                "security:network",
                vec![CapabilityStateV1::Disabled],
                Some(cfg.security.network_policy.clone()),
            ),
        ],
    );
    sections.insert("receipts".into(), vec![receipt_store_truth_for_config(cfg)]);
    sections.insert("memory".into(), vec![memory_truth_for_config(cfg)]);
    sections.insert(
        "governance".into(),
        vec![truth(
            "governance:p12-promotion-policy",
            vec![CapabilityStateV1::Configured, CapabilityStateV1::Available],
            Some("Risk-bearing promotion is delegated to verification-control and verification-adjudication".into()),
        )],
    );
    sections.insert(
        "repair".into(),
        vec![truth(
            "repair:p12-supersession-records",
            vec![CapabilityStateV1::Configured, CapabilityStateV1::Available],
            Some(
                "Repairs are delegated to verification-control and semantic-memory-forge records"
                    .into(),
            ),
        )],
    );
    sections.insert(
        "queue".into(),
        vec![truth(
            "queue:runtime",
            vec![CapabilityStateV1::Configured, CapabilityStateV1::Available],
            Some("P11 append-only daemon-owned queue substrate; jobs require namespace root and idempotency key".into()),
        )],
    );
    sections.insert(
        "schedule".into(),
        vec![truth(
            "schedule:runtime",
            vec![CapabilityStateV1::Configured, CapabilityStateV1::Available],
            Some("P11 one-shot schedule occurrence compiler; recurring schedules remain deferred until built on idempotency keys".into()),
        )],
    );
    sections.insert(
        "wake".into(),
        vec![truth(
            "wake:runtime",
            vec![CapabilityStateV1::Configured, CapabilityStateV1::Available],
            Some("P11 wake signals produce idempotency-keyed queue jobs; no network/file side effects by default".into()),
        )],
    );
    sections.insert(
        "daemon".into(),
        vec![truth(
            "daemon:runtime",
            vec![CapabilityStateV1::Configured, CapabilityStateV1::Available],
            Some("P11 daemon controller requires owner-scoped writes, leases, safe mode, and duplicate suppression".into()),
        )],
    );
    sections.insert(
        "runtime".into(),
        vec![truth(
            "runtime:runner",
            vec![CapabilityStateV1::Configured, CapabilityStateV1::Available],
            Some("runner uses executable provider boundary and P04 tool lifecycle gates".into()),
        )],
    );
    sections.insert("scaffold_surfaces".into(), scaffold_surface_truths());

    AiDENsDoctorReportV1::new(cfg.app_id.clone(), sections)
}

pub(crate) fn scaffold_surface_truths() -> Vec<RuntimeCapabilityTruthV1> {
    SCAFFOLD_ONLY_CRATES
        .iter()
        .map(|(crate_name, note)| {
            truth(
                format!("crate:{crate_name}"),
                vec![CapabilityStateV1::Disabled, CapabilityStateV1::Deferred],
                Some(format!("scaffold-only; {note}")),
            )
        })
        .collect()
}

pub(crate) fn tool_registry_for_config(cfg: &AiDENsConfigV1) -> ToolRegistryV1 {
    registry_from_enabled_bundles(
        &cfg.tools.enabled_bundles,
        cfg.tools.sandbox_root.as_deref(),
    )
}

pub(crate) fn tool_exposure_for_config(
    cfg: &AiDENsConfigV1,
) -> aidens_contracts::ToolExposureSetV1 {
    let route = route_for_config(cfg);
    let mut policy = if cfg.profile_id.as_deref() == Some("coding-agent") {
        ToolExposurePolicyV1::coding_agent_default()
    } else {
        ToolExposurePolicyV1::read_only_default()
    }
    .for_provider_route(&route);
    if let Some(sandbox_root) = cfg.tools.sandbox_root.as_deref() {
        policy = policy.with_sandbox_root(sandbox_root);
    }
    tool_registry_for_config(cfg)
        .plan_exposure_with_declarations(&policy, safe_coding_tool_declarations())
}

pub(crate) fn plan_runtime_parity_report(
    plan: &AiDENsAppPlanV1,
    provider_route: &ProviderRouteReportV1,
    tool_exposure: &ToolExposureSetV1,
    doctor: &AiDENsDoctorReportV1,
) -> PlanRuntimeParityReportV1 {
    let checks = vec![
        PlanRuntimeParityCheckV1::new(
            PlanRuntimeParityCheckKindV1::ProviderRoute,
            provider_route.route_label.clone(),
            doctor_provider_route_label(doctor),
        ),
        PlanRuntimeParityCheckV1::new(
            PlanRuntimeParityCheckKindV1::ToolExposure,
            sorted_join(tool_exposure.exposed_tool_ids.clone()),
            sorted_join(doctor_exposed_tool_ids(doctor)),
        ),
        PlanRuntimeParityCheckV1::new(
            PlanRuntimeParityCheckKindV1::MemoryMode,
            plan.memory_mode.to_string(),
            doctor_memory_mode(doctor),
        ),
        PlanRuntimeParityCheckV1::new(
            PlanRuntimeParityCheckKindV1::ScaffoldState,
            format!("{} disabled/deferred", SCAFFOLD_ONLY_CRATES.len()),
            doctor_scaffold_state(doctor),
        ),
    ];
    PlanRuntimeParityReportV1::new(plan.app_id.clone(), checks)
}

pub(crate) fn doctor_provider_route_label(doctor: &AiDENsDoctorReportV1) -> String {
    doctor
        .sections
        .get("provider_route")
        .and_then(|section| section.first())
        .and_then(|truth| truth.capability_id.strip_prefix("provider-route:"))
        .unwrap_or("<missing>")
        .to_string()
}

pub(crate) fn doctor_exposed_tool_ids(doctor: &AiDENsDoctorReportV1) -> Vec<String> {
    doctor
        .sections
        .get("tools")
        .into_iter()
        .flat_map(|section| section.iter())
        .filter(|truth| truth.states.contains(&CapabilityStateV1::ExposedThisTurn))
        .map(|truth| truth.capability_id.clone())
        .collect()
}

pub(crate) fn doctor_memory_mode(doctor: &AiDENsDoctorReportV1) -> String {
    doctor
        .sections
        .get("memory")
        .and_then(|section| section.first())
        .and_then(|truth| truth.reason.as_deref())
        .and_then(|reason| {
            reason
                .strip_prefix("memory_mode=")
                .and_then(|rest| rest.split(';').next())
        })
        .unwrap_or("<missing>")
        .to_string()
}

pub(crate) fn doctor_scaffold_state(doctor: &AiDENsDoctorReportV1) -> String {
    let Some(section) = doctor.sections.get("scaffold_surfaces") else {
        return "<missing>".into();
    };
    let all_deferred = section.iter().all(|truth| {
        truth.states.contains(&CapabilityStateV1::Disabled)
            && truth.states.contains(&CapabilityStateV1::Deferred)
            && !truth.states.contains(&CapabilityStateV1::Healthy)
    });
    if all_deferred {
        format!("{} disabled/deferred", section.len())
    } else {
        "promoted-or-mixed".into()
    }
}

pub(crate) fn sorted_join(mut values: Vec<String>) -> String {
    values.sort();
    values.join(",")
}

pub(crate) fn merged_reason_codes(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    left.extend(right);
    left.sort();
    left.dedup();
    left
}
