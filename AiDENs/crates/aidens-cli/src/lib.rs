pub(crate) use aidens_app_kit::{AiDENsApp, AiDENsProfile};
pub(crate) use aidens_boundary_kit::{
    compile_json_boundary, parse_strict_json, validate_json_schema,
};
pub(crate) use aidens_capability_kit::truth;
pub(crate) use aidens_config::{load_config_file, AiDENsConfigV1, ProviderConfigV1};
pub(crate) use aidens_contracts::{
    current_artifact_family_registry, generated_artifact_id_from_material,
    generated_schema_documents, generated_schema_manifest, generated_schema_manifest_pretty_json,
    non_authoritative_text_display_digest, AgentPermitRuleV1, AgentSpecV1, AiDENsAppPlanV1,
    AiDENsCompiledPlanV1, AiDENsDoctorReportV1, AiDENsRunBudgetDeadlineV1, AiDENsRunBundleV2,
    AiDENsRunBundleV3, AiDENsRunEventLogDigestV1, AiDENsRunFailureClassV1,
    AiDENsRunFailureTaxonomyV1, AiDENsRunReplayNormalizationV1, AiDENsRunSupportTierEvidenceV1,
    ApprovalDecisionV1, ArtifactId, BoundaryCompileRequestV1, CanonicalBackpointerV1,
    CanonicalToolSideEffectClass, CapabilityStateV1, CodexPacketInputV1, CodexPacketV1,
    CommandRunReportV1, CompletionAuditReportV1, ConfigApplyReportDraftV1, ConfigApplyReportV1,
    CrossPassTraceabilityMatrixV1, CrossPassTraceabilityRowV1, DisplayDigestV1, ExampleAppEntryV1,
    ExampleAppManifestV1, GateCommandResultV1, InstallSmokeReportV1, InstallSmokeStepV1,
    KnownLimitationV1, KnownLimitationsRegisterV1, MemoryModeV1, OperatorStatusReportV1,
    PassCompletionStateV1, PermitGrantV1, PermitUseReportV1, PlanRuntimeParityCheckKindV1,
    PlanRuntimeParityCheckV1, PlanRuntimeParityReportV1, ProviderBackendStatusV1,
    ProviderRouteKindV1, ProviderRouteReportV1, PublicDocFindingV1, RegressionDebtItemV1,
    RegressionDebtLedgerV1, ReleaseArtifactEntryV1, ReleaseArtifactKindV1,
    ReleaseArtifactManifestV1, ReleaseReadinessReportV1, ReleaseSurfaceStateV1, ReleaseSurfaceV1,
    ReportLevelV1, RuntimeCapabilityTruthV1, SandboxCapabilityTruthV1, SchemaCompatibilityCheckV1,
    SchemaCompatibilityModeV1, SchemaCompatibilityReportV1, SchemaPathCollisionFindingV1,
    StackAttemptId, StackContentDigest, StackTrialId, ToolExposureSetV1,
};
pub(crate) use aidens_daemon_kit::DaemonControllerV1;
pub(crate) use aidens_memory_kit::{
    memory_config_for_root, runtime_config_for_namespace, CanonicalMemoryAdapter,
};
pub(crate) use aidens_permit_kit::{HostPermitAuthorityV1, PermitCheckContextV1, PermitPolicyV1};
pub(crate) use aidens_plan_kit::{assemble_execution_plan, ExecutionPlanAssemblyInputV1};
pub(crate) use aidens_provider_kit::{
    provider_backend_matrix, provider_readiness_for_spec, route_receipt_for_spec, ProviderSpecV1,
};
pub(crate) use aidens_receipts::{
    CanonicalEventLog, CanonicalEventLogConfig, RunBundleStore, RunBundleStoreConfig,
};
pub(crate) use aidens_runner::{
    PlanActVerifyLoopV1, PlanActVerifyLoopV1Output, PlanActVerifyOutcomeV1,
};
pub(crate) use aidens_tool_kit::{
    registry_from_enabled_bundles, safe_coding_registry_for_current_dir,
    safe_coding_tool_declarations, ToolDispatcher, ToolExposurePolicyV1, ToolInvocationError,
    ToolInvocationOutcome, ToolRegistryV1,
};
pub(crate) use anyhow::{bail, Context, Result};
pub(crate) use chrono::{DateTime, SecondsFormat, Utc};
pub(crate) use clap::{Parser, Subcommand};
pub(crate) use serde_json::Value;
pub(crate) use std::collections::{BTreeMap, BTreeSet};
pub(crate) use std::io::Write;
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::time::{SystemTime, UNIX_EPOCH};

mod agent;
mod cfg;
mod doctor;
mod package;
mod scaffold;
mod schemas;
mod test_agent;

pub use agent::*;
pub use package::*;

pub(crate) use cfg::*;
pub(crate) use doctor::*;
pub(crate) use scaffold::*;
pub(crate) use schemas::*;
pub(crate) use test_agent::*;

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
    Verify {
        #[arg(long)]
        config: Option<String>,
        #[arg(long, default_value = ".")]
        root: String,
    },
    InspectReceipt {
        receipt_id: String,
        #[arg(long)]
        store: Option<String>,
        #[arg(long)]
        config: Option<String>,
    },
    New {
        profile: String,
        destination: String,
    },
    /// Run the autonomous gap-detection → task-generation → execution → capture → evaluation loop.
    Autonomous {
        #[arg(long, default_value = "~/.hermes/semantic-memory.db")]
        memory_dir: String,
        #[arg(long, default_value = "/tmp/aidens-queue")]
        queue_dir: String,
        #[arg(long, default_value = "http://127.0.0.1:11434")]
        model_url: String,
        #[arg(long, default_value = "granite4.1:3b")]
        chosen_model: String,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long, default_value = "http://127.0.0.1:1738")]
        http_base_url: String,
        #[arg(long, default_value_t = 0)]
        max_iterations: usize,
        #[arg(long, default_value_t = 60)]
        sleep_ms: u64,
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
        #[arg(long)]
        run_id: String,
        #[arg(long)]
        attempt_id: String,
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
        #[arg(long)]
        run_id: String,
        #[arg(long)]
        attempt_id: String,
        #[arg(long, default_value_t = 900)]
        expires_in_seconds: i64,
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
        #[arg(long, default_value = "operator")]
        revoked_by: String,
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
        Command::Verify { config, root } => verify_command(config, &root),
        Command::InspectReceipt {
            receipt_id,
            store,
            config,
        } => inspect_receipt_command(&receipt_id, store, config),
        Command::New {
            profile,
            destination,
        } => new_app(&profile, &destination),
        Command::Autonomous {
            memory_dir,
            queue_dir,
            model_url,
            chosen_model,
            api_key,
            http_base_url,
            max_iterations,
            sleep_ms,
        } => {
            let config = aidens_autonomous::LoopConfig {
                memory_dir: PathBuf::from(memory_dir),
                queue_dir: PathBuf::from(queue_dir),
                model_url,
                chosen_model,
                api_key,
                http_base_url,
                max_iterations,
                sleep_between_iterations_ms: sleep_ms,
                ..Default::default()
            };
            let loop_driver = aidens_autonomous::AutonomousLoop::from_config(config)?;
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(async {
                // Spawn a stderr state reporter.
                let state = loop_driver.state.clone();
                let reporter = tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        let snap = state.lock().map(|s| s.clone()).unwrap_or_default();
                        eprintln!(
                            "[autonomous] iter={} gaps={} tasks_gen={} tasks_done={} tasks_fail={} facts_cap={} facts_rej={} safe={} job={:?} err={:?}",
                            snap.iteration,
                            snap.gaps_detected,
                            snap.tasks_generated,
                            snap.tasks_completed,
                            snap.tasks_failed,
                            snap.facts_captured,
                            snap.facts_rejected,
                            snap.safe_mode,
                            snap.current_job,
                            snap.last_error,
                        );
                    }
                });
                let result = loop_driver.run().await;
                reporter.abort();
                result
            })?;
            Ok("autonomous loop completed".to_string())
        }
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

fn approval_request_id_for_context(context: &PermitCheckContextV1) -> Result<ArtifactId> {
    let material = serde_json::to_string(&(
        "aidens.approval-request-context.v1",
        context.tool_id.as_str(),
        &context.risk_class,
        context.sandbox_root.as_str(),
        context.run_id.as_ref(),
        context.attempt_id.as_ref(),
    ))
    .context("serialize canonical approval request context")?;
    Ok(generated_artifact_id_from_material(
        "approval-request",
        &material,
    ))
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
            run_id,
            attempt_id,
        } => {
            let context =
                PermitCheckContextV1::new(tool_id, parse_risk_class(&risk)?, sandbox_root)
                    .with_run_attempt(Some(ArtifactId(run_id)), Some(ArtifactId(attempt_id)));
            let mut request = PermitPolicyV1::default()
                .approval_request_for_context(&context)
                .context("side-effect request unexpectedly required no approval")?;
            request.request_id = approval_request_id_for_context(&context)?;
            Ok(serde_json::to_string_pretty(&request)?)
        }
        PermitCommand::Approve {
            request_id,
            tool_id,
            risk,
            sandbox_root,
            decided_by,
            run_id,
            attempt_id,
            expires_in_seconds,
        } => {
            let risk = parse_risk_class(&risk)?;
            let context = PermitCheckContextV1::new(tool_id, risk, sandbox_root)
                .with_run_attempt(Some(ArtifactId(run_id)), Some(ArtifactId(attempt_id)));
            let request_id = ArtifactId(request_id);
            let expected_request_id = approval_request_id_for_context(&context)?;
            if request_id != expected_request_id {
                bail!("approval request context mismatch");
            }
            let authority =
                HostPermitAuthorityV1::load().context("load host permit authority for approval")?;
            let grant = authority
                .issue_expiring_for_context(
                    &context,
                    decided_by.clone(),
                    chrono::Duration::seconds(expires_in_seconds),
                )
                .context("issue expiring host-trusted permit")?;
            let decision = ApprovalDecisionV1::approved(expected_request_id, grant, decided_by);
            Ok(serde_json::to_string_pretty(&decision)?)
        }
        PermitCommand::Deny {
            request_id,
            decided_by,
            reason,
        } => {
            let decision = ApprovalDecisionV1::denied(ArtifactId(request_id), decided_by, reason);
            Ok(serde_json::to_string_pretty(&decision)?)
        }
        PermitCommand::Revoke {
            permit_id,
            tool_id,
            risk,
            sandbox_root,
            reason,
            revoked_by,
        } => {
            let authority = HostPermitAuthorityV1::load()
                .context("load host permit authority for revocation")?;
            authority
                .revoke(&ArtifactId(permit_id.clone()), revoked_by, reason.clone())
                .context("persist permit revocation")?;
            let receipt = PermitUseReportV1::denied(
                ArtifactId(permit_id),
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
                receipt_ids: receipt_ids.into_iter().map(ArtifactId).collect(),
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
                &daemon.cancel(&ArtifactId(job_id), reason)?,
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
    let authority_context = permit_policy.bound_context_for_tool(tool_id);
    let dispatcher = ToolDispatcher::new(registry).with_permit_policy(permit_policy);
    let runtime = tokio::runtime::Runtime::new()?;
    let output = runtime.block_on(async {
        let (run_id, attempt_id) = authority_context
            .map(|(run_id, attempt_id)| (Some(run_id), Some(attempt_id)))
            .unwrap_or((None, None));
        dispatcher
            .invoke_with_context(tool_id, input, run_id, attempt_id)
            .await
    })?;
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "output": output.output,
        "tool_invocation_receipt": output.receipt,
        "permit_use_receipt": output.permit_use_receipt,
    }))?)
}

fn permit_policy_from_json(input: &str) -> Result<PermitPolicyV1> {
    let value = parse_strict_json(input).context("failed strict-parse permit JSON")?;
    if let Ok(values) = serde_json::from_value::<Vec<serde_json::Value>>(value.clone()) {
        let mut policy = HostPermitAuthorityV1::load()
            .context("load host permit authority for permit verification")?
            .policy();
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
        return Ok(HostPermitAuthorityV1::load()
            .context("load host permit authority for permit verification")?
            .policy()
            .with_grant(grant));
    }
    if let Ok(decision) = serde_json::from_value::<ApprovalDecisionV1>(value) {
        if let Some(grant) = decision.permit_grant {
            return Ok(HostPermitAuthorityV1::load()
                .context("load host permit authority for permit verification")?
                .policy()
                .with_grant(grant));
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

pub fn verify_command(config: Option<String>, root: &str) -> Result<String> {
    let path = config.unwrap_or_else(|| "aidens.toml".into());
    let (config_status, cfg) = load_or_default_config(&path)?;
    ensure_profile_policy(&cfg)?;
    let doctor = doctor_report_for_config(&config_status, &cfg);
    let route = route_for_config(&cfg);
    let receipt_store_configured =
        receipt_store_root_for_config(&cfg, std::path::Path::new(&path)).is_some();
    let healthy_count = doctor
        .sections
        .values()
        .flat_map(|section| section.iter())
        .filter(|truth| truth.states.contains(&CapabilityStateV1::Healthy))
        .count();
    let total_count = doctor
        .sections
        .values()
        .flat_map(|section| section.iter())
        .count();
    let all_healthy = healthy_count == total_count;
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "command": "verify",
        "config": config_status,
        "root": root,
        "provider_route": route.route_label,
        "provider_executable": !route.degraded,
        "receipt_store_configured": receipt_store_configured,
        "memory_mode": cfg.memory_mode,
        "gate_summary": {
            "healthy": healthy_count,
            "total": total_count,
            "all_healthy": all_healthy,
        },
        "verdict": if all_healthy { "gates-pass" } else { "gates-fail" },
        "doctor": doctor,
    }))?)
}

pub fn inspect_receipt_command(
    receipt_id: &str,
    store: Option<String>,
    config: Option<String>,
) -> Result<String> {
    let store_config = receipt_store_config_from_options(store, config)?;
    let store = CanonicalEventLog::open(store_config)?;
    Ok(serde_json::to_string_pretty(&store.inspect(receipt_id)?)?)
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

#[cfg(test)]
mod tests;
