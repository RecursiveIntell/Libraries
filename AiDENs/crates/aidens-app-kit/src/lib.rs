//! Safe app builder and profile expansion.

use aidens_config::load_config_file;
use aidens_contracts::{
    AiDENsAppPlanV1, ApiHonestyReportV1, CanonicalToolSideEffectClass, ConfigApplyReportDraftV1,
    ConfigApplyReportV1, MemoryModeV1, ReportLevelV1, RiskDisclosureV1,
};
use aidens_plan_kit::{assemble_execution_plan, ExecutionPlanAssemblyInputV1};
use aidens_provider_kit::{provider_readiness_for_spec, ProviderSpecV1};
use aidens_receipts::CanonicalEventLogConfig;
use aidens_runner::{AiDENsRunInput, AiDENsRunner};
use aidens_tool_kit::{registry_from_enabled_bundles, ToolExposurePolicyV1, ToolRegistryV1};
use anyhow::bail;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiDENsProfile {
    ChatOnly,
    CodingAgent,
    MemoryAgent,
    AutonomousDaemon,
    ResearchWorkbench,
}

impl AiDENsProfile {
    pub fn all() -> [Self; 5] {
        [
            Self::ChatOnly,
            Self::CodingAgent,
            Self::MemoryAgent,
            Self::AutonomousDaemon,
            Self::ResearchWorkbench,
        ]
    }

    pub fn from_id(profile_id: &str) -> Option<Self> {
        match profile_id.trim().to_ascii_lowercase().as_str() {
            "chat" | "chat-only" => Some(Self::ChatOnly),
            "coding" | "coding-agent" => Some(Self::CodingAgent),
            "memory" | "memory-agent" => Some(Self::MemoryAgent),
            "daemon" | "autonomous-daemon" => Some(Self::AutonomousDaemon),
            "research" | "research-workbench" => Some(Self::ResearchWorkbench),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::ChatOnly => "chat-only",
            Self::CodingAgent => "coding-agent",
            Self::MemoryAgent => "memory-agent",
            Self::AutonomousDaemon => "autonomous-daemon",
            Self::ResearchWorkbench => "research-workbench",
        }
    }

    pub fn product_surface_status(self) -> &'static str {
        match self {
            Self::ChatOnly | Self::CodingAgent => "supported",
            Self::MemoryAgent => "partial/proof-only",
            Self::AutonomousDaemon => "partial/safe-mode",
            Self::ResearchWorkbench => "deferred/example-only",
        }
    }

    pub fn product_surface_note(self) -> &'static str {
        match self {
            Self::ChatOnly => "supported chat profile with no tools by default",
            Self::CodingAgent => "supported coding profile; side-effect tools remain permit-gated",
            Self::MemoryAgent => "partial proof surface; canonical memory crates own memory truth",
            Self::AutonomousDaemon => "partial safe-mode surface; autonomous operation is deferred",
            Self::ResearchWorkbench => "deferred/example-only; not a complete product surface",
        }
    }

    pub fn runtime_defaults(
        self,
        app_id: impl Into<String>,
    ) -> anyhow::Result<AiDENsRuntimeDefaultsV1> {
        let app_id = app_id.into();
        let plan = self.expand(app_id.clone())?;
        let provider_route = match self {
            Self::CodingAgent => "explicit-mock-fixture",
            _ => "disabled",
        };
        Ok(AiDENsRuntimeDefaultsV1 {
            app_id,
            profile_id: self.id().into(),
            support_tier: self.product_surface_status().into(),
            provider_route: provider_route.into(),
            receipt_store_root: default_receipt_store_root(&plan.app_id, &plan.receipt_level),
            sandbox_root: ".".into(),
            receipt_level: plan.receipt_level,
            enabled_tool_bundles: plan.enabled_tool_bundles,
            disabled_tool_bundles: plan.disabled_tool_bundles,
            permit_policy: "side-effect-tools-require-explicit-scoped-permits".into(),
            hidden_defaults: false,
            memory_mode: match self {
                Self::ChatOnly | Self::CodingAgent => aidens_contracts::MemoryModeV1::Disabled,
                Self::MemoryAgent | Self::AutonomousDaemon | Self::ResearchWorkbench => {
                    aidens_contracts::MemoryModeV1::Optional
                }
            },
            governance_enabled: matches!(self, Self::AutonomousDaemon),
            kernel_reasoning_enabled: matches!(self, Self::ResearchWorkbench),
        })
    }

    pub fn expand(self, app_id: impl Into<String>) -> anyhow::Result<AiDENsAppPlanV1> {
        let app_id = app_id.into();
        let input = match self {
            Self::CodingAgent => ExecutionPlanAssemblyInputV1 {
                app_id,
                profile_id: self.id().into(),
                provider_required: true,
                memory_mode: MemoryModeV1::Optional,
                receipt_level: ReportLevelV1::Full,
                dangerous_auto_approval: false,
                risk_disclosures: coding_agent_risks(),
                enabled_tool_bundles: vec![
                    "repo-read".into(),
                    "repo-list".into(),
                    "file-stat".into(),
                    "repo-search".into(),
                    "patch-propose".into(),
                    "patch-apply".into(),
                    "run-checks".into(),
                ],
                disabled_tool_bundles: vec![
                    "shell-auto".into(),
                    "network-auto".into(),
                    "file-write-auto".into(),
                    "dangerous-auto-approval".into(),
                ],
            },
            _ => ExecutionPlanAssemblyInputV1 {
                app_id,
                profile_id: self.id().into(),
                provider_required: true,
                memory_mode: MemoryModeV1::Disabled,
                receipt_level: ReportLevelV1::Standard,
                dangerous_auto_approval: false,
                risk_disclosures: Vec::new(),
                enabled_tool_bundles: Vec::new(),
                disabled_tool_bundles: Vec::new(),
            },
        };
        Ok(assemble_execution_plan(input)
            .map_err(anyhow::Error::msg)?
            .plan)
    }
}

fn coding_agent_risks() -> Vec<RiskDisclosureV1> {
    [
        (
            CanonicalToolSideEffectClass::Write,
            "file writes require an explicit permit",
        ),
        (
            CanonicalToolSideEffectClass::Admin,
            "shell execution requires an explicit permit",
        ),
        (
            CanonicalToolSideEffectClass::Analysis,
            "network access requires an explicit permit",
        ),
    ]
    .into_iter()
    .map(|(risk_class, reason)| RiskDisclosureV1 {
        risk_class,
        granted_by_default: false,
        permit_required: true,
        reason: reason.into(),
    })
    .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiDENsRuntimeDefaultsV1 {
    pub app_id: String,
    pub profile_id: String,
    pub support_tier: String,
    pub provider_route: String,
    pub receipt_store_root: Option<String>,
    pub sandbox_root: String,
    pub receipt_level: ReportLevelV1,
    pub enabled_tool_bundles: Vec<String>,
    pub disabled_tool_bundles: Vec<String>,
    pub permit_policy: String,
    pub hidden_defaults: bool,
    pub memory_mode: aidens_contracts::MemoryModeV1,
    pub governance_enabled: bool,
    pub kernel_reasoning_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct AiDENsApp {
    runner: AiDENsRunner,
    plan: AiDENsAppPlanV1,
    config_apply_receipt: Option<ConfigApplyReportV1>,
    api_honesty_receipts: Vec<ApiHonestyReportV1>,
}

impl AiDENsApp {
    pub fn builder() -> AiDENsAppBuilder {
        AiDENsAppBuilder::default()
    }

    /// One-liner quickstart: build a mock agent and run a prompt.
    pub async fn chat(prompt: impl Into<String>) -> anyhow::Result<aidens_runner::AiDENsRunOutput> {
        let plan = AiDENsProfile::ChatOnly.expand("quickstart-chat")?;
        let app = AiDENsApp::from_plan(plan)
            .mock_provider("You are a helpful assistant.")
            .build()
            .await?;
        app.run_once(prompt).await
    }

    /// One-liner with a real provider: build an agent with the given profile
    /// and provider spec, run a prompt.
    pub async fn run_with(
        profile: AiDENsProfile,
        provider: ProviderSpecV1,
        prompt: impl Into<String>,
    ) -> anyhow::Result<aidens_runner::AiDENsRunOutput> {
        let plan = profile.expand("quickstart")?;
        let app = AiDENsApp::from_plan(plan)
            .provider_spec(provider)
            .build()
            .await?;
        app.run_once(prompt).await
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let output = self.run_once("doctor-run").await?;
        println!("{}", output.text);
        Ok(())
    }

    pub async fn run_once(
        &self,
        prompt: impl Into<String>,
    ) -> anyhow::Result<aidens_runner::AiDENsRunOutput> {
        self.runner.run(AiDENsRunInput::new(prompt)).await
    }

    pub fn plan(&self) -> &AiDENsAppPlanV1 {
        &self.plan
    }

    pub fn provider_route(&self) -> aidens_contracts::ProviderRouteReportV1 {
        self.runner.provider_route()
    }

    pub fn tool_ids(&self) -> Vec<String> {
        self.runner.tool_ids()
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tool_ids()
    }

    pub fn executable_tool_ids(&self) -> Vec<String> {
        self.runner.executable_tool_ids()
    }

    pub fn inspect_tools(&self) -> aidens_contracts::ToolExposureSetV1 {
        self.runner.tool_exposure()
    }

    pub fn config_apply_receipt(&self) -> Option<&ConfigApplyReportV1> {
        self.config_apply_receipt.as_ref()
    }

    pub fn api_honesty_receipts(&self) -> &[ApiHonestyReportV1] {
        &self.api_honesty_receipts
    }

    pub fn from_plan(plan: AiDENsAppPlanV1) -> AiDENsAppFromPlanBuilder {
        AiDENsAppFromPlanBuilder {
            plan,
            provider_spec: None,
            tools: None,
        }
    }

    pub fn from_config(config_file: impl Into<String>) -> AiDENsAppBuilder {
        Self::builder().config_file(config_file)
    }
}

#[derive(Debug, Clone)]
pub struct AiDENsAppBuilder {
    name: String,
    profile: AiDENsProfile,
    config_file: Option<String>,
    tools: Option<ToolRegistryV1>,
}

impl Default for AiDENsAppBuilder {
    fn default() -> Self {
        Self {
            name: "aidens-app".into(),
            profile: AiDENsProfile::ChatOnly,
            config_file: None,
            tools: None,
        }
    }
}

impl AiDENsAppBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn profile(mut self, profile: AiDENsProfile) -> Self {
        self.profile = profile;
        self
    }

    pub fn config_file(mut self, config_file: impl Into<String>) -> Self {
        self.config_file = Some(config_file.into());
        self
    }

    pub fn tools(mut self, tools: ToolRegistryV1) -> Self {
        self.tools = Some(tools);
        self
    }

    pub async fn build(self) -> anyhow::Result<AiDENsApp> {
        let mut plan = self.profile.expand(&self.name)?;
        let mut runner = AiDENsRunner::builder();
        let mut tools = self.tools;
        let explicit_tools = tools.is_some();
        let mut sandbox_root = None;
        let mut config_source = None;
        let mut receipt_store_root = default_receipt_store_root(&plan.app_id, &plan.receipt_level);
        let mut memory_store_root = None;
        let mut api_honesty_receipts = Vec::new();
        let mut config_tool_bundles = Vec::new();
        if let Some(config_file) = self.config_file {
            let loaded = load_config_file(config_file)?;
            config_source = Some(format!("loaded {}", loaded.path.display()));
            let profile = config_profile_or_default(&loaded.config, self.profile)?;
            plan = profile.expand(loaded.config.app_id.clone())?;
            plan.memory_mode = loaded.config.memory_mode.clone();
            plan.receipt_level = loaded.config.receipt_level.clone();
            plan.enabled_tool_bundles = loaded.config.tools.enabled_bundles.clone();
            receipt_store_root = receipt_store_root_for_config(&loaded.config, &loaded.path);
            memory_store_root = memory_store_root_for_config(&loaded.config, &loaded.path);
            config_tool_bundles = loaded.config.tools.enabled_bundles.clone();
            sandbox_root = loaded.config.tools.sandbox_root.clone();
            runner = runner
                .provider_spec(provider_spec_from_config(&loaded.config.provider))
                .receipt_level(loaded.config.receipt_level.clone());
            if explicit_tools {
                api_honesty_receipts.push(ApiHonestyReportV1::honored(
                    "AiDENsAppBuilder::tools",
                    vec!["custom-tool-registry".into()],
                    vec!["custom-tool-registry".into()],
                ));
            } else {
                tools = Some(registry_from_enabled_bundles(
                    &loaded.config.tools.enabled_bundles,
                    loaded.config.tools.sandbox_root.as_deref(),
                ));
            }
        }
        plan.validate().map_err(anyhow::Error::msg)?;
        ensure_memory_store_policy(&plan, memory_store_root.as_deref())?;
        let final_tools = tools.unwrap_or_else(|| {
            if plan.enabled_tool_bundles.is_empty() {
                ToolRegistryV1::default()
            } else {
                registry_from_enabled_bundles(&plan.enabled_tool_bundles, Some("."))
            }
        });
        runner = runner
            .tools(final_tools.clone())
            .receipt_level(plan.receipt_level.clone());
        if let Some(root) = receipt_store_root.clone() {
            runner = runner.canonical_receipt_log_config(CanonicalEventLogConfig::for_root(root));
        }
        let runner = runner.app_id(plan.app_id.clone()).build()?;
        let config_apply_receipt = config_source.map(|source| {
            let route = runner.provider_route();
            let policy = ToolExposurePolicyV1::read_only_default().for_provider_route(&route);
            let exposure = final_tools.plan_exposure(&policy);
            let mut reason_codes = Vec::new();
            if explicit_tools && !config_tool_bundles.is_empty() {
                reason_codes.push("builder-tools-override-config-tool-bundles".into());
            }
            if let Some(store) = runner.canonical_receipt_log_config() {
                reason_codes.push(format!(
                    "canonical-receipt-log:{}",
                    store.root_path.display()
                ));
            } else {
                reason_codes.push("receipt-store-not-configured".into());
            }
            if let Some(root) = memory_store_root.as_deref() {
                reason_codes.push(format!("durable-memory-store:{root}"));
            } else if plan.memory_mode == MemoryModeV1::Required {
                reason_codes.push("memory-required-without-durable-store".into());
            }
            ConfigApplyReportV1::new(ConfigApplyReportDraftV1 {
                app_id: plan.app_id.clone(),
                config_source: source,
                provider_route: route,
                tool_exposure: exposure,
                memory_mode: plan.memory_mode.clone(),
                receipt_level: plan.receipt_level.clone(),
                enabled_tool_bundles: config_tool_bundles,
                sandbox_root,
                applied: true,
                reason_codes,
            })
        });
        Ok(AiDENsApp {
            runner,
            plan,
            config_apply_receipt,
            api_honesty_receipts,
        })
    }
}

fn config_profile_or_default(
    cfg: &aidens_config::AiDENsConfigV1,
    default: AiDENsProfile,
) -> anyhow::Result<AiDENsProfile> {
    match cfg.profile_id.as_deref() {
        Some(profile_id) => AiDENsProfile::from_id(profile_id)
            .ok_or_else(|| anyhow::anyhow!("unknown AiDENs profile: {profile_id}")),
        None => Ok(default),
    }
}

fn provider_spec_from_config(provider: &aidens_config::ProviderConfigV1) -> ProviderSpecV1 {
    ProviderSpecV1 {
        kind: provider.kind.clone(),
        model: provider.model.clone(),
        api_key: provider.api_key.clone(),
        base_url: provider.base_url.clone(),
        mock_response: provider.mock_response.clone(),
    }
}

#[derive(Debug, Clone)]
pub struct AiDENsAppFromPlanBuilder {
    plan: AiDENsAppPlanV1,
    provider_spec: Option<ProviderSpecV1>,
    tools: Option<ToolRegistryV1>,
}

impl AiDENsAppFromPlanBuilder {
    pub fn provider_spec(mut self, provider_spec: ProviderSpecV1) -> Self {
        self.provider_spec = Some(provider_spec);
        self
    }

    pub fn mock_provider(mut self, response: impl Into<String>) -> Self {
        self.provider_spec = Some(ProviderSpecV1 {
            kind: "mock".into(),
            mock_response: Some(response.into()),
            ..ProviderSpecV1::default()
        });
        self
    }

    pub fn tools(mut self, tools: ToolRegistryV1) -> Self {
        self.tools = Some(tools);
        self
    }

    pub async fn build(self) -> anyhow::Result<AiDENsApp> {
        self.plan.validate().map_err(anyhow::Error::msg)?;
        ensure_memory_store_policy(&self.plan, None)?;
        let provider_spec = match self.provider_spec {
            Some(provider_spec) => provider_spec,
            None if self.plan.provider_required => {
                bail!(
                    "provider-unbound: AiDENsApp::from_plan requires provider_spec for provider_required plans"
                )
            }
            None => ProviderSpecV1::new("disabled"),
        };
        let readiness = provider_readiness_for_spec(&provider_spec);
        if self.plan.provider_required && !readiness.configured {
            bail!("provider-unbound: provider_required plan cannot use disabled provider")
        }
        if self.plan.provider_required && !readiness.executable {
            bail!(
                "provider-blocked: provider_required plan cannot build executable runtime: {}",
                readiness.reason_codes.join(",")
            )
        }
        let tools = self.tools.unwrap_or_else(|| {
            if self.plan.enabled_tool_bundles.is_empty() {
                ToolRegistryV1::default()
            } else {
                registry_from_enabled_bundles(&self.plan.enabled_tool_bundles, Some("."))
            }
        });
        let runner = AiDENsRunner::builder()
            .app_id(self.plan.app_id.clone())
            .provider_spec(provider_spec)
            .tools(tools)
            .receipt_level(self.plan.receipt_level.clone());
        let runner = if let Some(root) =
            default_receipt_store_root(&self.plan.app_id, &self.plan.receipt_level)
        {
            runner.canonical_receipt_log_config(CanonicalEventLogConfig::for_root(root))
        } else {
            runner
        };
        let runner = runner.build()?;
        Ok(AiDENsApp {
            runner,
            plan: self.plan,
            config_apply_receipt: None,
            api_honesty_receipts: vec![ApiHonestyReportV1::honored(
                "AiDENsApp::from_plan",
                vec![
                    "plan".into(),
                    "provider_spec".into(),
                    "tool_registry".into(),
                ],
                vec![
                    "plan".into(),
                    "provider_spec".into(),
                    "tool_registry".into(),
                ],
            )],
        })
    }
}

fn receipt_store_root_for_config(
    cfg: &aidens_config::AiDENsConfigV1,
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

fn memory_store_root_for_config(
    cfg: &aidens_config::AiDENsConfigV1,
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

fn ensure_memory_store_policy(
    plan: &AiDENsAppPlanV1,
    memory_store_root: Option<&str>,
) -> anyhow::Result<()> {
    if plan.memory_mode == MemoryModeV1::Required && memory_store_root.is_none() {
        bail!(
            "memory-required-without-durable-store: memory_mode=required needs [memory].store_root"
        )
    }
    Ok(())
}

fn default_receipt_store_root(app_id: &str, receipt_level: &ReportLevelV1) -> Option<String> {
    if receipt_level == &ReportLevelV1::Minimal {
        return None;
    }
    Some(format!(
        "target/aidens-receipts/{}",
        receipt_store_segment(app_id)
    ))
}

fn receipt_store_segment(app_id: &str) -> String {
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
        "aidens-app".into()
    } else {
        segment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coding_profile_expands_to_visible_plan_without_dangerous_auto_grants() {
        let plan = AiDENsProfile::CodingAgent.expand("agent").unwrap();

        assert!(!plan.dangerous_auto_approval);
        assert!(!plan.risk_disclosures.is_empty());
        assert!(plan
            .risk_disclosures
            .iter()
            .all(|risk| !risk.granted_by_default && risk.permit_required));
        assert!(plan
            .disabled_tool_bundles
            .contains(&"dangerous-auto-approval".into()));
        assert!(plan.risk_summary().contains("permit_required=true"));
    }

    #[test]
    fn p28_profile_expansion_is_result_bearing_not_panic_based() {
        let plan = AiDENsProfile::CodingAgent
            .expand("p28-result-bearing-agent")
            .expect("valid built-in profile expands");
        assert_eq!(plan.profile_id, "coding-agent");
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn product_profile_catalog_marks_partial_surfaces() {
        let catalog = AiDENsProfile::all()
            .into_iter()
            .map(|profile| (profile.id(), profile.product_surface_status()))
            .collect::<Vec<_>>();

        assert!(catalog.contains(&("chat-only", "supported")));
        assert!(catalog.contains(&("coding-agent", "supported")));
        assert!(catalog.contains(&("memory-agent", "partial/proof-only")));
        assert!(catalog.contains(&("autonomous-daemon", "partial/safe-mode")));
        assert!(catalog.contains(&("research-workbench", "deferred/example-only")));
    }

    #[test]
    fn runtime_defaults_are_visible_receipt_first_and_permit_gated() {
        let defaults = AiDENsProfile::CodingAgent
            .runtime_defaults("generated-agent")
            .unwrap();

        assert_eq!(defaults.profile_id, "coding-agent");
        assert_eq!(defaults.provider_route, "explicit-mock-fixture");
        assert_eq!(defaults.receipt_level, ReportLevelV1::Full);
        assert_eq!(
            defaults.receipt_store_root.as_deref(),
            Some("target/aidens-receipts/generated-agent")
        );
        assert_eq!(defaults.sandbox_root, ".");
        assert!(!defaults.hidden_defaults);
        assert!(defaults
            .disabled_tool_bundles
            .contains(&"dangerous-auto-approval".into()));
        assert!(defaults.permit_policy.contains("explicit"));
    }

    #[tokio::test]
    async fn builder_loads_provider_from_config_file() {
        let dir = std::env::temp_dir().join(format!("aidens-app-kit-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("aidens.toml");
        std::fs::write(
            &path,
            r#"
app_id = "configured-agent"
memory_mode = "optional"
receipt_level = "full"

[provider]
kind = "mock"
model = "mock-model"
mock_response = "configured response"

[receipts]
store_root = "receipts"
"#,
        )
        .unwrap();

        let app = AiDENsApp::builder()
            .name("ignored")
            .profile(AiDENsProfile::CodingAgent)
            .config_file(path.display().to_string())
            .build()
            .await
            .unwrap();

        assert_eq!(app.plan().app_id, "configured-agent");
        let output = app.run_once("hello").await.unwrap();
        assert_eq!(output.text, "configured response");
        assert_eq!(output.receipt.provider_route.unwrap().route_label, "mock");
        assert!(!output.durable_receipt_records.is_empty());
        assert!(dir
            .join("receipts")
            .join("canonical-receipts.ndjson")
            .exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn builder_rejects_unknown_profile_without_fallback() {
        let dir = std::env::temp_dir().join(format!(
            "aidens-app-kit-profile-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("aidens.toml");
        std::fs::write(
            &path,
            r#"
app_id = "configured-agent"
profile_id = "mystery-agent"
memory_mode = "disabled"
receipt_level = "standard"

[provider]
kind = "mock"
mock_response = "configured response"
"#,
        )
        .unwrap();

        let error = AiDENsApp::builder()
            .config_file(path.display().to_string())
            .build()
            .await
            .unwrap_err();

        assert!(error.to_string().contains("unknown AiDENs profile"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn from_plan_requires_explicit_provider_for_provider_required_plan() {
        let plan = AiDENsProfile::CodingAgent
            .expand("from-plan-agent")
            .unwrap();

        let error = AiDENsApp::from_plan(plan).build().await.unwrap_err();

        assert!(error.to_string().contains("provider-unbound"));
    }

    #[tokio::test]
    async fn from_plan_rejects_disabled_provider_for_provider_required_plan() {
        let plan = AiDENsProfile::CodingAgent
            .expand("from-plan-agent")
            .unwrap();

        let error = AiDENsApp::from_plan(plan)
            .provider_spec(ProviderSpecV1::new("disabled"))
            .build()
            .await
            .unwrap_err();

        assert!(error.to_string().contains("provider-unbound"));
        assert!(error.to_string().contains("disabled provider"));
    }

    #[tokio::test]
    async fn from_plan_accepts_a_constructible_ollama_boundary_without_a_liveness_claim() {
        let plan = AiDENsProfile::ChatOnly
            .expand("configured-ollama-agent")
            .unwrap();
        let mut provider = ProviderSpecV1::new("ollama");
        provider.model = Some("fixture-model".into());
        provider.base_url = Some("http://127.0.0.1:9".into());

        let app = AiDENsApp::from_plan(plan)
            .provider_spec(provider)
            .build()
            .await
            .expect("a constructible boundary must be usable even without a live probe receipt");

        assert_eq!(app.provider_route().route_label, "unavailable");
    }

    #[tokio::test]
    async fn from_plan_honors_bound_provider_and_plan_tools() {
        let plan = AiDENsProfile::CodingAgent
            .expand("from-plan-agent")
            .unwrap();
        let receipt_root = Path::new("target")
            .join("aidens-receipts")
            .join("from-plan-agent");
        let _ = std::fs::remove_dir_all(&receipt_root);

        let app = AiDENsApp::from_plan(plan)
            .mock_provider("plan response")
            .build()
            .await
            .unwrap();

        assert_eq!(app.provider_route().route_label, "mock");
        assert!(app.tool_ids().contains(&"aidens:repo-read:1".into()));
        assert!(app
            .api_honesty_receipts()
            .iter()
            .all(ApiHonestyReportV1::all_inputs_honored));
        let output = app.run_once("hello").await.unwrap();
        assert_eq!(output.text, "plan response");
        let _ = std::fs::remove_dir_all(&receipt_root);
    }

    #[tokio::test]
    async fn explicit_builder_tool_registry_is_preserved_over_config_bundles() {
        let dir =
            std::env::temp_dir().join(format!("aidens-app-kit-tools-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("aidens.toml");
        std::fs::write(
            &path,
            r#"
app_id = "configured-agent"
profile_id = "coding-agent"
memory_mode = "optional"
receipt_level = "full"

[provider]
kind = "mock"
model = "mock-model"
mock_response = "configured response"

[tools]
sandbox_root = "."
enabled_bundles = ["safe-coding"]

[receipts]
store_root = "receipts"
"#,
        )
        .unwrap();

        let app = AiDENsApp::builder()
            .config_file(path.display().to_string())
            .tools(ToolRegistryV1::default())
            .build()
            .await
            .unwrap();

        assert!(app.tool_ids().is_empty());
        assert!(app.list_tools().is_empty());
        assert!(app.inspect_tools().exposed_tool_ids.is_empty());
        let receipt = app.config_apply_receipt().expect("config apply receipt");
        assert!(receipt
            .reason_codes
            .contains(&"builder-tools-override-config-tool-bundles".into()));
        assert!(app
            .api_honesty_receipts()
            .iter()
            .any(ApiHonestyReportV1::all_inputs_honored));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
