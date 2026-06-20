use super::*;

pub(crate) struct PlanConfigLoadOutcome {
    pub(crate) config_status: String,
    pub(crate) config: AiDENsConfigV1,
}

pub(crate) fn load_plan_config_file(path: &str) -> Result<PlanConfigLoadOutcome> {
    match load_config_file(path) {
        Ok(loaded) => Ok(PlanConfigLoadOutcome {
            config_status: format!("loaded {}", loaded.path.display()),
            config: loaded.config,
        }),
        Err(config_error) => load_test_agent_plan_config(path)
            .with_context(|| format!("failed to load {path} as AiDENs config ({config_error})")),
    }
}

pub(crate) fn load_test_agent_plan_config(path: &str) -> Result<PlanConfigLoadOutcome> {
    let config_path = resolve_cli_path(path)?;
    let test_agent = load_test_agent_file(&config_path)?;
    let reference_root = test_agent_reference_root(&config_path)?;
    let agent_segment = receipt_store_segment(&test_agent.agent.name);
    let sandbox_root = test_agent
        .tools
        .sandbox_root
        .as_ref()
        .map(|root| resolve_against_root(root, &reference_root))
        .unwrap_or_else(|| {
            reference_root
                .join("target/p21/plan-sandbox")
                .join(&agent_segment)
        });
    let receipt_root = test_agent
        .receipts
        .store_root
        .as_ref()
        .map(|root| resolve_against_root(root, &reference_root))
        .unwrap_or_else(|| {
            reference_root
                .join("target/p21/plan-receipts")
                .join(&agent_segment)
        });
    let (config, fixture_path, _) = test_agent_effective_config(
        &config_path,
        &test_agent,
        &sandbox_root,
        &receipt_root,
        false,
    )?;
    Ok(PlanConfigLoadOutcome {
        config_status: format!(
            "loaded test-agent {} via fixture {}",
            config_path.display(),
            fixture_path.display()
        ),
        config,
    })
}

pub(crate) fn load_or_default_config(path: &str) -> Result<(String, AiDENsConfigV1)> {
    let path_ref = Path::new(path);
    if path_ref.exists() {
        let loaded = load_config_file(path_ref)?;
        Ok((format!("loaded {}", loaded.path.display()), loaded.config))
    } else {
        Ok((
            format!(
                "missing {}; using safe disabled defaults",
                path_ref.display()
            ),
            AiDENsConfigV1::safe_default("aidens-cli"),
        ))
    }
}

pub(crate) fn receipt_store_config_from_options(
    store: Option<String>,
    config: Option<String>,
) -> Result<CanonicalEventLogConfig> {
    if let Some(root) = store {
        return Ok(CanonicalEventLogConfig::for_root(root));
    }
    if let Some(config) = config {
        let loaded = load_config_file(&config)?;
        let root =
            receipt_store_root_for_config(&loaded.config, &loaded.path).ok_or_else(|| {
                anyhow::anyhow!(
                    "receipt-store-not-configured: receipt_level=minimal and no --store override"
                )
            })?;
        return Ok(CanonicalEventLogConfig::for_root(root));
    }
    Ok(CanonicalEventLogConfig::for_root(
        "target/aidens-receipts/aidens-cli",
    ))
}

pub(crate) fn receipt_store_root_for_config(
    cfg: &AiDENsConfigV1,
    config_path: &Path,
) -> Option<String> {
    let Some(root) = cfg.receipts.store_root.clone() else {
        return default_receipt_store_root(&cfg.app_id, &cfg.receipt_level);
    };
    let path = PathBuf::from(root);
    if path.is_absolute() {
        return Some(path.display().to_string());
    }
    Some(
        config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path)
            .display()
            .to_string(),
    )
}

pub(crate) fn memory_store_root_for_config(
    cfg: &AiDENsConfigV1,
    config_path: &Path,
) -> Option<String> {
    let root = cfg.memory.store_root.clone()?;
    let path = PathBuf::from(root);
    if path.is_absolute() {
        return Some(path.display().to_string());
    }
    Some(
        config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path)
            .display()
            .to_string(),
    )
}

pub(crate) fn default_receipt_store_root(
    app_id: &str,
    receipt_level: &ReportLevelV1,
) -> Option<String> {
    if receipt_level == &ReportLevelV1::Minimal {
        return None;
    }
    Some(format!(
        "target/aidens-receipts/{}",
        receipt_store_segment(app_id)
    ))
}

pub(crate) fn receipt_store_segment(app_id: &str) -> String {
    let mut segment = String::new();
    let mut last_dash = false;
    for ch in app_id.trim().chars() {
        let next = if ch.is_ascii_alphanumeric() || ch == '_' {
            ch.to_ascii_lowercase()
        } else {
            '-'
        };
        if next == '-' {
            if !last_dash {
                segment.push(next);
            }
            last_dash = true;
        } else {
            segment.push(next);
            last_dash = false;
        }
    }
    let segment = segment.trim_matches('-').to_string();
    if segment.is_empty() {
        "aidens-cli".into()
    } else {
        segment
    }
}

pub(crate) fn plan_from_config(cfg: &AiDENsConfigV1) -> Result<AiDENsAppPlanV1> {
    let profile = profile_for_config(cfg)?;
    let mut input = ExecutionPlanAssemblyInputV1::from(profile.expand(cfg.app_id.clone())?);
    input.memory_mode = cfg.memory_mode.clone();
    input.receipt_level = cfg.receipt_level.clone();
    input.enabled_tool_bundles = cfg.tools.enabled_bundles.clone();
    Ok(assemble_execution_plan(input)
        .map_err(anyhow::Error::msg)?
        .plan)
}

pub(crate) fn compile_config_plan(
    config_status: &str,
    cfg: &AiDENsConfigV1,
) -> Result<AiDENsCompiledPlanV1> {
    let plan = plan_from_config(cfg)?;
    plan.validate().map_err(anyhow::Error::msg)?;
    validate_plan_runtime_contract(&plan, cfg)?;
    let provider_route = route_for_config(cfg);
    let tool_exposure = tool_exposure_for_config(cfg);
    let doctor = doctor_report_for_config(config_status, cfg);
    let config_apply_receipt = ConfigApplyReportV1::new(ConfigApplyReportDraftV1 {
        app_id: cfg.app_id.clone(),
        config_source: config_status.into(),
        provider_route: provider_route.clone(),
        tool_exposure: tool_exposure.clone(),
        memory_mode: plan.memory_mode.clone(),
        receipt_level: plan.receipt_level.clone(),
        enabled_tool_bundles: cfg.tools.enabled_bundles.clone(),
        sandbox_root: cfg.tools.sandbox_root.clone(),
        applied: true,
        reason_codes: vec!["plan-kit:execution-plan-assembly-only".into()],
    });
    let parity_report = plan_runtime_parity_report(&plan, &provider_route, &tool_exposure, &doctor);
    if !parity_report.is_passing() {
        bail!(
            "plan/runtime parity failed: {}",
            parity_report.mismatches.join("; ")
        );
    }
    Ok(AiDENsCompiledPlanV1 {
        plan_id: ArtifactId::new("compiled-plan"),
        plan,
        provider_route,
        tool_exposure,
        doctor,
        config_apply_receipt,
        parity_report,
    })
}

pub(crate) fn validate_plan_runtime_contract(
    plan: &AiDENsAppPlanV1,
    cfg: &AiDENsConfigV1,
) -> Result<()> {
    if plan.provider_required && is_disabled_provider(&cfg.provider.kind) {
        bail!("provider_required plan cannot treat disabled provider as valid")
    }
    let readiness = provider_readiness_for_spec(&provider_spec_from_config(&cfg.provider));
    if plan.provider_required && !readiness.executable {
        bail!(
            "provider_required plan has no executable provider: {}",
            readiness.reason_codes.join(",")
        )
    }
    ensure_memory_store_policy(cfg)?;
    Ok(())
}

pub(crate) fn profile_for_config(cfg: &AiDENsConfigV1) -> Result<AiDENsProfile> {
    match cfg.profile_id.as_deref() {
        Some(profile_id) => AiDENsProfile::from_id(profile_id)
            .ok_or_else(|| anyhow::anyhow!("unknown AiDENs profile: {profile_id}")),
        None => Ok(AiDENsProfile::ChatOnly),
    }
}

pub(crate) fn ensure_profile_policy(cfg: &AiDENsConfigV1) -> Result<()> {
    profile_for_config(cfg).map(|_| ())
}

pub(crate) fn ensure_memory_store_policy(cfg: &AiDENsConfigV1) -> Result<()> {
    if cfg.memory_mode == MemoryModeV1::Required && cfg.memory.store_root.is_none() {
        bail!(
            "memory-required-without-durable-store: memory_mode=required needs [memory].store_root"
        )
    }
    Ok(())
}

pub(crate) fn route_for_config(cfg: &AiDENsConfigV1) -> ProviderRouteReportV1 {
    let spec = provider_spec_from_config(&cfg.provider);
    route_receipt_for_spec(&spec)
}

pub(crate) fn is_disabled_provider(kind: &str) -> bool {
    matches!(
        kind.trim().to_ascii_lowercase().as_str(),
        "" | "disabled" | "none"
    )
}

pub(crate) fn provider_spec_from_config(provider: &ProviderConfigV1) -> ProviderSpecV1 {
    ProviderSpecV1 {
        kind: provider.kind.clone(),
        model: provider.model.clone(),
        api_key: provider.api_key.clone(),
        base_url: provider.base_url.clone(),
        mock_response: provider.mock_response.clone(),
    }
}
