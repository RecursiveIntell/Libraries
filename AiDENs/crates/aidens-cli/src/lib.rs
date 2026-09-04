use aidens_app_kit::{AiDENsApp, AiDENsProfile};
use aidens_boundary_kit::{compile_json_boundary, parse_strict_json, validate_json_schema};
use aidens_capability_kit::truth;
use aidens_config::{load_config_file, AiDENsConfigV1, ProviderConfigV1};
use aidens_contracts::{
    current_artifact_family_registry, generated_artifact_id_from_material,
    generated_schema_documents, generated_schema_manifest, generated_schema_manifest_pretty_json,
    non_authoritative_text_display_digest, AgentPermitRuleV1, AgentSpecV1, AiDENsAppPlanV1,
    AiDENsCompiledPlanV1, AiDENsDoctorReportV1, AiDENsRunBudgetDeadlineV1, AiDENsRunBundleV2,
    AiDENsRunBundleV3, AiDENsRunEventLogDigestV1, AiDENsRunFailureClassV1,
    AiDENsRunFailureTaxonomyV1, AiDENsRunReplayNormalizationV1, AiDENsRunSupportTierEvidenceV1,
    ApprovalDecisionV1, ApprovalRequestV1, ArtifactId, BoundaryCompileRequestV1,
    CanonicalBackpointerV1, CanonicalToolSideEffectClass, CapabilityStateV1, CodexPacketInputV1,
    CodexPacketV1, CommandRunReportV1, CompletionAuditReportV1, ConfigApplyReportDraftV1,
    ConfigApplyReportV1, CrossPassTraceabilityMatrixV1, CrossPassTraceabilityRowV1,
    DisplayDigestV1, ExampleAppEntryV1, ExampleAppManifestV1, GateCommandResultV1,
    InstallSmokeReportV1, InstallSmokeStepV1, KnownLimitationV1, KnownLimitationsRegisterV1,
    MemoryModeV1, OperatorStatusReportV1, PassCompletionStateV1, PermitGrantV1, PermitUseReportV1,
    PlanRuntimeParityCheckKindV1, PlanRuntimeParityCheckV1, PlanRuntimeParityReportV1,
    ProviderBackendStatusV1, ProviderRouteKindV1, ProviderRouteReportV1, PublicDocFindingV1,
    RegressionDebtItemV1, RegressionDebtLedgerV1, ReleaseArtifactEntryV1, ReleaseArtifactKindV1,
    ReleaseArtifactManifestV1, ReleaseReadinessReportV1, ReleaseSurfaceStateV1, ReleaseSurfaceV1,
    ReportLevelV1, RuntimeCapabilityTruthV1, SandboxCapabilityTruthV1, SchemaCompatibilityCheckV1,
    SchemaCompatibilityModeV1, SchemaCompatibilityReportV1, SchemaPathCollisionFindingV1,
    StackAttemptId, StackContentDigest, StackTrialId, ToolExposureSetV1,
};
use aidens_daemon_kit::DaemonControllerV1;
use aidens_memory_kit::{
    memory_config_for_root, runtime_config_for_namespace, CanonicalMemoryAdapter,
};
use aidens_permit_kit::PermitPolicyV1;
use aidens_plan_kit::{assemble_execution_plan, ExecutionPlanAssemblyInputV1};
use aidens_provider_kit::{
    provider_backend_matrix, provider_readiness_for_spec, route_receipt_for_spec, ProviderSpecV1,
};
use aidens_receipts::{
    CanonicalEventLog, CanonicalEventLogConfig, RunBundleStore, RunBundleStoreConfig,
};
use aidens_runner::{PlanActVerifyLoopV1, PlanActVerifyLoopV1Output, PlanActVerifyOutcomeV1};
use aidens_tool_kit::{
    registry_from_enabled_bundles, safe_coding_registry_for_current_dir,
    safe_coding_tool_declarations, ToolDispatcher, ToolExposurePolicyV1, ToolInvocationError,
    ToolInvocationOutcome, ToolRegistryV1,
};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

mod agent;
mod package;

pub use agent::*;
pub use package::*;

const TEST_AGENT_MOCK_RESPONSE_DELIMITER: &str = "\n---aidens-next-response---\n";
const TEST_AGENT_SEEDED_README: &str =
    "AiDENs canonical test agent fixture\nstatus: executable vertical slice\n";

const SCAFFOLD_ONLY_CRATES: &[(&str, &str)] = &[
    (
        "aidens-profile-daemon",
        "deferred until daemon profile wiring",
    ),
    (
        "aidens-profile-desktop",
        "deferred until product surface work",
    ),
    (
        "aidens-profile-memory",
        "deferred until memory profile wiring",
    ),
    (
        "aidens-profile-research",
        "deferred until research profile wiring",
    ),
];

#[derive(Debug, Parser)]
#[command(name = "aidens")]
#[command(about = "AiDENs app-construction CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Profile {
        #[command(subcommand)]
        command: ProfileCommand,
    },
    Plan {
        #[command(subcommand)]
        command: PlanCommand,
    },
    Doctor {
        #[arg(long)]
        config: Option<String>,
    },
    Status {
        #[arg(long)]
        config: Option<String>,
    },
    CheckConfig {
        file: Option<String>,
    },
    ListTools,
    InspectTools {
        #[arg(long)]
        config: Option<String>,
    },
    Tools {
        #[command(subcommand)]
        command: ToolsCommand,
    },
    Permit {
        #[command(subcommand)]
        command: PermitCommand,
    },
    Permits {
        #[command(subcommand)]
        command: PermitCommand,
    },
    Receipts {
        #[command(subcommand)]
        command: EventLogCommand,
    },
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
    Boundary {
        #[command(subcommand)]
        command: BoundaryCommand,
    },
    View {
        #[command(subcommand)]
        command: ViewCommand,
    },
    Schemas {
        #[command(subcommand)]
        command: SchemasCommand,
    },
    Coding {
        #[command(subcommand)]
        command: CodingCommand,
    },
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    Queue {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    Package {
        #[command(subcommand)]
        command: PackageCommand,
    },
    ListCapabilities,
    ProviderCheck {
        #[arg(long)]
        config: Option<String>,
        file: Option<String>,
    },
    Run {
        #[arg(long)]
        config: Option<String>,
        prompt: Vec<String>,
    },
    RunTestAgent {
        config: String,
        #[arg(long)]
        prompt: Option<String>,
        #[arg(long)]
        out: Option<String>,
    },
    RunCodingAgent {
        config: String,
        #[arg(long)]
        out: Option<String>,
        #[arg(long)]
        permit_json: Option<String>,
    },
    InspectRun {
        dir: String,
    },
    New {
        profile: String,
        destination: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ProfileCommand {
    List,
    Explain { profile: String },
}

#[derive(Debug, Subcommand)]
pub enum PlanCommand {
    Validate {
        #[arg(long)]
        config: String,
    },
    Compile {
        #[arg(long)]
        config: String,
        #[arg(long)]
        out: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ToolsCommand {
    List,
    Inspect {
        #[arg(long)]
        config: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum PermitCommand {
    Inspect,
    Request {
        #[arg(long)]
        tool_id: String,
        #[arg(long)]
        risk: String,
        #[arg(long, default_value = ".")]
        sandbox_root: String,
    },
    Approve {
        #[arg(long)]
        request_id: String,
        #[arg(long)]
        tool_id: String,
        #[arg(long)]
        risk: String,
        #[arg(long, default_value = ".")]
        sandbox_root: String,
        #[arg(long, default_value = "operator")]
        decided_by: String,
    },
    Deny {
        #[arg(long)]
        request_id: String,
        #[arg(long, default_value = "operator")]
        decided_by: String,
        #[arg(long, default_value = "operator-denied")]
        reason: String,
    },
    Revoke {
        #[arg(long)]
        permit_id: String,
        #[arg(long)]
        tool_id: String,
        #[arg(long)]
        risk: String,
        #[arg(long, default_value = ".")]
        sandbox_root: String,
        #[arg(long, default_value = "operator-revoked")]
        reason: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum MemoryCommand {
    Status {
        #[arg(long)]
        config: Option<String>,
        #[arg(long)]
        store: Option<String>,
    },
    SeamFixture {
        #[arg(long)]
        out: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum EventLogCommand {
    List {
        #[arg(long)]
        store: Option<String>,
        #[arg(long)]
        config: Option<String>,
    },
    Inspect {
        #[arg(long)]
        store: Option<String>,
        #[arg(long)]
        config: Option<String>,
        #[arg(long)]
        receipt_id: String,
    },
    Export {
        #[arg(long)]
        store: Option<String>,
        #[arg(long)]
        config: Option<String>,
    },
    VerifyDigest {
        #[arg(long)]
        store: Option<String>,
        #[arg(long)]
        config: Option<String>,
        #[arg(long)]
        receipt_id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum BoundaryCommand {
    Compile {
        #[arg(long)]
        input: String,
        #[arg(long)]
        schema: Option<String>,
        #[arg(long = "treatment-field")]
        treatment_fields: Vec<String>,
        #[arg(long)]
        hard_fail_treatment_change: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum ViewCommand {
    Query {
        #[arg(long)]
        memory_store: String,
        #[arg(long, default_value = "temporal")]
        view_mode: String,
        #[arg(long)]
        query: String,
        #[arg(long)]
        subject: Option<String>,
        #[arg(long)]
        predicate: Option<String>,
        #[arg(long)]
        valid_at: Option<DateTime<Utc>>,
        #[arg(long)]
        recorded_at: Option<DateTime<Utc>>,
        #[arg(long = "alias")]
        aliases: Vec<String>,
        #[arg(long)]
        allow_alias_expansion: bool,
        #[arg(long)]
        allow_timeless_fallback: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum SchemasCommand {
    Generate {
        #[arg(long, default_value = "schemas")]
        out: String,
    },
    Check {
        #[arg(long, default_value = "schemas")]
        root: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum CodingCommand {
    RepoRead {
        #[arg(long, default_value = ".")]
        sandbox_root: String,
        #[arg(long)]
        path: String,
    },
    RepoList {
        #[arg(long, default_value = ".")]
        sandbox_root: String,
        #[arg(long, default_value = ".")]
        path: String,
    },
    RepoSearch {
        #[arg(long, default_value = ".")]
        sandbox_root: String,
        #[arg(long)]
        query: String,
        #[arg(long, default_value = ".")]
        path: String,
    },
    PatchPropose {
        #[arg(long, default_value = ".")]
        sandbox_root: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        diff: String,
    },
    PatchApply {
        #[arg(long, default_value = ".")]
        sandbox_root: String,
        #[arg(long)]
        diff: String,
        #[arg(long)]
        permit_json: String,
    },
    RunChecks {
        #[arg(long, default_value = ".")]
        sandbox_root: String,
        #[arg(long)]
        command: String,
        #[arg(long)]
        permit_json: String,
    },
    SandboxTruth {
        #[arg(long, default_value = ".")]
        sandbox_root: String,
    },
    CodexPacket {
        #[arg(long, default_value = "P10")]
        current_pass: String,
        #[arg(long, default_value = "P11")]
        next_pass: String,
        #[arg(long)]
        issue: String,
        #[arg(long = "source")]
        source_map: Vec<String>,
        #[arg(long = "changed")]
        changed_files: Vec<String>,
        #[arg(long = "command-receipt")]
        command_receipts: Vec<String>,
        #[arg(long = "receipt-id")]
        receipt_ids: Vec<String>,
        #[arg(long = "blocker")]
        blockers: Vec<String>,
        #[arg(long = "note")]
        notes: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum AgentCommand {
    Validate {
        #[arg(long)]
        spec: String,
    },
    Doctor {
        #[arg(long)]
        spec: String,
    },
    Run {
        #[arg(long)]
        spec: String,
        #[arg(long)]
        task: String,
        #[arg(long)]
        out: String,
        #[arg(long)]
        sandbox_root: Option<String>,
        #[arg(long)]
        permit_json: Option<String>,
        #[arg(long)]
        mock_response: Option<String>,
    },
    Inspect {
        #[arg(long)]
        run: String,
    },
    New {
        #[arg(long, default_value = "local-coding")]
        template: String,
        #[arg(long)]
        out: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum DaemonCommand {
    Namespace {
        #[arg(long)]
        root: String,
        #[arg(long, default_value = "default")]
        name: String,
        #[arg(long, default_value = "daemon")]
        owner: String,
    },
    Schedule {
        #[arg(long)]
        root: String,
        #[arg(long, default_value = "default")]
        name: String,
        #[arg(long, default_value = "daemon")]
        owner: String,
        #[arg(long)]
        schedule_id: String,
        #[arg(long)]
        occurrence_key: String,
        #[arg(long)]
        due_at: DateTime<Utc>,
        #[arg(long)]
        payload: String,
        #[arg(long, default_value = "read-only")]
        risk: String,
    },
    Wake {
        #[arg(long)]
        root: String,
        #[arg(long, default_value = "default")]
        name: String,
        #[arg(long, default_value = "daemon")]
        owner: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        signal_key: String,
        #[arg(long)]
        payload: String,
        #[arg(long, default_value = "read-only")]
        risk: String,
    },
    List {
        #[arg(long)]
        root: String,
        #[arg(long, default_value = "default")]
        name: String,
        #[arg(long, default_value = "daemon")]
        owner: String,
    },
    Lease {
        #[arg(long)]
        root: String,
        #[arg(long, default_value = "default")]
        name: String,
        #[arg(long, default_value = "daemon")]
        owner: String,
        #[arg(long, default_value_t = 300)]
        ttl_seconds: i64,
    },
    Cancel {
        #[arg(long)]
        root: String,
        #[arg(long, default_value = "default")]
        name: String,
        #[arg(long, default_value = "daemon")]
        owner: String,
        #[arg(long)]
        job_id: String,
        #[arg(long, default_value = "operator-cancelled")]
        reason: String,
    },
    SafeMode {
        #[arg(long)]
        root: String,
        #[arg(long, default_value = "default")]
        name: String,
        #[arg(long, default_value = "daemon")]
        owner: String,
        #[arg(long)]
        enabled: bool,
        #[arg(long, default_value = "operator-safe-mode")]
        reason: String,
    },
    Drain {
        #[arg(long)]
        root: String,
        #[arg(long, default_value = "default")]
        name: String,
        #[arg(long, default_value = "daemon")]
        owner: String,
        #[arg(long, default_value = "operator-drain")]
        reason: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum PackageCommand {
    Examples {
        #[arg(long, default_value = ".")]
        root: String,
    },
    InstallSmoke {
        #[arg(long, default_value = ".")]
        root: String,
        #[arg(long, default_value = "examples/aidens.mock.toml")]
        config: String,
        #[arg(long)]
        include_verify: bool,
    },
    Readiness {
        #[arg(long, default_value = ".")]
        root: String,
        #[arg(long, default_value = "examples/aidens.mock.toml")]
        config: String,
        #[arg(long)]
        include_verify: bool,
    },
    CompletionAudit {
        #[arg(long, default_value = ".")]
        root: String,
        #[arg(long, default_value = "examples/aidens.mock.toml")]
        config: String,
        #[arg(long = "gate-result")]
        gate_results: Vec<String>,
    },
}

pub fn run(cli: Cli) -> Result<String> {
    match cli.command {
        Command::Profile { command } => match command {
            ProfileCommand::List => profile_list(),
            ProfileCommand::Explain { profile } => profile_explain(&profile),
        },
        Command::Plan { command } => match command {
            PlanCommand::Validate { config } => plan_validate(&config),
            PlanCommand::Compile { config, out } => plan_compile(&config, &out),
        },
        Command::Doctor { config } => doctor(config),
        Command::Status { config } => status(config),
        Command::CheckConfig { file } => check_config(file),
        Command::ListTools => list_tools(),
        Command::InspectTools { config } => inspect_tools(config),
        Command::Tools { command } => tools_command(command),
        Command::Permit { command } => permit_command(command),
        Command::Permits { command } => permit_command(command),
        Command::Receipts { command } => receipts_command(command),
        Command::Memory { command } => memory_command(command),
        Command::Boundary { command } => boundary_command(command),
        Command::View { command } => view_command(command),
        Command::Schemas { command } => schemas_command(command),
        Command::Coding { command } => coding_command(command),
        Command::Daemon { command } => daemon_command(command),
        Command::Queue { command } => daemon_command(command),
        Command::Package { command } => package_command(command),
        Command::Agent { command } => agent_command(command),
        Command::ListCapabilities => list_capabilities(),
        Command::ProviderCheck { config, file } => provider_check(config.or(file)),
        Command::Run { config, prompt } => run_once_command(config, prompt),
        Command::RunTestAgent {
            config,
            prompt,
            out,
        } => run_test_agent_command(&config, prompt, out),
        Command::RunCodingAgent {
            config,
            out,
            permit_json,
        } => run_coding_agent_command(&config, out, permit_json),
        Command::InspectRun { dir } => inspect_run_bundle_command(&dir),
        Command::New {
            profile,
            destination,
        } => new_app(&profile, &destination),
    }
}

pub fn profile_list() -> Result<String> {
    Ok(AiDENsProfile::all()
        .into_iter()
        .map(|profile| format!("{}\t{}", profile.id(), profile.product_surface_status()))
        .collect::<Vec<_>>()
        .join("\n"))
}

pub fn profile_explain(profile: &str) -> Result<String> {
    let profile = parse_profile(profile)?;
    let plan = profile.expand(profile.id())?;
    Ok(format!(
        "{}\nStatus: {} ({})\n{}\nEnabled tool bundles: {:?}\nDisabled by default: {:?}",
        plan.human_summary(),
        profile.product_surface_status(),
        profile.product_surface_note(),
        plan.risk_summary(),
        plan.enabled_tool_bundles,
        plan.disabled_tool_bundles
    ))
}

pub fn plan_validate(config: &str) -> Result<String> {
    let loaded = load_plan_config_file(config)?;
    let plan = plan_from_config(&loaded.config)?;
    plan.validate().map_err(anyhow::Error::msg)?;
    validate_plan_runtime_contract(&plan, &loaded.config)?;
    Ok(format!(
        "valid: {} profile={} provider={} source={}",
        plan.app_id, plan.profile_id, loaded.config.provider.kind, loaded.config_status
    ))
}

pub fn plan_compile(config: &str, out: &str) -> Result<String> {
    let loaded = load_plan_config_file(config)?;
    let compiled = compile_config_plan(&loaded.config_status, &loaded.config)?;
    if let Some(parent) = Path::new(out).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }
    std::fs::write(out, serde_json::to_string_pretty(&compiled)?)
        .with_context(|| format!("failed to write {out}"))?;
    Ok(format!(
        "compiled: {} -> {}",
        compiled.plan.app_id,
        Path::new(out).display()
    ))
}

pub fn doctor(config: Option<String>) -> Result<String> {
    let path = config.unwrap_or_else(|| "aidens.toml".into());
    let (config_status, cfg) = load_or_default_config(&path)?;
    ensure_profile_policy(&cfg)?;
    ensure_memory_store_policy(&cfg)?;
    let report = doctor_report_for_config(&config_status, &cfg);
    report_json_with_support_tiers(&report, support_tiers_from_doctor(&report))
}

pub fn status(config: Option<String>) -> Result<String> {
    let path = config.unwrap_or_else(|| "aidens.toml".into());
    let (config_status, cfg) = load_or_default_config(&path)?;
    ensure_profile_policy(&cfg)?;
    let doctor = doctor_report_for_config(&config_status, &cfg);
    let route = route_for_config(&cfg);
    let receipt_store_configured = receipt_store_root_for_config(&cfg, Path::new(&path)).is_some();
    let report = OperatorStatusReportV1::new(
        cfg.app_id.clone(),
        config_status,
        route.route_label,
        cfg.memory_mode.clone(),
        receipt_store_configured,
        doctor,
    );
    report_json_with_support_tiers(&report, support_tiers_from_doctor(&report.doctor))
}

pub fn check_config(file: Option<String>) -> Result<String> {
    let path = file.unwrap_or_else(|| "aidens.toml".into());
    let loaded = load_config_file(&path)?;
    ensure_profile_policy(&loaded.config)?;
    let redacted = loaded.config.redacted_json()?;
    Ok(format!(
        "AiDENs check-config: {}\n{}",
        loaded.path.display(),
        serde_json::to_string_pretty(&redacted)?
    ))
}

pub fn provider_check(config: Option<String>) -> Result<String> {
    let path = config.unwrap_or_else(|| "aidens.toml".into());
    let (config_status, cfg) = load_or_default_config(&path)?;
    ensure_profile_policy(&cfg)?;
    let spec = provider_spec_from_config(&cfg.provider);
    let readiness = provider_readiness_for_spec(&spec);
    let route = route_for_config(&cfg);
    let matrix = provider_backend_matrix();
    let backend = matrix.entry_for(&cfg.provider.kind);
    let provider_kind = backend
        .map(|entry| entry.provider_kind.clone())
        .unwrap_or_else(|| cfg.provider.kind.trim().to_ascii_lowercase());
    let backend_status = backend
        .map(|entry| entry.status.to_string())
        .unwrap_or_else(|| ProviderBackendStatusV1::Unsupported.to_string());
    let structured_output =
        backend.is_some_and(|entry| readiness.executable && entry.structured_output_executable);
    let chat_completion = readiness.executable;
    let streaming = backend.is_some_and(|entry| readiness.executable && entry.streaming_executable);
    let support_label = backend
        .map(|entry| provider_matrix_support_label(&entry.provider_kind, entry.status))
        .unwrap_or("unsupported");
    let support_tier = backend
        .map(|entry| provider_support_tier(&entry.provider_kind, entry.status))
        .unwrap_or("failed");

    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "command": "provider-check",
        "config": config_status,
        "provider": provider_kind,
        "configured_provider": cfg.provider.kind,
        "model": cfg.provider.model,
        "configured": readiness.configured,
        "executable": readiness.executable,
        "chat_completion": chat_completion,
        "route": route.route_label,
        "native_tool_loop": route.native_tool_loop,
        "structured_output": structured_output,
        "streaming": streaming,
        "degraded": route.degraded,
        "backend_status": backend_status,
        "support_label": support_label,
        "support_tier": support_tier,
        "reason_codes": merged_reason_codes(readiness.reason_codes, route.reason_codes),
    }))?)
}

pub fn list_tools() -> Result<String> {
    let registry = safe_coding_registry_for_current_dir();
    let declarations = safe_coding_tool_declarations();
    let exposure = registry.plan_exposure_with_declarations(
        &ToolExposurePolicyV1::coding_agent_default(),
        declarations.clone(),
    );
    Ok(format!(
        "AiDENs list-tools\ndeclared: {:?}\nregistered: {:?}\nexecutable: {:?}\nexposed_this_turn: {:?}\nhidden_this_turn: {:?}\nblocked_this_turn: {:?}\ndeclared_but_not_registered: {:?}\nprovider_schema_tool_ids: {:?}",
        exposure.declared_tool_ids,
        exposure.registered_tool_ids,
        exposure.executable_tool_ids,
        exposure.exposed_tool_ids,
        exposure.hidden_tool_ids,
        exposure.blocked_tool_ids,
        registry.declared_not_registered_tool_ids(&declarations),
        exposure
            .provider_tool_schemas
            .iter()
            .map(|schema| schema.tool_id.clone())
            .collect::<Vec<_>>()
    ))
}

pub fn inspect_tools(config: Option<String>) -> Result<String> {
    let (config_status, cfg) = match config {
        Some(path) => load_or_default_config(&path)?,
        None => ("safe current-directory defaults".into(), {
            let mut cfg = AiDENsConfigV1::safe_default("aidens-cli");
            cfg.tools.sandbox_root = Some(".".into());
            cfg.tools.enabled_bundles = vec!["safe-coding".into()];
            cfg
        }),
    };
    ensure_profile_policy(&cfg)?;
    let registry = tool_registry_for_config(&cfg);
    let exposure = tool_exposure_for_config(&cfg);
    let declarations = safe_coding_tool_declarations();
    let provider_schema_tool_ids = exposure
        .provider_tool_schemas
        .iter()
        .map(|schema| schema.tool_id.clone())
        .collect::<Vec<_>>();
    let requires_permit = exposure
        .decisions
        .iter()
        .filter(|decision| decision.permit_required)
        .map(|decision| decision.capability_id.clone())
        .collect::<Vec<_>>();
    let mut support_tiers = empty_support_tiers();
    let mut tool_capabilities = Vec::new();
    for decision in &exposure.decisions {
        let tool_id = &decision.capability_id;
        let support_tier = tool_support_tier(
            exposure.executable_tool_ids.contains(tool_id),
            exposure.exposed_tool_ids.contains(tool_id),
            exposure.hidden_tool_ids.contains(tool_id),
            exposure.blocked_tool_ids.contains(tool_id),
            decision.permit_required,
            registry.contains_tool_id(tool_id),
        );
        push_support_tier(&mut support_tiers, support_tier, tool_id.clone());
        tool_capabilities.push(serde_json::json!({
                "tool_id": tool_id,
                "declared": exposure.declared_tool_ids.contains(tool_id),
                "registered": exposure.registered_tool_ids.contains(tool_id),
                "executable": exposure.executable_tool_ids.contains(tool_id),
                "exposed_this_turn": exposure.exposed_tool_ids.contains(tool_id),
                "hidden_this_turn": exposure.hidden_tool_ids.contains(tool_id),
                "blocked_this_turn": exposure.blocked_tool_ids.contains(tool_id),
                "requires_permit": decision.permit_required,
                "provider_schema_tool_id": provider_schema_tool_ids.contains(tool_id),
                "support_tier": support_tier,
                "outcome": &decision.outcome,
                "reason_codes": &decision.reason_codes,
        }));
    }

    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "command": "tools inspect",
        "config": config_status,
        "support_tiers": support_tiers,
        "declared": exposure.declared_tool_ids,
        "registered": exposure.registered_tool_ids,
        "executable": exposure.executable_tool_ids,
        "exposed_this_turn": exposure.exposed_tool_ids,
        "hidden_this_turn": exposure.hidden_tool_ids,
        "blocked_this_turn": exposure.blocked_tool_ids,
        "requires_permit": requires_permit,
        "provider_schema_tool_ids": provider_schema_tool_ids,
        "declared_but_not_registered": registry.declared_not_registered_tool_ids(&declarations),
        "tool_capabilities": tool_capabilities,
        "exposure": exposure,
    }))?)
}

pub fn tools_command(command: ToolsCommand) -> Result<String> {
    match command {
        ToolsCommand::List => list_tools(),
        ToolsCommand::Inspect { config } => inspect_tools(config),
    }
}

pub fn permit_command(command: PermitCommand) -> Result<String> {
    match command {
        PermitCommand::Inspect => Ok(serde_json::to_string_pretty(&serde_json::json!({
            "ledger": "stateless-cli-command",
            "active_permits": [],
            "reason_codes": ["persisted permit-use evidence is emitted through run receipts"]
        }))?),
        PermitCommand::Request {
            tool_id,
            risk,
            sandbox_root,
        } => {
            let request = ApprovalRequestV1::scoped(
                tool_id,
                parse_risk_class(&risk)?,
                sandbox_root,
                "side-effect tool requires explicit scoped permit",
            );
            Ok(serde_json::to_string_pretty(&request)?)
        }
        PermitCommand::Approve {
            request_id,
            tool_id,
            risk,
            sandbox_root,
            decided_by,
        } => {
            let risk = parse_risk_class(&risk)?;
            let grant = PermitGrantV1::scoped(risk, tool_id, sandbox_root, decided_by.clone());
            let decision =
                ApprovalDecisionV1::approved(ArtifactId::new(request_id), grant, decided_by);
            Ok(serde_json::to_string_pretty(&decision)?)
        }
        PermitCommand::Deny {
            request_id,
            decided_by,
            reason,
        } => {
            let decision =
                ApprovalDecisionV1::denied(ArtifactId::new(request_id), decided_by, reason);
            Ok(serde_json::to_string_pretty(&decision)?)
        }
        PermitCommand::Revoke {
            permit_id,
            tool_id,
            risk,
            sandbox_root,
            reason,
        } => {
            let receipt = PermitUseReportV1::denied(
                ArtifactId::new(permit_id),
                tool_id,
                parse_risk_class(&risk)?,
                sandbox_root,
                format!("permit-revoked:{reason}"),
            );
            Ok(serde_json::to_string_pretty(&receipt)?)
        }
    }
}

pub fn receipts_command(command: EventLogCommand) -> Result<String> {
    match command {
        EventLogCommand::List { store, config } => {
            let store = CanonicalEventLog::open(receipt_store_config_from_options(store, config)?)?;
            let records = store.list_records()?;
            Ok(serde_json::to_string_pretty(&serde_json::json!({
                "store": store.config(),
                "records": records
                    .iter()
                    .map(|record| serde_json::json!({
                        "receipt_id": record.receipt_id.clone(),
                        "owner_crate": record.owner_crate.clone(),
                        "schema_name": record.schema_name.clone(),
                        "content_digest": record.content_digest.clone(),
                        "recorded_at": record.recorded_at.clone(),
                    }))
                    .collect::<Vec<_>>(),
            }))?)
        }
        EventLogCommand::Inspect {
            store,
            config,
            receipt_id,
        } => {
            let store = CanonicalEventLog::open(receipt_store_config_from_options(store, config)?)?;
            Ok(serde_json::to_string_pretty(&store.inspect(&receipt_id)?)?)
        }
        EventLogCommand::Export { store, config } => {
            let store = CanonicalEventLog::open(receipt_store_config_from_options(store, config)?)?;
            Ok(serde_json::to_string_pretty(&store.list_records()?)?)
        }
        EventLogCommand::VerifyDigest {
            store,
            config,
            receipt_id,
        } => {
            let store = CanonicalEventLog::open(receipt_store_config_from_options(store, config)?)?;
            Ok(serde_json::to_string_pretty(&serde_json::json!({
                "receipt_id": receipt_id,
                "verified": store.verify_digest(&receipt_id)?,
            }))?)
        }
    }
}

pub fn memory_command(command: MemoryCommand) -> Result<String> {
    match command {
        MemoryCommand::Status { config, store } => {
            let (config_status, cfg, config_path) = match config {
                Some(path) => {
                    let loaded = load_config_file(&path)?;
                    (
                        format!("loaded {}", loaded.path.display()),
                        loaded.config,
                        loaded.path,
                    )
                }
                None => (
                    "safe current-directory defaults".into(),
                    AiDENsConfigV1::safe_default("aidens-cli"),
                    PathBuf::from("aidens.toml"),
                ),
            };
            let store_root = store.or_else(|| memory_store_root_for_config(&cfg, &config_path));
            Ok(serde_json::to_string_pretty(&serde_json::json!({
                "kind": "memory-status",
                "config": config_status,
                "memory_mode": cfg.memory_mode,
                "store_root": store_root,
                "canonical_owner": "semantic-memory",
                "runtime_owner": "knowledge-runtime",
                "truth": memory_truth_for_config(&cfg),
            }))?)
        }
        MemoryCommand::SeamFixture { out } => memory_seam_fixture_command(out),
    }
}

pub fn memory_seam_fixture_command(out: Option<String>) -> Result<String> {
    let out_dir = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/p24/memory-seam"));
    let out_dir = resolve_output_path(out_dir)?;
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let memory_root = out_dir.join("semantic-memory");
    let namespace = "p24-memory-fixture";
    let envelope = p24_memory_fixture_envelope(namespace)?;
    let batch = aidens_memory_kit::canonical_stack::transform_forge_export(&envelope)?;

    let runtime = tokio::runtime::Runtime::new()?;
    let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(&memory_root),
        runtime_config_for_namespace(namespace),
    )?;
    let import_result = runtime.block_on(async { adapter.import_forge_export(&envelope).await })?;
    let (query_results, query_trace) =
        runtime.block_on(async { adapter.query("canonical seam fixture", None).await })?;
    if query_results.is_empty() {
        bail!("memory seam fixture imported but knowledge-runtime query returned no results");
    }

    let envelope_path = out_dir.join("export-envelope-v3.json");
    let batch_path = out_dir.join("projection-import-batch-v3.json");
    let report_path = out_dir.join("memory-runtime-seam-report.json");
    write_json_file(&envelope_path, &envelope)?;
    write_json_file(&batch_path, &batch)?;
    let grounding_evidence = aidens_memory_kit::MemoryGroundingEvidenceV1::canonical_seam(
        "canonical-seam",
        "canonical seam fixture",
        query_results.len(),
        envelope.records.len(),
        format!("{:?}", query_trace.trace_ctx),
        Vec::new(),
        Vec::new(),
    );
    let report = serde_json::json!({
        "schema": "AiDENsMemoryRuntimeSeamEvidenceV1",
        "support_tier": "supported-local-fixture",
        "semantic_status": grounding_evidence.semantic_status,
        "local_truth_store": false,
        "grounding_evidence": grounding_evidence,
        "source_envelope_id": envelope.envelope_id,
        "export_schema_version": envelope.schema_version,
        "export_content_digest": envelope.content_digest,
        "bridge_batch_schema": batch.schema_version,
        "bridge_content_digest": batch.content_digest,
        "digest_preserved": envelope.content_digest == batch.content_digest,
        "backpointers": {
            "source_envelope_id": batch.source_envelope_id,
            "trace_ctx": batch.trace_ctx,
            "execution_context": batch.execution_context,
        },
        "import_result": import_result,
        "query": {
            "runtime_owner": "knowledge-runtime",
            "memory_owner": "semantic-memory",
            "query": "canonical seam fixture",
            "result_count": query_results.len(),
            "results": query_results,
            "trace": query_trace,
            "view_disclosure": {
                "view_mode": "semantic",
                "widening": "none",
                "degradation": []
            }
        },
        "artifacts": {
            "export_envelope_v3": envelope_path,
            "projection_import_batch_v3": batch_path,
            "memory_store": memory_root,
        },
        "canonical_owners": {
            "export": "semantic-memory-forge",
            "bridge": "forge-memory-bridge",
            "storage": "semantic-memory",
            "runtime": "knowledge-runtime"
        },
        "truth_boundary": "AiDENs emits local operator seam evidence only; memory truth remains canonical-owner delegated."
    });
    write_json_file(&report_path, &report)?;

    Ok(format!(
        "AiDENs memory seam fixture\noutput: {}\nreport: {}\nenvelope: {}\nbatch: {}\nquery_results: {}",
        out_dir.display(),
        report_path.display(),
        envelope_path.display(),
        batch_path.display(),
        report["query"]["result_count"]
    ))
}

fn p24_memory_fixture_envelope(
    namespace: &str,
) -> Result<aidens_memory_kit::canonical_stack::ExportEnvelopeV3> {
    use aidens_memory_kit::canonical_stack::{
        ClaimId, ClaimVersionId, EntityId, EnvelopeId, ExportClaim, ExportEnvelopeV2, ExportRecord,
        ScopeKey, TraceCtx, EXPORT_ENVELOPE_V2_SCHEMA,
    };

    let scope_key = ScopeKey::namespace_only(namespace);
    let claim = ExportClaim {
        claim_id: Some(ClaimId::new("claim:p24-memory-fixture")),
        claim_version_id: Some(ClaimVersionId::new("claim-version:p24-memory-fixture:v1")),
        subject_entity_id: EntityId::new("entity:p24-memory-fixture"),
        predicate: "has_status".into(),
        object_anchor: serde_json::json!("canonical seam fixture imported"),
        valid_from: Some("2026-05-03T00:00:00Z".into()),
        valid_to: None,
        confidence: 1.0,
        content: "P24 canonical seam fixture imported through forge-memory-bridge into semantic-memory and queried by knowledge-runtime".into(),
        projection_family: "p24_fixture".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: Some(serde_json::json!({
            "kernel_semantics_v3": {
                "claim_family_id": "claim-family:p24-memory-fixture",
                "assertion_group_id": "assertion-group:p24-memory-fixture",
                "projection_visibility_class": "standard",
                "export_confidence_class": "verified"
            }
        })),
    };
    let records = vec![ExportRecord::Claim(claim)];
    let envelope = ExportEnvelopeV2 {
        envelope_id: EnvelopeId::new("envelope:p24-memory-fixture"),
        schema_version: EXPORT_ENVELOPE_V2_SCHEMA.into(),
        content_digest: ExportEnvelopeV2::compute_digest(
            "forge", &scope_key, &records, None, None,
        )?,
        source_authority: "forge".into(),
        scope_key,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        export_meta: None,
        evidence_bundle: None,
        records,
    };
    Ok(envelope.enrich_to_v3()?)
}

pub fn boundary_command(command: BoundaryCommand) -> Result<String> {
    match command {
        BoundaryCommand::Compile {
            input,
            schema,
            treatment_fields,
            hard_fail_treatment_change,
        } => {
            let mut request = BoundaryCompileRequestV1::new(input)
                .with_treatment_critical_fields(treatment_fields)
                .with_hard_fail_on_treatment_change(hard_fail_treatment_change);
            if let Some(schema) = schema {
                request =
                    request.with_schema(serde_json::from_str(&schema).with_context(|| {
                        "boundary compile --schema must be a JSON Schema object literal"
                    })?);
            }
            Ok(serde_json::to_string_pretty(&compile_json_boundary(
                request,
            ))?)
        }
    }
}

pub fn view_command(command: ViewCommand) -> Result<String> {
    match command {
        ViewCommand::Query {
            memory_store,
            view_mode,
            query,
            subject,
            predicate,
            valid_at,
            recorded_at,
            aliases,
            allow_alias_expansion,
            allow_timeless_fallback,
        } => {
            if subject.is_some()
                || predicate.is_some()
                || !aliases.is_empty()
                || allow_alias_expansion
                || allow_timeless_fallback
            {
                bail!(
                    "legacy AiDENs memory view filters were removed; query through the canonical semantic-memory/knowledge-runtime path"
                );
            }

            let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
                memory_config_for_root(memory_store),
                runtime_config_for_namespace("aidens"),
            )?;
            let runtime = tokio::runtime::Runtime::new()?;
            let (results, trace) = match (valid_at, recorded_at) {
                (Some(valid_at), Some(recorded_at)) => {
                    let valid_at = valid_at.to_rfc3339_opts(SecondsFormat::Secs, true);
                    let recorded_at = recorded_at.to_rfc3339_opts(SecondsFormat::Secs, true);
                    runtime.block_on(adapter.query_temporal(
                        &query,
                        None,
                        &valid_at,
                        &recorded_at,
                    ))?
                }
                (None, None) => runtime.block_on(adapter.query(&query, None))?,
                _ => bail!(
                    "canonical bitemporal view queries require both --valid-at and --recorded-at"
                ),
            };
            Ok(serde_json::to_string_pretty(&serde_json::json!({
                "kind": "canonical-runtime-view",
                "view_mode": view_mode,
                "canonical_owner": "knowledge-runtime",
                "memory_owner": "semantic-memory",
                "result_count": results.len(),
                "results": results,
                "trace": trace,
            }))?)
        }
    }
}

pub fn schemas_command(command: SchemasCommand) -> Result<String> {
    match command {
        SchemasCommand::Generate { out } => schemas_generate(&out),
        SchemasCommand::Check { root } => schemas_check(&root),
    }
}

pub fn coding_command(command: CodingCommand) -> Result<String> {
    match command {
        CodingCommand::RepoRead { sandbox_root, path } => invoke_coding_tool(
            &sandbox_root,
            PermitPolicyV1::default(),
            "aidens:repo-read:1",
            serde_json::json!({ "path": path }),
        ),
        CodingCommand::RepoList { sandbox_root, path } => invoke_coding_tool(
            &sandbox_root,
            PermitPolicyV1::default(),
            "aidens:repo-list:1",
            serde_json::json!({ "path": path }),
        ),
        CodingCommand::RepoSearch {
            sandbox_root,
            query,
            path,
        } => invoke_coding_tool(
            &sandbox_root,
            PermitPolicyV1::default(),
            "aidens:repo-search:1",
            serde_json::json!({ "query": query, "path": path }),
        ),
        CodingCommand::PatchPropose {
            sandbox_root,
            summary,
            diff,
        } => invoke_coding_tool(
            &sandbox_root,
            PermitPolicyV1::default(),
            "aidens:patch-propose:1",
            serde_json::json!({ "summary": summary, "diff": diff }),
        ),
        CodingCommand::PatchApply {
            sandbox_root,
            diff,
            permit_json,
        } => {
            let permit = permit_policy_from_json(&permit_json)?;
            invoke_coding_tool(
                &sandbox_root,
                permit,
                "aidens:patch-apply:1",
                serde_json::json!({ "diff": diff }),
            )
        }
        CodingCommand::RunChecks {
            sandbox_root,
            command,
            permit_json,
        } => {
            let permit = permit_policy_from_json(&permit_json)?;
            invoke_coding_tool(
                &sandbox_root,
                permit,
                "aidens:run-checks:1",
                serde_json::json!({ "command": command }),
            )
        }
        CodingCommand::SandboxTruth { sandbox_root } => Ok(serde_json::to_string_pretty(
            &SandboxCapabilityTruthV1::coding_default(sandbox_root),
        )?),
        CodingCommand::CodexPacket {
            current_pass,
            next_pass,
            issue,
            source_map,
            changed_files,
            command_receipts,
            receipt_ids,
            blockers,
            notes,
        } => {
            let commands_run = command_receipts
                .iter()
                .map(|receipt| command_run_receipt_from_arg(receipt))
                .collect::<Result<Vec<_>>>()?;
            let packet = CodexPacketV1::new(CodexPacketInputV1 {
                current_pass,
                next_pass,
                issue,
                source_map,
                changed_files,
                commands_run,
                receipt_ids: receipt_ids.into_iter().map(ArtifactId::new).collect(),
                blockers,
                notes,
            });
            Ok(serde_json::to_string_pretty(&packet)?)
        }
    }
}

pub fn agent_command(command: AgentCommand) -> Result<String> {
    match command {
        AgentCommand::Validate { spec } => agent_validate_command(&spec),
        AgentCommand::Doctor { spec } => agent_doctor_command(&spec),
        AgentCommand::Run {
            spec,
            task,
            out,
            sandbox_root,
            permit_json,
            mock_response,
        } => agent_run_command(&spec, &task, &out, sandbox_root, permit_json, mock_response),
        AgentCommand::Inspect { run } => inspect_run_bundle_command(&run),
        AgentCommand::New { template, out } => agent_new_command(&template, &out),
    }
}

pub fn daemon_command(command: DaemonCommand) -> Result<String> {
    match command {
        DaemonCommand::Namespace { root, name, owner } => {
            let namespace = DaemonControllerV1::namespace(&root, name, owner);
            Ok(serde_json::to_string_pretty(&namespace)?)
        }
        DaemonCommand::Schedule {
            root,
            name,
            owner,
            schedule_id,
            occurrence_key,
            due_at,
            payload,
            risk,
        } => {
            let daemon = daemon_controller(&root, &name, &owner)?;
            let outcome = daemon.enqueue_schedule_occurrence(
                schedule_id,
                occurrence_key,
                due_at,
                json_arg(&payload)?,
                parse_risk_class(&risk)?,
            )?;
            Ok(serde_json::to_string_pretty(&outcome)?)
        }
        DaemonCommand::Wake {
            root,
            name,
            owner,
            source,
            signal_key,
            payload,
            risk,
        } => {
            let daemon = daemon_controller(&root, &name, &owner)?;
            let outcome = daemon.enqueue_wake_signal(
                source,
                signal_key,
                json_arg(&payload)?,
                parse_risk_class(&risk)?,
            )?;
            Ok(serde_json::to_string_pretty(&outcome)?)
        }
        DaemonCommand::List { root, name, owner } => {
            let namespace = DaemonControllerV1::namespace(&root, name, owner);
            let daemon = DaemonControllerV1::open_read_only(&root, namespace)?;
            Ok(serde_json::to_string_pretty(&daemon.snapshot()?)?)
        }
        DaemonCommand::Lease {
            root,
            name,
            owner,
            ttl_seconds,
        } => {
            let daemon = daemon_controller(&root, &name, &owner)?;
            Ok(serde_json::to_string_pretty(
                &daemon.acquire_next(owner, ttl_seconds)?,
            )?)
        }
        DaemonCommand::Cancel {
            root,
            name,
            owner,
            job_id,
            reason,
        } => {
            let daemon = daemon_controller(&root, &name, &owner)?;
            Ok(serde_json::to_string_pretty(
                &daemon.cancel(&ArtifactId::new(job_id), reason)?,
            )?)
        }
        DaemonCommand::SafeMode {
            root,
            name,
            owner,
            enabled,
            reason,
        } => {
            let daemon = daemon_controller(&root, &name, &owner)?;
            Ok(serde_json::to_string_pretty(
                &daemon.set_safe_mode(enabled, reason)?,
            )?)
        }
        DaemonCommand::Drain {
            root,
            name,
            owner,
            reason,
        } => {
            let daemon = daemon_controller(&root, &name, &owner)?;
            Ok(serde_json::to_string_pretty(&daemon.drain(reason)?)?)
        }
    }
}

fn daemon_controller(root: &str, name: &str, owner: &str) -> Result<DaemonControllerV1> {
    let namespace = DaemonControllerV1::namespace(root, name, owner);
    Ok(DaemonControllerV1::open(
        root,
        namespace,
        owner.to_string(),
    )?)
}

fn json_arg(input: &str) -> Result<serde_json::Value> {
    serde_json::from_str(input)
        .with_context(|| "expected JSON object/array/string/number for payload argument")
}

fn invoke_coding_tool(
    sandbox_root: &str,
    permit_policy: PermitPolicyV1,
    tool_id: &str,
    input: serde_json::Value,
) -> Result<String> {
    let registry = ToolRegistryV1::safe_coding_with_dispatchers(sandbox_root)?;
    let dispatcher = ToolDispatcher::new(registry).with_permit_policy(permit_policy);
    let runtime = tokio::runtime::Runtime::new()?;
    let output = runtime.block_on(async { dispatcher.invoke(tool_id, input).await })?;
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "output": output.output,
        "tool_invocation_receipt": output.receipt,
        "permit_use_receipt": output.permit_use_receipt,
    }))?)
}

fn permit_policy_from_json(input: &str) -> Result<PermitPolicyV1> {
    let value = parse_strict_json(input).context("failed strict-parse permit JSON")?;
    if let Ok(values) = serde_json::from_value::<Vec<serde_json::Value>>(value.clone()) {
        let mut policy = PermitPolicyV1::default();
        for value in values {
            if let Ok(grant) = serde_json::from_value::<PermitGrantV1>(value.clone()) {
                policy = policy.with_grant(grant);
            } else if let Ok(decision) = serde_json::from_value::<ApprovalDecisionV1>(value) {
                if let Some(grant) = decision.permit_grant {
                    policy = policy.with_grant(grant);
                } else {
                    bail!("approval decision array contains a denial");
                }
            } else {
                bail!("--permit-json array entries must be PermitGrantV1 or approved ApprovalDecisionV1 JSON");
            }
        }
        return Ok(policy);
    }
    if let Ok(grant) = serde_json::from_value::<PermitGrantV1>(value.clone()) {
        return Ok(PermitPolicyV1::default().with_grant(grant));
    }
    if let Ok(decision) = serde_json::from_value::<ApprovalDecisionV1>(value) {
        if let Some(grant) = decision.permit_grant {
            return Ok(PermitPolicyV1::default().with_grant(grant));
        }
        bail!("approval decision does not contain an approved permit grant");
    }
    bail!("--permit-json must be PermitGrantV1 or approved ApprovalDecisionV1 JSON")
}

fn command_run_receipt_from_arg(input: &str) -> Result<CommandRunReportV1> {
    if looks_like_json(input) {
        let value =
            parse_strict_json(input).context("failed strict-parse CommandRunReportV1 JSON")?;
        return serde_json::from_value::<CommandRunReportV1>(value)
            .with_context(|| "failed to decode CommandRunReportV1 JSON");
    }
    let raw = std::fs::read_to_string(input).with_context(|| {
        format!(
            "--command-receipt must be CommandRunReportV1 JSON or a readable file path: {input}"
        )
    })?;
    let value = parse_strict_json(&raw)
        .with_context(|| format!("failed strict-parse CommandRunReportV1 from {input}"))?;
    serde_json::from_value::<CommandRunReportV1>(value)
        .with_context(|| format!("failed to parse CommandRunReportV1 from {input}"))
}

fn looks_like_json(input: &str) -> bool {
    input
        .trim_start()
        .chars()
        .next()
        .is_some_and(|ch| matches!(ch, '{' | '['))
}

pub fn schemas_generate(out: &str) -> Result<String> {
    let root = Path::new(out);
    std::fs::create_dir_all(root)
        .with_context(|| format!("failed to create schema root {}", root.display()))?;
    let documents = generated_schema_documents();
    for document in &documents {
        let path = root.join(&document.registration.schema_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create schema directory {}", parent.display())
            })?;
        }
        std::fs::write(&path, document.pretty_json())
            .with_context(|| format!("failed to write schema {}", path.display()))?;
    }
    if let Some(document) = documents
        .iter()
        .find(|document| document.registration.family == "artifact-envelope")
    {
        let compatibility_path = root.join("artifact_envelope.schema.json");
        std::fs::write(&compatibility_path, document.pretty_json()).with_context(|| {
            format!(
                "failed to write schema compatibility alias {}",
                compatibility_path.display()
            )
        })?;
    }
    let manifest_path = root.join("generated_schema_manifest_v1.json");
    std::fs::write(&manifest_path, generated_schema_manifest_pretty_json())
        .with_context(|| format!("failed to write manifest {}", manifest_path.display()))?;
    Ok(format!(
        "generated {} schema files into {}",
        documents.len(),
        root.display()
    ))
}

pub fn schemas_check(root: &str) -> Result<String> {
    let report = schema_check_report(root)?;
    let encoded = serde_json::to_string_pretty(&report)?;
    if !report.compatible {
        bail!("{encoded}");
    }
    Ok(encoded)
}

pub fn schema_check_report(root: &str) -> Result<SchemaCompatibilityReportV1> {
    let root = Path::new(root);
    let registry = current_artifact_family_registry();
    let documents = generated_schema_documents();
    let mut expected_paths = documents
        .iter()
        .map(|document| document.registration.schema_path.clone())
        .collect::<BTreeSet<_>>();
    expected_paths.insert("artifact_envelope.schema.json".into());
    let path_report = collect_schema_path_report(root)?;
    let actual_paths = path_report.paths;
    let mut checks = Vec::new();
    let mut missing_schema_paths = Vec::new();
    let mut incompatible_schema_paths = Vec::new();

    for document in &documents {
        let relative = document.registration.schema_path.clone();
        let path = root.join(&relative);
        let mut failure_reason_codes = Vec::new();
        let compatible = if !path.exists() {
            missing_schema_paths.push(relative.clone());
            failure_reason_codes.push("registered-schema-missing".into());
            false
        } else {
            let actual = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read schema {}", path.display()))?;
            match parse_strict_json(&actual) {
                Ok(actual_schema) if actual_schema == document.schema => true,
                Ok(_) => {
                    incompatible_schema_paths.push(relative.clone());
                    failure_reason_codes.push("schema-content-drift-without-major-bump".into());
                    false
                }
                Err(_) => {
                    incompatible_schema_paths.push(relative.clone());
                    failure_reason_codes.push("schema-json-invalid-or-duplicate-key".into());
                    false
                }
            }
        };
        for mode in [
            SchemaCompatibilityModeV1::Backward,
            SchemaCompatibilityModeV1::Forward,
            SchemaCompatibilityModeV1::Full,
            SchemaCompatibilityModeV1::Transitive,
        ] {
            checks.push(if compatible {
                SchemaCompatibilityCheckV1::exact(
                    document.registration.family.clone(),
                    document.registration.version,
                    mode,
                )
            } else {
                SchemaCompatibilityCheckV1::incompatible(
                    document.registration.family.clone(),
                    document.registration.version,
                    mode,
                    failure_reason_codes.clone(),
                )
            });
        }
    }
    if let Some(document) = documents
        .iter()
        .find(|document| document.registration.family == "artifact-envelope")
    {
        let relative = "artifact_envelope.schema.json".to_string();
        let path = root.join(&relative);
        if !path.exists() {
            missing_schema_paths.push(relative);
        } else {
            let actual = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read schema {}", path.display()))?;
            match parse_strict_json(&actual) {
                Ok(actual_schema) if actual_schema == document.schema => {}
                Ok(_) | Err(_) => incompatible_schema_paths.push(relative),
            }
        }
    }

    let unregistered_schema_paths = actual_paths
        .difference(&expected_paths)
        .cloned()
        .collect::<Vec<_>>();

    let manifest_path = root.join("generated_schema_manifest_v1.json");
    if !manifest_path.exists() {
        missing_schema_paths.push("generated_schema_manifest_v1.json".into());
    } else {
        let actual = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read manifest {}", manifest_path.display()))?;
        match parse_strict_json(&actual) {
            Ok(actual_manifest)
                if actual_manifest
                    == serde_json::to_value(generated_schema_manifest())
                        .unwrap_or(serde_json::Value::Null) => {}
            Ok(_) => incompatible_schema_paths.push("generated_schema_manifest_v1.json".into()),
            Err(_) => incompatible_schema_paths.push("generated_schema_manifest_v1.json".into()),
        }
    }

    Ok(SchemaCompatibilityReportV1::new(
        &registry,
        documents.len(),
        checks,
        missing_schema_paths,
        unregistered_schema_paths,
        incompatible_schema_paths,
        path_report.collision_findings,
    ))
}

struct SchemaPathCollection {
    paths: BTreeSet<String>,
    collision_findings: Vec<SchemaPathCollisionFindingV1>,
}

fn collect_schema_path_report(root: &Path) -> Result<SchemaPathCollection> {
    let mut paths = BTreeSet::new();
    if !root.exists() {
        return Ok(SchemaPathCollection {
            paths,
            collision_findings: Vec::new(),
        });
    }
    collect_schema_paths_inner(root, root, &mut paths)?;
    let mut folded = BTreeMap::<String, Vec<String>>::new();
    for path in &paths {
        folded
            .entry(path.to_ascii_lowercase())
            .or_default()
            .push(path.clone());
    }
    let collision_findings = folded
        .into_iter()
        .filter_map(|(normalized, paths)| {
            (paths.len() > 1).then(|| SchemaPathCollisionFindingV1::new(normalized, paths))
        })
        .collect();
    Ok(SchemaPathCollection {
        paths,
        collision_findings,
    })
}

fn collect_schema_paths_inner(
    root: &Path,
    current: &Path,
    paths: &mut BTreeSet<String>,
) -> Result<()> {
    for entry in std::fs::read_dir(current)
        .with_context(|| format!("failed to read schema directory {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_schema_paths_inner(root, &path, paths)?;
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".schema.json"))
        {
            paths.insert(relative_path(root, &path)?);
        }
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<String> {
    Ok(path
        .strip_prefix(root)
        .with_context(|| format!("failed to relativize {}", path.display()))?
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

pub fn list_capabilities() -> Result<String> {
    let cfg = AiDENsConfigV1::safe_default("aidens-cli");
    let report = doctor_report_for_config("safe default", &cfg);
    Ok(serde_json::to_string_pretty(&report)?)
}

pub fn run_once_command(config: Option<String>, prompt: Vec<String>) -> Result<String> {
    let prompt = prompt.join(" ");
    if prompt.trim().is_empty() {
        bail!("run requires a prompt")
    }
    let path = config.unwrap_or_else(|| "aidens.toml".into());
    let runtime = tokio::runtime::Runtime::new()?;
    let output = runtime.block_on(async {
        let app = AiDENsApp::from_config(path).build().await?;
        app.run_once(prompt).await
    })?;
    Ok(output.text)
}

pub fn run_test_agent_command(
    config: &str,
    prompt: Option<String>,
    out: Option<String>,
) -> Result<String> {
    let config_path = resolve_cli_path(config)?;
    let test_agent = load_test_agent_file(&config_path)?;
    if test_agent.provider.kind.trim() != "mock" {
        bail!("run-test-agent currently supports only mock fixture providers");
    }
    let reference_root = test_agent_reference_root(&config_path)?;
    let run_id = test_agent_run_id(&test_agent, &config_path)?;
    let out_dir = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/p23/runs").join(&run_id));
    let out_dir = resolve_output_path(out_dir)?;
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create output directory {}", out_dir.display()))?;

    let sandbox_root = test_agent
        .tools
        .sandbox_root
        .as_ref()
        .map(|root| resolve_against_root(root, &reference_root))
        .unwrap_or_else(|| out_dir.join("repo"));
    let receipt_root = test_agent
        .receipts
        .store_root
        .as_ref()
        .map(|root| resolve_against_root(root, &reference_root))
        .unwrap_or_else(|| out_dir.join("receipts"));
    std::fs::create_dir_all(&sandbox_root)
        .with_context(|| format!("failed to create sandbox root {}", sandbox_root.display()))?;
    std::fs::create_dir_all(&receipt_root)
        .with_context(|| format!("failed to create receipt root {}", receipt_root.display()))?;

    let (effective_config, fixture_path, fixture_plan) = test_agent_effective_config(
        &config_path,
        &test_agent,
        &sandbox_root,
        &receipt_root,
        true,
    )?;
    let prompt = prompt.unwrap_or_else(|| fixture_plan.user_prompt.clone());
    let effective_config_path = out_dir.join("effective-aidens.toml");
    std::fs::write(&effective_config_path, effective_config.to_toml_string()?)
        .with_context(|| format!("failed to write {}", effective_config_path.display()))?;

    let runtime = tokio::runtime::Runtime::new()?;
    let output = runtime.block_on(async {
        let app = AiDENsApp::from_config(effective_config_path.display().to_string())
            .build()
            .await?;
        app.run_once(prompt).await
    })?;
    let event_log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&receipt_root))?;
    let canonical_records = event_log.list_records()?;
    if canonical_records.is_empty() {
        bail!("run-test-agent did not persist durable receipt records");
    }
    if test_agent.agency.enabled {
        if output.agency_policy_reports.is_empty() || output.receipt.agency_receipt_ids.is_empty() {
            bail!("agency.enabled=true but runner produced no agency policy receipts");
        }
        if !canonical_records
            .iter()
            .any(|record| record.schema_name == "agency-policy-report-v1")
        {
            bail!("agency.enabled=true but canonical event log has no agency policy report");
        }
    }
    if test_agent.agency.require_receipts
        && !canonical_records
            .iter()
            .all(|record| record.verify_digest())
    {
        bail!("run-test-agent receipt digest verification failed");
    }

    let bundle = TestAgentBundlePaths::new(out_dir.clone());
    write_json_file(&bundle.run_report, &output.receipt)?;
    write_json_file(&bundle.turn_report, &output.turn_receipt)?;
    write_json_file(&bundle.tool_exposure, &output.tool_exposure)?;
    write_json_file(&bundle.agency_policy_reports, &output.agency_policy_reports)?;
    std::fs::write(&bundle.final_text, &output.text)
        .with_context(|| format!("failed to write {}", bundle.final_text.display()))?;
    write_test_agent_event_log(&bundle.event_log, &output, &canonical_records)?;
    write_test_agent_run_bundle(
        &bundle.run_bundle,
        &TestAgentRunBundleInput {
            run_id: &run_id,
            profile: effective_config
                .profile_id
                .as_deref()
                .unwrap_or("test-agent"),
            config_path: &config_path,
            fixture_path: &fixture_path,
            output_dir: &out_dir,
            receipt_root: &receipt_root,
            output: &output,
            canonical_records: &canonical_records,
        },
    )?;
    write_test_agent_summary(
        &bundle.summary,
        &TestAgentSummaryInput {
            run_id: &run_id,
            config_path: &config_path,
            fixture_path: &fixture_path,
            output_dir: &out_dir,
            sandbox_root: &sandbox_root,
            receipt_root: &receipt_root,
            seeded_files: &fixture_plan.seeded_files,
            final_text: &output.text,
            run_receipt_id: output.receipt.receipt_id.as_str(),
            turn_final_state: format!("{:?}", output.turn_receipt.final_state),
            agency_report_count: output.agency_policy_reports.len(),
            canonical_record_count: canonical_records.len(),
        },
    )?;

    Ok(format!(
        "AiDENs run-test-agent\nconfig: {}\nfixture: {}\noutput: {}\nrun_bundle: {}\nfinal: {}\nrun_report: {}\nevent_log: {}\nagency_policy_reports: {}",
        config_path.display(),
        fixture_path.display(),
        out_dir.display(),
        bundle.run_bundle.display(),
        bundle.final_text.display(),
        bundle.run_report.display(),
        bundle.event_log.display(),
        bundle.agency_policy_reports.display()
    ))
}

pub fn run_coding_agent_command(
    config: &str,
    out: Option<String>,
    permit_json: Option<String>,
) -> Result<String> {
    let config_path = resolve_cli_path(config)?;
    let loaded = load_config_file(&config_path).with_context(|| {
        format!(
            "failed to load coding-agent config {}",
            config_path.display()
        )
    })?;
    let cfg = loaded.config;
    if cfg.profile_id.as_deref() != Some("coding-agent") {
        bail!("run-coding-agent requires profile_id = \"coding-agent\"");
    }

    let config_root = config_path.parent().unwrap_or_else(|| Path::new("."));
    let sandbox_root = cfg
        .tools
        .sandbox_root
        .as_deref()
        .map(|root| resolve_against_root(root, config_root))
        .unwrap_or_else(|| config_root.to_path_buf())
        .canonicalize()
        .with_context(|| "coding-agent sandbox_root must exist")?;
    let run_id = format!("{}-coding-agent-local", receipt_store_segment(&cfg.app_id));
    let out_dir = out
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/p24/runs").join(&run_id));
    let out_dir = resolve_output_path(out_dir)?;
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create output directory {}", out_dir.display()))?;
    let receipt_root = out_dir.join("receipts");
    let canonical_log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&receipt_root))?;

    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&sandbox_root)?;
    let exposure = registry.plan_exposure(
        &ToolExposurePolicyV1::coding_agent_default()
            .with_sandbox_root(sandbox_root.display().to_string()),
    );
    let runtime = tokio::runtime::Runtime::new()?;
    let read_path = coding_agent_read_path(&sandbox_root)?;
    let search_query = coding_agent_search_query(&cfg.app_id);
    let diff = coding_agent_patch_diff(&sandbox_root, &read_path)?;
    let permit_policy = permit_json
        .as_deref()
        .map(permit_policy_from_json)
        .transpose()?
        .unwrap_or_default();
    let dispatcher = ToolDispatcher::new(registry).with_permit_policy(permit_policy);

    let mut tool_reports = Vec::new();
    let mut canonical_records = Vec::new();
    for (label, tool_id, input) in [
        (
            "repo_list",
            "aidens:repo-list:1",
            serde_json::json!({ "path": ".", "max_entries": 50 }),
        ),
        (
            "repo_read",
            "aidens:repo-read:1",
            serde_json::json!({ "path": read_path }),
        ),
        (
            "repo_search",
            "aidens:repo-search:1",
            serde_json::json!({ "query": search_query, "path": "." }),
        ),
        (
            "file_status",
            "aidens:file-stat:1",
            serde_json::json!({ "path": "." }),
        ),
        (
            "patch_propose",
            "aidens:patch-propose:1",
            serde_json::json!({
                "summary": "P24 fixture patch proposal; proposal only unless a scoped permit is supplied",
                "diff": diff.clone(),
            }),
        ),
        (
            "patch_apply_permit_gate",
            "aidens:patch-apply:1",
            serde_json::json!({ "diff": diff.clone() }),
        ),
        (
            "run_checks_permit_gate",
            "aidens:run-checks:1",
            serde_json::json!({ "command": ["cargo", "check", "--workspace"] }),
        ),
    ] {
        let report = runtime.block_on(invoke_coding_agent_step(&dispatcher, label, tool_id, input));
        if let Some(record) = append_coding_agent_step_record(&canonical_log, &report)? {
            canonical_records.push(record);
        }
        tool_reports.push(report);
    }

    let git_status = repo_status_report(&sandbox_root)?;
    canonical_records.push(canonical_log.append_orchestration_report(
        "coding-agent-status-v1",
        "coding-agent-status",
        git_status.clone(),
    )?);

    let receipt_chain = coding_agent_receipt_chain(&tool_reports);
    let loop_summary = coding_agent_loop_summary(&tool_reports);
    let semantic_status = coding_agent_semantic_status(&tool_reports);
    let v11a_evidence = coding_agent_v11a_evidence(
        &run_id,
        &config_path,
        &sandbox_root,
        &tool_reports,
        &receipt_chain,
        &git_status,
    )?;

    let report = serde_json::json!({
        "schema": "AiDENsCodingAgentLocalRunV1",
        "run_id": run_id,
        "config": config_path,
        "sandbox_root": sandbox_root,
        "tool_exposure": exposure,
        "steps": tool_reports,
        "receipt_chain": receipt_chain,
        "loop_summary": loop_summary,
        "semantic_status": semantic_status,
        "v11a_evidence": v11a_evidence,
        "status": git_status,
        "write_policy": {
            "permit_required": true,
            "permit_supplied": permit_json.is_some(),
            "unapproved_write_default": "blocked",
            "check_command_requires_permit": true
        },
        "support_tier": "supported-local",
        "canonical_backpointers": [
            CanonicalBackpointerV1::owner_type(
                "llm-tool-runtime",
                "ToolReceipt",
                "canonical-tool-receipt-owner"
            )
        ]
    });
    let bundle = TestAgentBundlePaths::new(out_dir.clone());
    write_json_file(&out_dir.join("coding-agent-report.json"), &report)?;
    write_json_file(&bundle.tool_exposure, &exposure)?;
    write_json_file(&out_dir.join("canonical-records.json"), &canonical_records)?;
    write_coding_agent_event_log(&bundle.event_log, &report)?;
    std::fs::write(
        &bundle.final_text,
        "coding-agent local lane completed with permit-gated patch/check evidence\n",
    )
    .with_context(|| format!("failed to write {}", bundle.final_text.display()))?;

    let run_bundle = build_local_run_bundle_v2(LocalRunBundleInput {
        run_id: &run_id,
        profile: "coding-agent",
        workload_class: "supported-local-coding-agent",
        provider_route: Some("local-tools-only"),
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
        replay_command: format!(
            "cargo run -p aidens-cli -- run-coding-agent {} --out {}",
            config_path.display(),
            out_dir.display()
        ),
        fixture_path: None,
        output_dir: &out_dir,
        event_log_path: &bundle.event_log,
        canonical_record_count: canonical_records.len(),
        event_count: tool_reports.len() + 1,
        elapsed_ms: 0,
        degradation: vec!["provider-route:local-tools-only".into()],
        support: AiDENsRunSupportTierEvidenceV1 {
            support_tier: "supported-local".into(),
            supported: vec![
                "repo-list".into(),
                "repo-read".into(),
                "repo-search".into(),
                "repo-status".into(),
                "patch-propose".into(),
                "permit-gated-patch-apply".into(),
                "permit-gated-run-checks".into(),
                "durable-local-receipts".into(),
            ],
            partial: vec!["provider-free-local-orchestration".into()],
            deferred: vec![
                "cloud-provider-execution".into(),
                "native-provider-tool-loop".into(),
            ],
            reason_codes: vec!["p24-supported-local-fixture-evidence".into()],
        },
        failure: coding_agent_failure_taxonomy(&report),
        output_paths: vec![
            bundle.final_text.display().to_string(),
            out_dir
                .join("coding-agent-report.json")
                .display()
                .to_string(),
            bundle.tool_exposure.display().to_string(),
            bundle.event_log.display().to_string(),
            receipt_root
                .join("canonical-receipts.ndjson")
                .display()
                .to_string(),
        ],
        provider_receipts: Vec::new(),
        tool_receipts: coding_agent_tool_receipt_ids(&report),
        permit_receipts: coding_agent_permit_receipt_ids(&report),
    })?;
    write_json_file(&bundle.run_bundle, &run_bundle)?;
    write_test_agent_summary(
        &bundle.summary,
        &TestAgentSummaryInput {
            run_id: &run_id,
            config_path: &config_path,
            fixture_path: &sandbox_root,
            output_dir: &out_dir,
            sandbox_root: &sandbox_root,
            receipt_root: &receipt_root,
            seeded_files: &[],
            final_text: "coding-agent local lane completed with permit-gated patch/check evidence",
            run_receipt_id: run_bundle.bundle_id.as_ref(),
            turn_final_state: "local-tools-complete".into(),
            agency_report_count: 0,
            canonical_record_count: canonical_records.len(),
        },
    )?;

    Ok(format!(
        "AiDENs run-coding-agent\nconfig: {}\nsandbox: {}\noutput: {}\nrun_bundle: {}\nreport: {}\nevent_log: {}",
        config_path.display(),
        sandbox_root.display(),
        out_dir.display(),
        bundle.run_bundle.display(),
        out_dir.join("coding-agent-report.json").display(),
        bundle.event_log.display(),
    ))
}

pub fn new_app(profile: &str, destination: &str) -> Result<String> {
    let profile = parse_profile(profile)?;
    let summary = scaffold_project_at(profile, Path::new(destination))?;
    Ok(format!(
        "AiDENs new\nprofile: {}\nname: {}\ncreated: {}\nfiles: {:?}",
        profile.id(),
        summary.package_name,
        summary.app_dir.display(),
        summary.files
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldSummary {
    pub app_dir: PathBuf,
    pub package_name: String,
    pub files: Vec<String>,
}

pub fn scaffold_project(
    profile: AiDENsProfile,
    name: &str,
    destination_root: &Path,
) -> Result<ScaffoldSummary> {
    let package_name = sanitize_package_name(name)?;
    scaffold_project_inner(profile, destination_root.join(&package_name), package_name)
}

pub fn scaffold_project_at(profile: AiDENsProfile, app_dir: &Path) -> Result<ScaffoldSummary> {
    let name = app_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("destination must include an app directory name"))?;
    let package_name = sanitize_package_name(name)?;
    scaffold_project_inner(profile, app_dir.to_path_buf(), package_name)
}

fn scaffold_project_inner(
    profile: AiDENsProfile,
    app_dir: PathBuf,
    package_name: String,
) -> Result<ScaffoldSummary> {
    if app_dir.exists() {
        bail!("target app directory already exists: {}", app_dir.display());
    }

    let plan = profile.expand(&package_name)?;
    plan.validate().map_err(anyhow::Error::msg)?;
    let mut cfg = AiDENsConfigV1::safe_default(&package_name);
    cfg.profile_id = Some(profile.id().into());
    cfg.memory_mode = plan.memory_mode;
    cfg.receipt_level = plan.receipt_level;
    cfg.provider = scaffold_provider_config(profile);
    cfg.receipts.store_root = Some(format!("target/aidens-receipts/{package_name}"));
    cfg.tools.sandbox_root = Some(scaffold_sandbox_root(&app_dir)?.display().to_string());
    cfg.tools.enabled_bundles = scaffold_tool_bundles(profile, &plan.enabled_tool_bundles);

    let files = vec![
        "aidens-scaffold-manifest.json".to_string(),
        "Cargo.toml".to_string(),
        "aidens.toml".to_string(),
        "README.md".to_string(),
        "AGENT.md".to_string(),
        "docs/tools.md".to_string(),
        "docs/permits.md".to_string(),
        "docs/receipts.md".to_string(),
        "src/main.rs".to_string(),
        "tests/smoke.rs".to_string(),
    ];
    let payloads = vec![
        (
            "aidens-scaffold-manifest.json",
            render_scaffold_manifest(profile, &package_name, &files)?,
        ),
        (
            "Cargo.toml",
            render_cargo_toml(&package_name, &workspace_aidens_crate_path()),
        ),
        ("aidens.toml", cfg.to_toml_string()?),
        ("README.md", render_generated_readme(&package_name)),
        ("AGENT.md", render_generated_agent_doc(&package_name)),
        ("docs/tools.md", render_tools_doc()),
        ("docs/permits.md", render_permits_doc()),
        ("docs/receipts.md", render_receipts_doc(&package_name)),
        ("src/main.rs", render_main_rs()),
        ("tests/smoke.rs", render_smoke_test(&package_name)),
    ];
    stage_scaffold_tree(&app_dir, &payloads)?;

    Ok(ScaffoldSummary {
        app_dir,
        package_name,
        files,
    })
}

fn stage_scaffold_tree(app_dir: &Path, payloads: &[(&str, String)]) -> Result<()> {
    let stage_dir = scaffold_stage_dir(app_dir)?;
    if stage_dir.exists() {
        bail!(
            "scaffold staging directory already exists: {}",
            stage_dir.display()
        );
    }
    if let Err(error) = write_staged_scaffold(&stage_dir, payloads)
        .and_then(|()| complete_staged_scaffold(&stage_dir, app_dir))
    {
        let _ = std::fs::remove_dir_all(&stage_dir);
        return Err(error);
    }
    Ok(())
}

fn write_staged_scaffold(stage_dir: &Path, payloads: &[(&str, String)]) -> Result<()> {
    for dir in ["src", "tests", "docs"] {
        std::fs::create_dir_all(stage_dir.join(dir))
            .with_context(|| format!("failed to create {}", stage_dir.join(dir).display()))?;
    }
    for (relative, contents) in payloads {
        write_scaffold_file(stage_dir, relative, contents)?;
    }
    Ok(())
}

fn complete_staged_scaffold(stage_dir: &Path, app_dir: &Path) -> Result<()> {
    if app_dir.exists() {
        bail!("target app directory already exists: {}", app_dir.display());
    }
    std::fs::rename(stage_dir, app_dir).with_context(|| {
        format!(
            "failed to atomically publish scaffold {} -> {}",
            stage_dir.display(),
            app_dir.display()
        )
    })
}

fn write_scaffold_file(stage_dir: &Path, relative: &str, contents: &str) -> Result<()> {
    if relative.contains("..") || Path::new(relative).is_absolute() {
        bail!("invalid scaffold relative path: {relative}");
    }
    let path = stage_dir.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .with_context(|| format!("failed to create-new scaffold file {}", path.display()))?;
    file.write_all(contents.as_bytes())
        .with_context(|| format!("failed to write scaffold file {}", path.display()))
}

fn scaffold_stage_dir(app_dir: &Path) -> Result<PathBuf> {
    let parent = app_dir.parent().unwrap_or_else(|| Path::new("."));
    let name = app_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("destination must include an app directory name"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    Ok(parent.join(format!(
        ".{name}.aidens-scaffold-tmp-{}-{nonce}",
        std::process::id()
    )))
}

fn scaffold_provider_config(profile: AiDENsProfile) -> ProviderConfigV1 {
    if profile == AiDENsProfile::CodingAgent {
        return ProviderConfigV1 {
            kind: "mock".into(),
            model: Some("aidens-safe-mock".into()),
            api_key: None,
            base_url: None,
            mock_response: Some(render_coding_agent_mock_response()),
        };
    }
    ProviderConfigV1 {
        kind: "disabled".into(),
        model: None,
        api_key: None,
        base_url: None,
        mock_response: None,
    }
}

fn scaffold_tool_bundles(profile: AiDENsProfile, plan_bundles: &[String]) -> Vec<String> {
    if profile == AiDENsProfile::CodingAgent {
        return [
            "repo-read",
            "repo-list",
            "file-stat",
            "repo-search",
            "patch-propose",
        ]
        .into_iter()
        .map(String::from)
        .collect();
    }
    plan_bundles.to_vec()
}

fn scaffold_sandbox_root(app_dir: &Path) -> Result<PathBuf> {
    if app_dir.is_absolute() {
        Ok(app_dir.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(app_dir))
    }
}

struct PlanConfigLoadOutcome {
    config_status: String,
    config: AiDENsConfigV1,
}

fn load_plan_config_file(path: &str) -> Result<PlanConfigLoadOutcome> {
    match load_config_file(path) {
        Ok(loaded) => Ok(PlanConfigLoadOutcome {
            config_status: format!("loaded {}", loaded.path.display()),
            config: loaded.config,
        }),
        Err(config_error) => load_test_agent_plan_config(path)
            .with_context(|| format!("failed to load {path} as AiDENs config ({config_error})")),
    }
}

fn load_test_agent_plan_config(path: &str) -> Result<PlanConfigLoadOutcome> {
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

fn load_or_default_config(path: &str) -> Result<(String, AiDENsConfigV1)> {
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

fn receipt_store_config_from_options(
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

fn receipt_store_truth_for_config(cfg: &AiDENsConfigV1) -> RuntimeCapabilityTruthV1 {
    if let Some(root) = cfg
        .receipts
        .store_root
        .clone()
        .or_else(|| default_receipt_store_root(&cfg.app_id, &cfg.receipt_level))
    {
        return truth(
            "receipts:canonical-log",
            vec![
                CapabilityStateV1::Configured,
                CapabilityStateV1::Available,
                CapabilityStateV1::Healthy,
            ],
            Some(format!(
                "append-only canonical receipt/report log; root={root}; receipt_level={}",
                cfg.receipt_level
            )),
        );
    }
    truth(
        "receipts:minimal-no-durable-store",
        vec![CapabilityStateV1::Configured, CapabilityStateV1::Disabled],
        Some("receipt_level=minimal; no durable store configured".into()),
    )
}

fn receipt_store_root_for_config(cfg: &AiDENsConfigV1, config_path: &Path) -> Option<String> {
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

fn memory_store_root_for_config(cfg: &AiDENsConfigV1, config_path: &Path) -> Option<String> {
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
        "aidens-cli".into()
    } else {
        segment
    }
}

fn plan_from_config(cfg: &AiDENsConfigV1) -> Result<AiDENsAppPlanV1> {
    let profile = profile_for_config(cfg)?;
    let mut input = ExecutionPlanAssemblyInputV1::from(profile.expand(cfg.app_id.clone())?);
    input.memory_mode = cfg.memory_mode.clone();
    input.receipt_level = cfg.receipt_level.clone();
    input.enabled_tool_bundles = cfg.tools.enabled_bundles.clone();
    Ok(assemble_execution_plan(input)
        .map_err(anyhow::Error::msg)?
        .plan)
}

fn compile_config_plan(config_status: &str, cfg: &AiDENsConfigV1) -> Result<AiDENsCompiledPlanV1> {
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

fn validate_plan_runtime_contract(plan: &AiDENsAppPlanV1, cfg: &AiDENsConfigV1) -> Result<()> {
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

fn profile_for_config(cfg: &AiDENsConfigV1) -> Result<AiDENsProfile> {
    match cfg.profile_id.as_deref() {
        Some(profile_id) => AiDENsProfile::from_id(profile_id)
            .ok_or_else(|| anyhow::anyhow!("unknown AiDENs profile: {profile_id}")),
        None => Ok(AiDENsProfile::ChatOnly),
    }
}

fn ensure_profile_policy(cfg: &AiDENsConfigV1) -> Result<()> {
    profile_for_config(cfg).map(|_| ())
}

fn ensure_memory_store_policy(cfg: &AiDENsConfigV1) -> Result<()> {
    if cfg.memory_mode == MemoryModeV1::Required && cfg.memory.store_root.is_none() {
        bail!(
            "memory-required-without-durable-store: memory_mode=required needs [memory].store_root"
        )
    }
    Ok(())
}

fn route_for_config(cfg: &AiDENsConfigV1) -> ProviderRouteReportV1 {
    let spec = provider_spec_from_config(&cfg.provider);
    route_receipt_for_spec(&spec)
}

fn is_disabled_provider(kind: &str) -> bool {
    matches!(
        kind.trim().to_ascii_lowercase().as_str(),
        "" | "disabled" | "none"
    )
}

fn provider_spec_from_config(provider: &ProviderConfigV1) -> ProviderSpecV1 {
    ProviderSpecV1 {
        kind: provider.kind.clone(),
        model: provider.model.clone(),
        api_key: provider.api_key.clone(),
        base_url: provider.base_url.clone(),
        mock_response: provider.mock_response.clone(),
    }
}

fn provider_capability_truth(cfg: &AiDENsConfigV1) -> RuntimeCapabilityTruthV1 {
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

fn provider_capability_matrix_truths() -> Vec<RuntimeCapabilityTruthV1> {
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

fn provider_matrix_support_label(
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

fn provider_support_tier(provider_kind: &str, status: ProviderBackendStatusV1) -> &'static str {
    match (provider_kind, status) {
        ("mock", ProviderBackendStatusV1::Executable) => "supported",
        ("ollama", ProviderBackendStatusV1::Executable) => "partial",
        (_, ProviderBackendStatusV1::Executable) => "partial",
        (_, ProviderBackendStatusV1::Disabled) => "deferred",
        (_, ProviderBackendStatusV1::BoundaryUnavailable) => "deferred",
        (_, ProviderBackendStatusV1::Unsupported) => "failed",
    }
}

fn semantic_disclosure_value(
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

fn report_json_with_support_tiers(
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

fn empty_support_tiers() -> BTreeMap<String, Vec<String>> {
    ["supported", "partial", "scaffold", "deferred", "failed"]
        .into_iter()
        .map(|tier| (tier.to_string(), Vec::new()))
        .collect()
}

fn push_support_tier(
    tiers: &mut BTreeMap<String, Vec<String>>,
    tier: &str,
    capability_id: impl Into<String>,
) {
    tiers
        .entry(tier.to_string())
        .or_default()
        .push(capability_id.into());
}

fn finalize_support_tiers(mut tiers: BTreeMap<String, Vec<String>>) -> serde_json::Value {
    for values in tiers.values_mut() {
        values.sort();
        values.dedup();
    }
    serde_json::to_value(tiers).unwrap_or(serde_json::Value::Null)
}

fn support_tiers_from_doctor(report: &AiDENsDoctorReportV1) -> serde_json::Value {
    let mut tiers = empty_support_tiers();
    for (section, truths) in &report.sections {
        for truth in truths {
            let tier = capability_support_tier(section, truth);
            push_support_tier(&mut tiers, tier, truth.capability_id.clone());
        }
    }
    finalize_support_tiers(tiers)
}

fn support_tiers_from_examples(manifest: &ExampleAppManifestV1) -> serde_json::Value {
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

fn support_tiers_from_surfaces(surfaces: &[ReleaseSurfaceV1]) -> serde_json::Value {
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

fn release_surface_state_support_tier(
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

fn capability_support_tier(section: &str, truth: &RuntimeCapabilityTruthV1) -> &'static str {
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

fn tool_support_tier(
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

fn provider_route_truth(cfg: &AiDENsConfigV1) -> RuntimeCapabilityTruthV1 {
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

fn memory_truth_for_config(cfg: &AiDENsConfigV1) -> RuntimeCapabilityTruthV1 {
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

fn doctor_report_for_config(config_status: &str, cfg: &AiDENsConfigV1) -> AiDENsDoctorReportV1 {
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

fn scaffold_surface_truths() -> Vec<RuntimeCapabilityTruthV1> {
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

fn tool_registry_for_config(cfg: &AiDENsConfigV1) -> ToolRegistryV1 {
    registry_from_enabled_bundles(
        &cfg.tools.enabled_bundles,
        cfg.tools.sandbox_root.as_deref(),
    )
}

fn tool_exposure_for_config(cfg: &AiDENsConfigV1) -> aidens_contracts::ToolExposureSetV1 {
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

fn plan_runtime_parity_report(
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

fn doctor_provider_route_label(doctor: &AiDENsDoctorReportV1) -> String {
    doctor
        .sections
        .get("provider_route")
        .and_then(|section| section.first())
        .and_then(|truth| truth.capability_id.strip_prefix("provider-route:"))
        .unwrap_or("<missing>")
        .to_string()
}

fn doctor_exposed_tool_ids(doctor: &AiDENsDoctorReportV1) -> Vec<String> {
    doctor
        .sections
        .get("tools")
        .into_iter()
        .flat_map(|section| section.iter())
        .filter(|truth| truth.states.contains(&CapabilityStateV1::ExposedThisTurn))
        .map(|truth| truth.capability_id.clone())
        .collect()
}

fn doctor_memory_mode(doctor: &AiDENsDoctorReportV1) -> String {
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

fn doctor_scaffold_state(doctor: &AiDENsDoctorReportV1) -> String {
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

fn sorted_join(mut values: Vec<String>) -> String {
    values.sort();
    values.join(",")
}

fn merged_reason_codes(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    left.extend(right);
    left.sort();
    left.dedup();
    left
}

#[derive(Debug, serde::Deserialize)]
struct TestAgentFileV1 {
    agent: TestAgentSectionV1,
    provider: TestAgentProviderSectionV1,
    #[serde(default)]
    tools: TestAgentToolsSectionV1,
    #[serde(default)]
    receipts: TestAgentReceiptsSectionV1,
    #[serde(default)]
    agency: TestAgentAgencySectionV1,
}

#[derive(Debug, serde::Deserialize)]
struct TestAgentSectionV1 {
    name: String,
    #[serde(default)]
    profile: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct TestAgentProviderSectionV1 {
    kind: String,
    fixture: String,
}

#[derive(Debug, Default, serde::Deserialize)]
struct TestAgentToolsSectionV1 {
    #[serde(default)]
    expose: Vec<String>,
    #[serde(default)]
    enabled_bundles: Vec<String>,
    #[serde(default)]
    sandbox_root: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct TestAgentReceiptsSectionV1 {
    #[serde(default)]
    store_root: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct TestAgentAgencySectionV1 {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    require_receipts: bool,
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
struct TestAgentFixturePlan {
    user_prompt: String,
    mock_response: String,
    requested_tool_bundles: Vec<String>,
    seeded_files: Vec<PathBuf>,
}

#[derive(Debug)]
struct TestAgentBundlePaths {
    final_text: PathBuf,
    run_bundle: PathBuf,
    run_report: PathBuf,
    turn_report: PathBuf,
    tool_exposure: PathBuf,
    agency_policy_reports: PathBuf,
    event_log: PathBuf,
    summary: PathBuf,
}

impl TestAgentBundlePaths {
    fn new(root: PathBuf) -> Self {
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

struct TestAgentRunBundleInput<'a> {
    run_id: &'a str,
    profile: &'a str,
    config_path: &'a Path,
    fixture_path: &'a Path,
    output_dir: &'a Path,
    receipt_root: &'a Path,
    output: &'a aidens_runner::AiDENsRunOutput,
    canonical_records: &'a [aidens_receipts::CanonicalEventLogEntry],
}

struct TestAgentSummaryInput<'a> {
    run_id: &'a str,
    config_path: &'a Path,
    fixture_path: &'a Path,
    output_dir: &'a Path,
    sandbox_root: &'a Path,
    receipt_root: &'a Path,
    seeded_files: &'a [PathBuf],
    final_text: &'a str,
    run_receipt_id: &'a str,
    turn_final_state: String,
    agency_report_count: usize,
    canonical_record_count: usize,
}

struct LocalRunBundleInput<'a> {
    run_id: &'a str,
    profile: &'a str,
    workload_class: &'a str,
    provider_route: Option<&'a str>,
    trace_ctx: Option<aidens_contracts::StackTraceCtx>,
    attempt_id: Option<aidens_contracts::StackAttemptId>,
    trial_id: Option<aidens_contracts::StackTrialId>,
    replay_command: String,
    fixture_path: Option<String>,
    output_dir: &'a Path,
    event_log_path: &'a Path,
    canonical_record_count: usize,
    event_count: usize,
    elapsed_ms: i64,
    degradation: Vec<String>,
    support: AiDENsRunSupportTierEvidenceV1,
    failure: AiDENsRunFailureTaxonomyV1,
    output_paths: Vec<String>,
    provider_receipts: Vec<String>,
    tool_receipts: Vec<String>,
    permit_receipts: Vec<String>,
}

async fn invoke_coding_agent_step(
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

fn coding_agent_success_step(
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

fn append_coding_agent_step_record(
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

fn coding_agent_read_path(sandbox_root: &Path) -> Result<String> {
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

fn coding_agent_search_query(app_id: &str) -> String {
    app_id
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .find(|part| part.len() >= 3)
        .unwrap_or("AiDENs")
        .to_string()
}

fn coding_agent_patch_diff(sandbox_root: &Path, read_path: &str) -> Result<String> {
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

fn repo_status_report(sandbox_root: &Path) -> Result<serde_json::Value> {
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

fn write_coding_agent_event_log(path: &Path, report: &serde_json::Value) -> Result<()> {
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

fn coding_agent_tool_receipt_ids(report: &serde_json::Value) -> Vec<String> {
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

fn coding_agent_permit_receipt_ids(report: &serde_json::Value) -> Vec<String> {
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

fn coding_agent_v11a_evidence(
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
        proof_profile.profile_id.as_str().to_string(),
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

fn coding_agent_receipt_chain(steps: &[serde_json::Value]) -> Vec<serde_json::Value> {
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

fn coding_agent_loop_summary(steps: &[serde_json::Value]) -> serde_json::Value {
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

fn coding_agent_semantic_status(steps: &[serde_json::Value]) -> &'static str {
    if steps
        .iter()
        .any(|step| step.get("status").and_then(serde_json::Value::as_str) == Some("check_failed"))
    {
        "degraded_exact_check"
    } else {
        "exact_check"
    }
}

fn coding_agent_failure_taxonomy(report: &serde_json::Value) -> AiDENsRunFailureTaxonomyV1 {
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

fn build_local_run_bundle_v2(input: LocalRunBundleInput<'_>) -> Result<AiDENsRunBundleV2> {
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

fn event_log_digest(
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

fn replay_normalized_event_log(text: &str) -> Result<String> {
    let mut values = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let mut value: serde_json::Value = serde_json::from_str(line)
            .with_context(|| "event log line must be valid JSON for replay normalization")?;
        normalize_replay_value(&mut value);
        values.push(value);
    }
    serde_json::to_string(&values).with_context(|| "failed to encode normalized event log")
}

fn normalize_replay_value(value: &mut serde_json::Value) {
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

fn load_test_agent_file(path: &Path) -> Result<TestAgentFileV1> {
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

fn load_test_agent_fixture(path: &Path) -> Result<Value> {
    serde_json::from_str(
        &std::fs::read_to_string(path)
            .with_context(|| format!("failed to read test-agent fixture {}", path.display()))?,
    )
    .with_context(|| format!("failed to parse test-agent fixture {}", path.display()))
}

fn test_agent_effective_config(
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

fn prepare_test_agent_fixture(
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

fn fixture_turn_content<'a>(turns: &'a [Value], role: &str) -> Result<&'a str> {
    turns
        .iter()
        .find(|turn| turn.get("role").and_then(Value::as_str) == Some(role))
        .and_then(|turn| turn.get("content").and_then(Value::as_str))
        .ok_or_else(|| anyhow::anyhow!("test-agent fixture missing {role} content"))
}

fn required_json_str<'a>(value: &'a Value, key: &str, label: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{label} missing or not a string"))
}

fn canonical_test_agent_tool_id(tool_name: &str) -> Result<&'static str> {
    match tool_name {
        "repo.read" => Ok("aidens:repo-read:1"),
        "repo.list" => Ok("aidens:repo-list:1"),
        "repo.search" => Ok("aidens:repo-search:1"),
        "file.stat" => Ok("aidens:file-stat:1"),
        _ => bail!("unsupported test-agent tool: {tool_name}"),
    }
}

fn test_agent_bundle_for_tool(tool_name: &str) -> Result<&'static str> {
    match tool_name {
        "repo.read" => Ok("repo-read"),
        "repo.list" => Ok("repo-list"),
        "repo.search" => Ok("repo-search"),
        "file.stat" => Ok("file-stat"),
        _ => bail!("unsupported test-agent tool: {tool_name}"),
    }
}

fn test_agent_enabled_bundles(
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

fn test_agent_profile_id(test_agent: &TestAgentFileV1, enabled_bundles: &[String]) -> String {
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

fn test_agent_run_id(test_agent: &TestAgentFileV1, config_path: &Path) -> Result<String> {
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

fn write_json_file(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn write_test_agent_run_bundle(path: &Path, input: &TestAgentRunBundleInput<'_>) -> Result<()> {
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
        provider_receipts: vec![input.output.receipt.receipt_id.as_str().to_string()],
        tool_receipts: input
            .output
            .receipt
            .tool_invocation_receipts
            .iter()
            .map(|receipt| receipt.receipt_id.as_str().to_string())
            .collect(),
        permit_receipts: input
            .output
            .receipt
            .permit_use_receipts
            .iter()
            .map(|receipt| receipt.receipt_id.as_str().to_string())
            .collect(),
    })?;
    write_json_file(path, &bundle)
}

fn test_agent_degradation_reasons(output: &aidens_runner::AiDENsRunOutput) -> Vec<String> {
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

fn write_test_agent_event_log(
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

fn write_test_agent_summary(path: &Path, input: &TestAgentSummaryInput<'_>) -> Result<()> {
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

fn resolve_cli_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()?.join(path))
}

fn resolve_output_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    resolve_cli_path(path)
}

fn resolve_against_root(path: impl AsRef<Path>, root: &Path) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn resolve_bundle_path(bundle_dir: &Path, path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        bundle_dir.join(path)
    }
}

fn test_agent_reference_root(config_path: &Path) -> Result<PathBuf> {
    let start = config_path.parent().unwrap_or_else(|| Path::new("."));
    for ancestor in start.ancestors() {
        if ancestor.join("Cargo.toml").exists() && ancestor.join("fixtures").exists() {
            return Ok(ancestor.to_path_buf());
        }
    }
    std::env::current_dir().context("failed to resolve test-agent reference root")
}

fn parse_profile(profile: &str) -> Result<AiDENsProfile> {
    AiDENsProfile::from_id(profile)
        .ok_or_else(|| anyhow::anyhow!("unknown AiDENs profile: {profile}"))
}

fn parse_risk_class(risk: &str) -> Result<CanonicalToolSideEffectClass> {
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

fn sanitize_package_name(name: &str) -> Result<String> {
    let mut package = String::new();
    let mut last_was_dash = false;
    for ch in name.trim().chars() {
        let next = if ch.is_ascii_alphanumeric() || ch == '_' {
            ch.to_ascii_lowercase()
        } else {
            '-'
        };
        if next == '-' {
            if !last_was_dash {
                package.push(next);
            }
            last_was_dash = true;
        } else {
            package.push(next);
            last_was_dash = false;
        }
    }
    let package = package.trim_matches('-').to_string();
    if package.is_empty() {
        bail!("app name must contain at least one ASCII alphanumeric character");
    }
    if package.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        return Ok(format!("aidens-{package}"));
    }
    Ok(package)
}

fn workspace_aidens_crate_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .join("crates")
        .join("aidens")
}

fn render_cargo_toml(package_name: &str, aidens_path: &Path) -> String {
    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
aidens = {{ path = "{}" }}
anyhow = "1"
tokio = {{ version = "1", features = ["rt-multi-thread", "macros"] }}
"#,
        aidens_path.display()
    )
}

fn render_main_rs() -> String {
    r#"use aidens::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = format!("{}/aidens.toml", env!("CARGO_MANIFEST_DIR"));
    let app = AiDENsApp::from_config(config).build().await?;
    let output = app.run_once("read README").await?;
    println!("{}", output.text);
    Ok(())
}
"#
    .into()
}

fn render_coding_agent_mock_response() -> String {
    format!(
        "{}{}{}",
        serde_json::json!({
            "tool_call": {
                "tool_id": "aidens:repo-read:1",
                "input": {"path": "README.md"},
            }
        }),
        TEST_AGENT_MOCK_RESPONSE_DELIMITER,
        "README evidence summary:\n{{last_tool_content}}"
    )
}

fn render_scaffold_manifest(
    profile: AiDENsProfile,
    package_name: &str,
    files: &[String],
) -> Result<String> {
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "schema": "AiDENsScaffoldManifestV1",
        "app_id": package_name,
        "profile_id": profile.id(),
        "support_tier": profile.product_surface_status(),
        "support_note": profile.product_surface_note(),
        "provider_route": if profile == AiDENsProfile::CodingAgent {
            "explicit-mock-fixture"
        } else {
            "disabled"
        },
        "receipt_store": format!("target/aidens-receipts/{package_name}"),
        "sandbox_root": ".",
        "files": files,
        "readiness_claim": "scaffold-generated; run receipts and smoke tests before claiming a completed app run",
        "canonical_owners": {
            "receipt_truth": "aidens-receipts / owner crate records",
            "tool_runtime": "llm-tool-runtime through aidens-tool-kit",
            "provider_route": "aidens-provider-kit route receipt"
        },
        "forbidden_claims": [
            "production-cloud-ready",
            "broad-autonomy-ready",
            "canonical-truth-owner",
            "v11B-complete",
            "v11C-complete"
        ],
        "reason_codes": [
            "scaffold-manifest-first",
            "explicit-mock-route",
            "receipt-store-declared",
            "side-effect-tools-not-enabled-by-default"
        ]
    }))?)
}

fn render_generated_readme(package_name: &str) -> String {
    format!(
        r#"# {package_name}

This generated AiDENs coding-agent scaffold runs with an explicit mock fixture by default.

It can read this README through the configured repository-read tool and writes receipts under `target/aidens-receipts/{package_name}`.

This is a scaffolded supported-local starting point, not a completed app-run proof. Inspect `aidens-scaffold-manifest.json`, run the generated smoke test, and inspect canonical receipts before making any completion claim.

Default safety posture:

- provider: explicit mock fixture, no cloud API key
- tools: read/list/search/stat plus patch proposal only
- writes/admin commands: not enabled by default
- receipts: full receipt level
- network policy: disabled

Try:

```bash
cargo run -p aidens-cli -- run --config path/to/{package_name}/aidens.toml "read README"
```
"#
    )
}

fn render_generated_agent_doc(package_name: &str) -> String {
    format!(
        r#"# Agent Operator Notes

Agent: `{package_name}`

This project is a runnable AiDENs coding-agent scaffold. It uses AiDENs as the orchestration/profile/policy/product layer and delegates execution evidence to the configured runner and receipt log.

Do not add cloud provider support by editing prompt text. Provider support must be implemented and tested before it can be marked executable.

Do not claim this scaffold is a completed app run until material operation receipts exist and the generated smoke test passes.

The generated default is intentionally safe:

- explicit mock provider fixture only
- repository sandbox scoped to this project directory
- no write/admin tool exposure by default
- receipt log enabled
"#
    )
}

fn render_tools_doc() -> String {
    r#"# Tools

Default exposed tool bundles:

- `repo-read`
- `repo-list`
- `file-stat`
- `repo-search`
- `patch-propose`

The generated config does not enable `patch-apply`, `run-checks`, shell, network, or admin tools. Add side-effect tools only with scoped permits and tests.
"#
    .into()
}

fn render_permits_doc() -> String {
    r#"# Permits

Read-only inspection tools do not need permits.

Any file write, patch apply, shell command, network operation, queue/admin action, or other side-effect tool must be permit-gated with an explicit scope. Do not grant broad write or admin access by default.
"#
    .into()
}

fn render_receipts_doc(package_name: &str) -> String {
    format!(
        r#"# Receipts

This scaffold uses full receipt level.

Default receipt store:

```text
target/aidens-receipts/{package_name}
```

Inspect receipts from the AiDENs workspace:

```bash
cargo run -p aidens-cli -- receipts list --config path/to/{package_name}/aidens.toml
```

The run report records provider route, tool exposure, tool calls, stop rules, agency policy report IDs, degraded paths, and failures.

User-visible completion depends on durable receipt records. A final string alone is not evidence of completion.
"#
    )
}

fn render_smoke_test(app_id: &str) -> String {
    format!(
        r#"use aidens::prelude::*;

#[test]
fn generated_plan_is_safe_and_visible() {{
    let plan = AiDENsProfile::CodingAgent
        .expand("{app_id}")
        .expect("generated profile expands");

    assert!(!plan.dangerous_auto_approval);
    assert!(plan.validate().is_ok());
    assert!(plan.risk_summary().contains("permit_required=true"));
}}

#[test]
fn generated_config_is_receipt_first_and_secret_free() {{
    let manifest = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("aidens-scaffold-manifest.json"),
    )
    .expect("generated scaffold manifest is readable");
    assert!(manifest.contains("\"schema\": \"AiDENsScaffoldManifestV1\""));
    assert!(manifest.contains("\"receipt_store\""));
    assert!(manifest.contains("\"explicit-mock-route\""));

    let config = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("aidens.toml"),
    )
    .expect("generated scaffold config is readable");
    assert!(config.contains("kind = \"mock\""));
    assert!(config.contains("receipt_level = \"full\""));
    assert!(config.contains("store_root = \"target/aidens-receipts/{app_id}\""));
    assert!(!config.contains("api_key"));
    assert!(!config.contains("secret"));
    assert!(!config.contains("token"));
    assert!(!config.contains("patch-apply"));
    assert!(!config.contains("run-checks"));
}}
"#
    )
}

#[cfg(test)]
mod tests;
