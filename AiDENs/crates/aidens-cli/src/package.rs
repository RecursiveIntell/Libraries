use super::*;

use std::io::Read;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub fn package_command(command: PackageCommand) -> Result<String> {
    match command {
        PackageCommand::Examples { root } => {
            let manifest = example_app_manifest(Path::new(&root))?;
            report_json_with_support_tiers(&manifest, support_tiers_from_examples(&manifest))
        }
        PackageCommand::InstallSmoke {
            root,
            config,
            include_verify,
        } => {
            let receipt = install_smoke_receipt(Path::new(&root), &config, include_verify)?;
            Ok(serde_json::to_string_pretty(&receipt)?)
        }
        PackageCommand::Readiness {
            root,
            config,
            include_verify,
        } => {
            let report = release_readiness_report(Path::new(&root), &config, include_verify)?;
            let encoded = report_json_with_support_tiers(
                &report,
                support_tiers_from_surfaces(&report.surfaces),
            )?;
            if report.blocks_release() {
                bail!("{encoded}");
            }
            Ok(encoded)
        }
        PackageCommand::CompletionAudit {
            root,
            config,
            gate_results,
        } => {
            let report = completion_audit_report(Path::new(&root), &config, gate_results)?;
            Ok(serde_json::to_string_pretty(&report)?)
        }
    }
}

pub fn release_readiness_report(
    root: &Path,
    config: &str,
    include_verify: bool,
) -> Result<ReleaseReadinessReportV1> {
    let manifest = example_app_manifest(root)?;
    let smoke = install_smoke_receipt(root, config, include_verify)?;
    let findings = scan_public_docs_for_scaffold_claims(root)?;
    Ok(ReleaseReadinessReportV1::new(
        release_surfaces(),
        findings,
        manifest,
        smoke,
    ))
}

pub(crate) const P24_REQUIRED_GATE_COMMANDS: &[&str] = &[
    "cargo fmt --all --check",
    "cargo check --workspace --all-targets --all-features",
    "cargo test --workspace --all-targets --all-features",
    "cargo clippy --workspace --all-targets --all-features -- -D warnings",
    "bash scripts/p24_verify.sh",
];

const SOURCE_BASIS_HEADER_PREFIX: &str = "# Source Basis - ";

pub fn completion_audit_report(
    root: &Path,
    config: &str,
    gate_results: Vec<String>,
) -> Result<CompletionAuditReportV1> {
    let release_readiness = release_readiness_report(root, config, false)?;
    let source_basis = workspace_source_basis_label(root);
    let traceability_matrix = cross_pass_traceability_matrix(root)?;
    let release_artifact_manifest = release_artifact_manifest(root)?;
    let known_limitations = known_limitations_register();
    let regression_debt = regression_debt_ledger();
    Ok(CompletionAuditReportV1::new(
        source_basis,
        parse_gate_results(gate_results)?,
        release_readiness,
        traceability_matrix,
        release_artifact_manifest,
        known_limitations,
        regression_debt,
    ))
}

fn parse_gate_results(gate_results: Vec<String>) -> Result<Vec<GateCommandResultV1>> {
    let mut parsed = BTreeMap::new();
    for raw in gate_results {
        let (command, status) = raw
            .rsplit_once('=')
            .or_else(|| raw.split_once('|'))
            .with_context(|| {
                format!("gate result must be '<command>=passed|failed|waived|not-run': {raw}")
            })?;
        let command = command.trim();
        let status = status.trim();
        let result = match status {
            "passed" => GateCommandResultV1::passed(command, "cli:gate-result"),
            "failed" => GateCommandResultV1::failed(
                command,
                1,
                "external gate result reported failure",
                "gate-command-failed",
            ),
            "waived" => GateCommandResultV1::waived(command, "governance-waiver:external"),
            "not-run" => GateCommandResultV1::not_run(command, "gate-command-not-run"),
            _ => bail!("unknown gate result status for {command}: {status}"),
        };
        parsed.insert(command.to_string(), result);
    }
    let mut results = Vec::new();
    for command in P24_REQUIRED_GATE_COMMANDS {
        results.push(
            parsed.remove(*command).unwrap_or_else(|| {
                GateCommandResultV1::not_run(*command, "gate-result-not-supplied")
            }),
        );
    }
    results.extend(parsed.into_values());
    Ok(results)
}

fn release_artifact_manifest(root: &Path) -> Result<ReleaseArtifactManifestV1> {
    let required = [
        ("AGENTS.md", ReleaseArtifactKindV1::Document),
        ("SOURCE_BASIS.md", ReleaseArtifactKindV1::Document),
        ("BUILD_ORDER_DAG.md", ReleaseArtifactKindV1::Document),
        ("README.md", ReleaseArtifactKindV1::Document),
        ("STATUS.md", ReleaseArtifactKindV1::Document),
        (
            "ARTIFACT_SCHEMA_REGISTRY.md",
            ReleaseArtifactKindV1::Document,
        ),
        ("pass_manifest.json", ReleaseArtifactKindV1::Manifest),
        ("scripts/verify.sh", ReleaseArtifactKindV1::Script),
        (
            "scripts/assert_no_fake_completion.sh",
            ReleaseArtifactKindV1::Script,
        ),
        (
            "scripts/assert_no_scaffold_promoted.sh",
            ReleaseArtifactKindV1::Script,
        ),
        (".github/workflows/ci.yml", ReleaseArtifactKindV1::Ci),
        (
            "schemas/generated_schema_manifest_v1.json",
            ReleaseArtifactKindV1::Manifest,
        ),
        ("schemas", ReleaseArtifactKindV1::Schema),
        ("tests/fixtures", ReleaseArtifactKindV1::Fixture),
        ("examples", ReleaseArtifactKindV1::Example),
        ("passes", ReleaseArtifactKindV1::Document),
        ("docs", ReleaseArtifactKindV1::Document),
        ("handoffs", ReleaseArtifactKindV1::Handoff),
    ];
    let mut artifacts = Vec::new();
    for (path, kind) in required {
        artifacts.push(release_artifact_entry(root, path, kind, true)?);
    }
    Ok(ReleaseArtifactManifestV1::new(
        "aidens-p19-release",
        workspace_source_basis_label(root),
        artifacts,
    ))
}

pub(crate) fn workspace_source_basis_label(root: &Path) -> String {
    let run_basis = source_basis_from_markdown(&root.join("SOURCE_BASIS.md"))
        .or_else(|| {
            current_run_from_markdown(&root.join("docs").join("codex-runs").join("CURRENT_RUN.md"))
        })
        .unwrap_or_else(|| "current-workspace".to_string());
    format!("aidens-{run_basis}-current-workspace")
}

fn source_basis_from_markdown(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with(SOURCE_BASIS_HEADER_PREFIX) {
            let raw = line.trim_start_matches(SOURCE_BASIS_HEADER_PREFIX);
            return Some(normalize_basis_token(raw));
        }
    }
    None
}

fn current_run_from_markdown(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if !line.to_lowercase().starts_with("current run:") {
            continue;
        }
        if let Some(run) = line.split_once('`').and_then(|(_, rest)| {
            let (run, _) = rest.split_once('`')?;
            Some(run)
        }) {
            return Some(normalize_basis_token(run));
        }
    }
    None
}

fn normalize_basis_token(raw: &str) -> String {
    raw.chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .collect::<String>()
        .to_lowercase()
}

fn release_artifact_entry(
    root: &Path,
    relative: &str,
    kind: ReleaseArtifactKindV1,
    required: bool,
) -> Result<ReleaseArtifactEntryV1> {
    let path = root.join(relative);
    if !path.exists() {
        return Ok(ReleaseArtifactEntryV1::missing(relative, kind, required));
    }
    let (digest, byte_len) = if path.is_dir() {
        digest_directory(root, &path)?
    } else {
        let bytes =
            std::fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        (
            non_authoritative_text_display_digest(&String::from_utf8_lossy(&bytes)),
            bytes.len() as u64,
        )
    };
    Ok(ReleaseArtifactEntryV1::present(
        relative, kind, required, digest, byte_len,
    ))
}

fn digest_directory(root: &Path, dir: &Path) -> Result<(String, u64)> {
    let mut files = Vec::new();
    collect_files_recursive(dir, &mut files)?;
    files.sort();
    let mut material = String::new();
    let mut bytes_total = 0_u64;
    for path in files {
        let bytes =
            std::fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        bytes_total += bytes.len() as u64;
        let relative = relative_path(root, &path)?;
        material.push_str(&relative);
        material.push('\0');
        material.push_str(&non_authoritative_text_display_digest(
            &String::from_utf8_lossy(&bytes),
        ));
        material.push('\n');
    }
    Ok((
        non_authoritative_text_display_digest(&material),
        bytes_total,
    ))
}

fn collect_files_recursive(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in
        std::fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect file type for {}", path.display()))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_files_recursive(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

fn cross_pass_traceability_matrix(root: &Path) -> Result<CrossPassTraceabilityMatrixV1> {
    let tasks_root = root.join("tasks");
    let mut rows = Vec::new();
    for task_path in sorted_json_files(&tasks_root)? {
        let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&task_path)?)?;
        let Some(pass_id) = value.get("pass_id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        if !is_p19_release_pass_id(pass_id) {
            continue;
        }
        let title = value
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("pass acceptance");
        let crates = string_array(&value, "crates");
        let artifacts = string_array(&value, "artifacts");
        let acceptance = string_array(&value, "acceptance");
        let pass_doc = value
            .get("pass_doc")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("passes")
            .to_string();
        let prompt_file = value
            .get("prompt_file")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("prompts")
            .to_string();
        let handoff = handoff_for_pass(root, pass_id);
        let mut docs = vec![pass_doc, prompt_file, "STATUS.md".into()];
        let mut evidence_refs = vec!["STATUS.md".into()];
        if let Some(handoff) = handoff {
            docs.push(handoff.clone());
            evidence_refs.push(handoff);
        }
        let state = if pass_id == "P19"
            || evidence_refs
                .iter()
                .any(|path| path.starts_with("handoffs/"))
        {
            PassCompletionStateV1::Done
        } else {
            PassCompletionStateV1::Deferred
        };
        rows.push(
            CrossPassTraceabilityRowV1::new(format!("{pass_id}:acceptance"), pass_id, title, state)
                .with_crates(crates)
                .with_artifacts(artifacts)
                .with_tests(vec![
                    "cargo test --workspace".into(),
                    "bash scripts/verify.sh".into(),
                ])
                .with_docs(docs)
                .with_acceptance_gates(acceptance)
                .with_evidence(evidence_refs),
        );
    }
    for run_dir in ["p27", "p24", "p23", "p22"] {
        if rows.is_empty() {
            rows.extend(handoff_traceability_rows(root, run_dir)?);
        }
    }
    Ok(CrossPassTraceabilityMatrixV1::new(rows))
}

fn handoff_traceability_rows(
    root: &Path,
    run_dir: &str,
) -> Result<Vec<CrossPassTraceabilityRowV1>> {
    let reports_root = root.join("handoffs").join(run_dir);
    let run_id = run_dir.to_ascii_uppercase();
    let verifier_command = format!("bash scripts/{}_verify.sh", run_dir);
    let mut rows = Vec::new();
    for report_path in sorted_markdown_files(&reports_root)? {
        let Some(name) = report_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(phase) = name
            .strip_prefix("PHASE_")
            .and_then(|rest| rest.strip_suffix("_REPORT.md"))
        else {
            continue;
        };
        let rel = relative_path(root, &report_path)?;
        rows.push(
            CrossPassTraceabilityRowV1::new(
                format!("{run_id}:phase-{phase}"),
                &run_id,
                format!("{run_id} phase {phase} acceptance evidence"),
                PassCompletionStateV1::Done,
            )
            .with_crates(vec!["AiDENs workspace".into()])
            .with_artifacts(vec![
                "z.py-package-certifier".into(),
                format!("{run_dir}-assertion-suite"),
                "phase-handoff-report".into(),
            ])
            .with_tests(vec![
                "cargo test --workspace --all-targets".into(),
                verifier_command.clone(),
            ])
            .with_docs(vec![
                "README.md".into(),
                "STATUS.md".into(),
                "SOURCE_BASIS.md".into(),
                rel.clone(),
            ])
            .with_acceptance_gates(vec![format!(
                "{run_id} phase acceptance gates recorded in handoff"
            )])
            .with_evidence(vec![rel]),
        );
    }
    Ok(rows)
}

fn is_p19_release_pass_id(pass_id: &str) -> bool {
    let Some(number) = pass_id
        .strip_prefix('P')
        .filter(|suffix| suffix.len() == 2)
        .and_then(|suffix| suffix.parse::<u8>().ok())
    else {
        return false;
    };
    number <= 19
}

fn sorted_markdown_files(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in
        std::fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn sorted_json_files(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in
        std::fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn string_array(value: &serde_json::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn handoff_for_pass(root: &Path, pass_id: &str) -> Option<String> {
    let handoff_root = root.join("handoffs");
    let entries = std::fs::read_dir(handoff_root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name()?.to_string_lossy();
        if file_name.starts_with(pass_id) && file_name.ends_with(".md") {
            return Some(format!("handoffs/{file_name}"));
        }
    }
    None
}

fn known_limitations_register() -> KnownLimitationsRegisterV1 {
    let mut limitations = SCAFFOLD_ONLY_CRATES
        .iter()
        .map(|(crate_name, note)| {
            KnownLimitationV1::deferred(
                format!("crate:{crate_name}"),
                format!("{crate_name} remains scaffold-only"),
                format!("operators must treat this crate as deferred; {note}"),
            )
            .with_review_after_pass("post-P19-roadmap")
        })
        .collect::<Vec<_>>();
    limitations.push(
        KnownLimitationV1::partial(
            "provider:http-api-boundaries",
            "OpenAI, OpenRouter, and Anthropic provider routes are diagnostic unavailable boundaries",
            "mock and local diagnostic routes are supported; HTTP API execution is not claimed",
        )
        .with_workaround("use examples/aidens.mock.toml for release smoke"),
    );
    limitations.push(
        KnownLimitationV1::partial(
            "release:binary-packaging",
            "P19 produces release manifests and audit artifacts, not signed binary installers",
            "operators can audit the source package but installer publication remains an external release operation",
        )
        .with_workaround("ship the source tree plus generated schemas and fixtures listed in the release artifact manifest"),
    );
    KnownLimitationsRegisterV1::new(limitations)
}

fn regression_debt_ledger() -> RegressionDebtLedgerV1 {
    RegressionDebtLedgerV1::new(vec![
        RegressionDebtItemV1::guarded(
            "release:docs",
            "public docs must not claim scaffold-only crates are complete",
            "scripts/assert_no_scaffold_promoted.sh",
            vec!["bash scripts/verify.sh".into()],
        ),
        RegressionDebtItemV1::guarded(
            "schemas:registry",
            "schema files can drift from Rust-owned artifact types",
            "aidens schemas check",
            vec!["cargo test -p aidens-cli schemas".into()],
        ),
        RegressionDebtItemV1::guarded(
            "provider:routes",
            "provider labels can regress into native-tool claims without executable proof",
            "provider route/readiness tests",
            vec!["cargo test -p aidens-provider-kit".into()],
        ),
        RegressionDebtItemV1::guarded(
            "memory:required-mode",
            "MemoryRequired paths can regress into running without a durable store",
            "doctor/run memory-required tests",
            vec!["cargo test -p aidens-cli memory_required".into()],
        ),
    ])
}

pub fn example_app_manifest(root: &Path) -> Result<ExampleAppManifestV1> {
    let examples_root = root.join("examples");
    let mut examples = Vec::new();
    for path in example_config_paths(&examples_root)? {
        let loaded = load_config_file(&path)?;
        let relative = relative_path(root, &path)?;
        let profile_id = loaded
            .config
            .profile_id
            .clone()
            .unwrap_or_else(|| "chat-only".into());
        let mut entry = ExampleAppEntryV1::new(
            relative.clone(),
            profile_id.clone(),
            loaded.config.provider.kind.clone(),
            loaded.config.memory_mode.clone(),
            example_status(&loaded.config, &profile_id),
        )
        .with_command(format!("aidens provider-check --config {relative}"));
        if loaded.config.provider.kind == "mock" {
            entry = entry.with_command(format!("aidens run --config {relative} hello"));
        }
        if profile_id == "coding-agent" {
            entry = entry.with_command(format!("aidens inspect-tools --config {relative}"));
        }
        for reason in example_reason_codes(&loaded.config, &profile_id) {
            entry = entry.with_reason(reason);
        }
        examples.push(entry);
    }
    Ok(ExampleAppManifestV1::new(
        examples,
        SCAFFOLD_ONLY_CRATES
            .iter()
            .map(|(crate_name, note)| format!("{crate_name}: {note}"))
            .collect(),
    ))
}

fn install_smoke_receipt(
    root: &Path,
    config: &str,
    include_verify: bool,
) -> Result<InstallSmokeReportV1> {
    let smoke_root = root
        .join("target")
        .join("p14-install-smoke")
        .join(unique_smoke_segment());
    std::fs::create_dir_all(&smoke_root)
        .with_context(|| format!("failed to create {}", smoke_root.display()))?;
    let mut steps = Vec::new();

    let app_dir = smoke_root.join("new-user-app");
    match scaffold_project_at(AiDENsProfile::CodingAgent, &app_dir) {
        Ok(summary) => steps.push(InstallSmokeStepV1::passed(
            "new-app",
            format!("aidens new coding-agent {}", app_dir.display()),
            &format!("created {}", summary.app_dir.display()),
        )),
        Err(error) => steps.push(InstallSmokeStepV1::failed(
            "new-app",
            format!("aidens new coding-agent {}", app_dir.display()),
            &error.to_string(),
            "new-app-failed",
        )),
    }

    let config_path = root.join(config);
    let config_arg = config_path.display().to_string();
    steps.push(smoke_step(
        "provider-check",
        format!("aidens provider-check --config {config}"),
        || provider_check(Some(config_arg.clone())),
    ));
    steps.push(smoke_step(
        "inspect-tools",
        format!("aidens inspect-tools --config {config}"),
        || inspect_tools(Some(config_arg.clone())),
    ));
    steps.push(smoke_step(
        "mock-turn",
        format!("aidens run --config {config} hello"),
        || run_once_command(Some(config_arg.clone()), vec!["hello".into()]),
    ));
    steps.push(smoke_step(
        "receipts-list",
        format!("aidens receipts list --config {config}"),
        || {
            receipts_command(EventLogCommand::List {
                store: None,
                config: Some(config_arg.clone()),
            })
        },
    ));
    if include_verify {
        steps.push(smoke_step("verify", "bash scripts/verify.sh", || {
            run_shell_command(root, "bash", &["scripts/verify.sh"])
        }));
    } else {
        steps.push(InstallSmokeStepV1::passed(
            "verify-script-present",
            "test -f scripts/verify.sh",
            &root.join("scripts").join("verify.sh").exists().to_string(),
        ));
    }

    Ok(InstallSmokeReportV1::new(steps))
}

fn smoke_step(
    step: impl Into<String>,
    command: impl Into<String>,
    run: impl FnOnce() -> Result<String>,
) -> InstallSmokeStepV1 {
    let step = step.into();
    let command = command.into();
    match run() {
        Ok(output) => InstallSmokeStepV1::passed(step, command, &output),
        Err(error) => {
            InstallSmokeStepV1::failed(step, command, &error.to_string(), "smoke-step-failed")
        }
    }
}

pub(crate) fn run_shell_command(root: &Path, program: &str, args: &[&str]) -> Result<String> {
    let sandbox_root = canonicalize_sandbox_root(root)?;
    let executable = resolve_allowed_command_executable(program)?;
    let timed_output = run_command_with_timeout(
        &sandbox_root,
        executable.to_string_lossy().as_ref(),
        args,
        std::time::Duration::from_secs(RUN_COMMAND_TIMEOUT_SECONDS),
        true,
    )
    .with_context(|| format!("failed to run {program} {}", args.join(" ")))?;
    if timed_output.timed_out {
        let combined = format!(
            "{}{}",
            capped_utf8_lossy(&timed_output.stdout, MAX_COMMAND_OUTPUT_BYTES),
            capped_utf8_lossy(&timed_output.stderr, MAX_COMMAND_OUTPUT_BYTES),
        );
        if timed_output.kill_failed {
            bail!(
                "{program} {} failed: command timed out and process cleanup may be incomplete: {combined}",
                args.join(" ")
            );
        }
        bail!(
            "{program} {} failed: command timed out: {combined}",
            args.join(" ")
        );
    }
    if !timed_output
        .status
        .as_ref()
        .is_some_and(std::process::ExitStatus::success)
    {
        let combined = format!(
            "{}{}",
            capped_utf8_lossy(&timed_output.stdout, MAX_COMMAND_OUTPUT_BYTES),
            capped_utf8_lossy(&timed_output.stderr, MAX_COMMAND_OUTPUT_BYTES),
        );
        bail!("{program} {} failed: {combined}", args.join(" "));
    }
    Ok(format!(
        "{}{}",
        capped_utf8_lossy(&timed_output.stdout, MAX_COMMAND_OUTPUT_BYTES),
        capped_utf8_lossy(&timed_output.stderr, MAX_COMMAND_OUTPUT_BYTES),
    ))
}

const MAX_COMMAND_OUTPUT_BYTES: usize = 65_536;
const RUN_COMMAND_TIMEOUT_SECONDS: u64 = 120;
const RUN_COMMAND_GRACEFUL_TERMINATE_SECONDS: u64 = 1;

fn canonicalize_sandbox_root(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("sandbox root does not exist: {}", path.display()))?;
    if !canonical.is_dir() {
        bail!("sandbox root is not a directory: {}", canonical.display());
    }
    Ok(canonical)
}

struct TimedCommandOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status: Option<std::process::ExitStatus>,
    timed_out: bool,
    kill_failed: bool,
}

fn run_command_with_timeout(
    sandbox_root: &Path,
    executable: &str,
    args: &[&str],
    timeout: std::time::Duration,
    kill_process_group: bool,
) -> anyhow::Result<TimedCommandOutput> {
    let mut command_proc = std::process::Command::new(executable);
    command_proc
        .args(args)
        .current_dir(sandbox_root)
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(unix)]
    if kill_process_group {
        command_proc.process_group(0);
    }
    let mut child = command_proc.spawn()?;
    let stdout = child
        .stdout
        .take()
        .context("failed to capture child stdout")?;
    let stderr = child
        .stderr
        .take()
        .context("failed to capture child stderr")?;
    let (stdout_tx, stdout_rx) = mpsc::channel::<Result<(Vec<u8>, bool), String>>();
    let (stderr_tx, stderr_rx) = mpsc::channel::<Result<(Vec<u8>, bool), String>>();

    let output_cap = MAX_COMMAND_OUTPUT_BYTES;
    std::thread::spawn(move || {
        let result = capture_stream_with_cap(stdout, output_cap).map_err(|error| error.to_string());
        let _ = stdout_tx.send(result);
    });
    let output_cap = MAX_COMMAND_OUTPUT_BYTES;
    std::thread::spawn(move || {
        let result = capture_stream_with_cap(stderr, output_cap).map_err(|error| error.to_string());
        let _ = stderr_tx.send(result);
    });

    let started = Instant::now();
    let mut wait_interval = Duration::from_millis(5);
    let mut status = None;
    let mut timed_out = false;
    let mut kill_failed = false;
    loop {
        if let Some(process_status) = child.try_wait()? {
            status = Some(process_status);
            break;
        }

        if started.elapsed() >= timeout {
            timed_out = true;
            let graceful_timeout = Duration::from_secs(RUN_COMMAND_GRACEFUL_TERMINATE_SECONDS);
            kill_failed = terminate_timed_out_command(&mut child, executable);
            let child_ended = await_child_exit_with_timeout(&mut child, graceful_timeout)?;
            if !child_ended {
                if child.kill().is_ok() {
                    let child_ended = await_child_exit_with_timeout(&mut child, graceful_timeout)?;
                    if !child_ended {
                        kill_failed = true;
                        return Ok(TimedCommandOutput {
                            stdout: Vec::new(),
                            stderr: b"command did not exit after timeout and kill attempts"
                                .to_vec(),
                            status: Some(timed_out_failure_exit_status()),
                            timed_out: true,
                            kill_failed,
                        });
                    }
                } else {
                    kill_failed = true;
                    return Ok(TimedCommandOutput {
                        stdout: Vec::new(),
                        stderr: b"command did not exit after timeout and direct kill failed"
                            .to_vec(),
                        status: Some(timed_out_failure_exit_status()),
                        timed_out: true,
                        kill_failed,
                    });
                }
            }
            if status.is_none() {
                status = child.try_wait()?;
            }
            break;
        }

        let elapsed = started.elapsed();
        let remaining = timeout.saturating_sub(elapsed);
        std::thread::sleep(wait_interval.min(remaining).min(Duration::from_millis(250)));
        wait_interval = (wait_interval * 2).min(Duration::from_millis(250));
    }

    if status.is_none() {
        status = child.try_wait()?;
        if status.is_none() {
            kill_failed = true;
        }
    }

    let collect_stream = |receiver: mpsc::Receiver<Result<(Vec<u8>, bool), String>>,
                          stream_name: &str|
     -> anyhow::Result<(Vec<u8>, bool)> {
        match receiver.recv_timeout(Duration::from_millis(250)) {
            Ok(result) => {
                result.map_err(|error| anyhow::anyhow!("{stream_name} capture failed: {error}"))
            }
            Err(mpsc::RecvTimeoutError::Timeout) => Ok((Vec::new(), true)),
            Err(mpsc::RecvTimeoutError::Disconnected) => Ok((Vec::new(), true)),
        }
    };

    let (stdout, stdout_truncated) = collect_stream(stdout_rx, "stdout")?;
    let (stderr, stderr_truncated) = collect_stream(stderr_rx, "stderr")?;

    let mut capped_stdout = capped_utf8_lossy(&stdout, MAX_COMMAND_OUTPUT_BYTES);
    let mut capped_stderr = capped_utf8_lossy(&stderr, MAX_COMMAND_OUTPUT_BYTES);
    if stdout_truncated {
        capped_stdout.push_str("[output truncated]");
    }
    if stderr_truncated {
        capped_stderr.push_str("[output truncated]");
    }

    Ok(TimedCommandOutput {
        stdout: capped_stdout.into_bytes(),
        stderr: capped_stderr.into_bytes(),
        status,
        timed_out,
        kill_failed,
    })
}

fn await_child_exit_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> anyhow::Result<bool> {
    let started = Instant::now();
    let mut wait_interval = Duration::from_millis(5);
    loop {
        if child.try_wait()?.is_some() {
            return Ok(true);
        }
        if started.elapsed() >= timeout {
            return Ok(false);
        }
        let remaining = timeout.saturating_sub(started.elapsed());
        std::thread::sleep(wait_interval.min(remaining).min(Duration::from_millis(50)));
        wait_interval = (wait_interval * 2).min(Duration::from_millis(50));
    }
}

fn timed_out_failure_exit_status() -> std::process::ExitStatus {
    #[cfg(unix)]
    {
        std::process::ExitStatus::from_raw(1 << 8)
    }
    #[cfg(windows)]
    {
        std::process::ExitStatus::from_raw(1)
    }
    #[cfg(not(any(unix, windows)))]
    {
        std::process::ExitStatus::default()
    }
}

fn terminate_timed_out_command(child: &mut std::process::Child, command_label: &str) -> bool {
    #[cfg(unix)]
    {
        if terminate_unix_process_group(child.id()).is_ok() {
            return false;
        }
        eprintln!(
            "WARNING: kill-failure for timed-out command {command_label}: process-group termination unavailable or failed"
        );
        if child.kill().is_err() {
            return true;
        }
        false
    }
    #[cfg(not(unix))]
    {
        if child.kill().is_ok() {
            return false;
        }
        eprintln!("WARNING: kill-failure for timed-out command {command_label}: timed-out command could not be killed");
        true
    }
}

#[cfg(unix)]
fn terminate_unix_process_group(child_pid: u32) -> anyhow::Result<()> {
    let process_group = format!("-{child_pid}");
    for kill_path in ["/bin/kill", "/usr/bin/kill"] {
        if !Path::new(kill_path).exists() {
            continue;
        }
        let status = std::process::Command::new(kill_path)
            .arg("-KILL")
            .arg(&process_group)
            .status()?;
        if status.success() {
            return Ok(());
        }
    }
    Err(anyhow::anyhow!("kill process-group failed"))
}

fn resolve_allowed_command_executable(command: &str) -> anyhow::Result<PathBuf> {
    let candidates: &[&str] = match command {
        "cargo" => &[
            "/usr/bin/cargo",
            "/usr/local/bin/cargo",
            "/root/.cargo/bin/cargo",
        ],
        "bash" => &["/usr/bin/bash", "/bin/bash"],
        other => bail!("command executable is not in the fixed allowlist: {other}"),
    };
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
        .ok_or_else(|| {
            anyhow::anyhow!("allowed command executable not found in fixed paths: {command}")
        })
}

fn capped_utf8_lossy(bytes: &[u8], cap: usize) -> String {
    String::from_utf8_lossy(&bytes[..bytes.len().min(cap)]).to_string()
}

fn capture_stream_with_cap(
    mut stream: impl Read + Send + 'static,
    cap: usize,
) -> std::io::Result<(Vec<u8>, bool)> {
    let mut captured = Vec::new();
    let mut buffer = [0_u8; 4096];
    let mut total_read = 0_usize;
    let mut truncated = false;
    loop {
        let read = match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(bytes) => bytes,
            Err(error) => return Err(error),
        };
        if total_read < cap {
            let keep = cap - total_read;
            captured.extend_from_slice(&buffer[..read.min(keep)]);
        } else {
            truncated = true;
        }
        total_read += read;
        if read > 0 && !truncated && total_read > cap {
            truncated = true;
        }
    }
    Ok((captured, truncated))
}

fn example_config_paths(examples_root: &Path) -> Result<Vec<PathBuf>> {
    if !examples_root.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(examples_root).with_context(|| {
        format!(
            "failed to read examples directory {}",
            examples_root.display()
        )
    })? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("toml") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn example_status(cfg: &AiDENsConfigV1, profile_id: &str) -> ReleaseSurfaceStateV1 {
    if provider_readiness_for_spec(&provider_spec_from_config(&cfg.provider)).executable {
        if matches!(
            profile_id,
            "memory-agent" | "autonomous-daemon" | "research-workbench"
        ) || cfg.provider.kind == "ollama"
        {
            ReleaseSurfaceStateV1::Partial
        } else {
            ReleaseSurfaceStateV1::Supported
        }
    } else if is_disabled_provider(&cfg.provider.kind) {
        ReleaseSurfaceStateV1::Deferred
    } else {
        ReleaseSurfaceStateV1::Degraded
    }
}

fn example_reason_codes(cfg: &AiDENsConfigV1, profile_id: &str) -> Vec<String> {
    let mut reasons = Vec::new();
    if matches!(
        profile_id,
        "memory-agent" | "autonomous-daemon" | "research-workbench"
    ) {
        reasons.push(format!("profile-surface-partial:{profile_id}"));
    }
    if !provider_readiness_for_spec(&provider_spec_from_config(&cfg.provider)).executable {
        reasons.push("provider-not-executable-in-local-smoke".into());
    }
    if cfg.provider.kind == "ollama" {
        reasons.push("provider-local-service-required:ollama".into());
        reasons.push("provider-surface-partial:ollama-chat".into());
    }
    if cfg.memory_mode == MemoryModeV1::Required && cfg.memory.store_root.is_none() {
        reasons.push("memory-required-without-store".into());
    }
    reasons
}

fn release_surfaces() -> Vec<ReleaseSurfaceV1> {
    let mut surfaces = vec![
        ReleaseSurfaceV1::new(
            "cli:status",
            ReleaseSurfaceStateV1::Supported,
            "operator status reports provider, memory, receipts, degraded, and blocked modes",
        )
        .with_command("aidens status --config examples/aidens.mock.toml"),
        ReleaseSurfaceV1::new(
            "cli:provider-check",
            ReleaseSurfaceStateV1::Supported,
            "provider-check uses executable backend truth",
        )
        .with_command("aidens provider-check --config examples/aidens.mock.toml"),
        ReleaseSurfaceV1::new(
            "cli:tools",
            ReleaseSurfaceStateV1::Supported,
            "tool listing distinguishes declared, registered, executable, hidden, and blocked tools",
        )
        .with_command("aidens tools inspect --config examples/aidens.mock.toml"),
        ReleaseSurfaceV1::new(
            "cli:receipts",
            ReleaseSurfaceStateV1::Supported,
            "receipt commands inspect durable NDJSON evidence",
        )
        .with_command("aidens receipts list --config examples/aidens.mock.toml"),
        ReleaseSurfaceV1::new(
            "cli:schemas",
            ReleaseSurfaceStateV1::Supported,
            "schema generation and compatibility checks are Rust-owned",
        )
        .with_command("aidens schemas check"),
        ReleaseSurfaceV1::new(
            "cli:memory",
            ReleaseSurfaceStateV1::Partial,
            "memory status and runtime views exist; profile wiring remains explicit",
        )
        .with_command("aidens memory status --config examples/aidens.memory.toml"),
        ReleaseSurfaceV1::new(
            "cli:queue",
            ReleaseSurfaceStateV1::Partial,
            "queue/daemon substrate exists; daemon profile packaging is deferred",
        )
        .with_command("aidens queue list --root target/aidens-daemon --name default"),
        ReleaseSurfaceV1::new(
            "cli:package",
            ReleaseSurfaceStateV1::Supported,
            "package readiness blocks on false scaffold claims and failed smoke steps",
        )
        .with_command("aidens package readiness"),
    ];
    surfaces.extend(SCAFFOLD_ONLY_CRATES.iter().map(|(crate_name, note)| {
        ReleaseSurfaceV1::new(
            format!("crate:{crate_name}"),
            ReleaseSurfaceStateV1::Deferred,
            format!("scaffold-only; {note}"),
        )
    }));
    surfaces
}

fn scan_public_docs_for_scaffold_claims(root: &Path) -> Result<Vec<PublicDocFindingV1>> {
    let mut paths = vec![
        root.join("README.md"),
        root.join("STATUS.md"),
        root.join("STATUS_TEMPLATE.md"),
        root.join("ARTIFACT_SCHEMA_REGISTRY.md"),
    ];
    collect_markdown_paths(&root.join("docs"), &mut paths)?;
    paths.sort();
    paths.dedup();

    let mut findings = Vec::new();
    for path in paths.into_iter().filter(|path| path.exists()) {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read public doc {}", path.display()))?;
        for (index, line) in contents.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            for (crate_name, _) in SCAFFOLD_ONLY_CRATES {
                if public_doc_line_promotes_scaffold(&lower, crate_name) {
                    findings.push(PublicDocFindingV1::scaffold_claim(
                        relative_path(root, &path)?,
                        (index + 1) as u32,
                        format!("crate:{crate_name}"),
                        line.trim().chars().take(240).collect::<String>(),
                    ));
                }
            }
        }
    }
    Ok(findings)
}

fn collect_markdown_paths(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root)
        .with_context(|| format!("failed to read docs directory {}", root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect file type for {}", path.display()))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_markdown_paths(&path, paths)?;
        } else if file_type.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("md")
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn public_doc_line_promotes_scaffold(line: &str, crate_name: &str) -> bool {
    line.contains(crate_name)
        && [
            "complete",
            "ready",
            "production",
            "healthy",
            "implemented",
            "available",
        ]
        .iter()
        .any(|word| line.contains(word))
        && ![
            "scaffold",
            "deferred",
            "horizon",
            "unsupported",
            "not available",
            "not implemented",
        ]
        .iter()
        .any(|word| line.contains(word))
}

fn unique_smoke_segment() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{}-{nanos}", std::process::id())
}

#[cfg(test)]
mod security_regression_tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "aidens-cli-package-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    #[cfg(unix)]
    #[test]
    fn p30_cli_package_collect_markdown_paths_ignores_symlinked_directories() {
        let root = temp_root();
        let docs = root.join("docs");
        let target = docs.join("notes");
        std::fs::create_dir_all(&target).unwrap();
        let _ = std::fs::write(docs.join("safe.md"), "Safe doc");
        let _ = std::fs::write(target.join("deep.md"), "Nested doc");

        std::os::unix::fs::symlink(&docs, docs.join("loop")).unwrap();

        let mut paths = Vec::new();
        collect_markdown_paths(&docs, &mut paths).unwrap();

        assert!(paths.iter().any(|path| path.ends_with("safe.md")));
        assert!(paths.iter().any(|path| path.ends_with("deep.md")));
        assert!(!paths.iter().any(|path| path.ends_with("loop")));
        assert!(paths.len() >= 2);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn p30_cli_package_collect_files_recursive_ignores_symlinked_directories() {
        let root = temp_root();
        let safe_dir = root.join("safe");
        std::fs::create_dir_all(&safe_dir).unwrap();
        let _ = std::fs::write(safe_dir.join("ok.txt"), "ok");

        std::os::unix::fs::symlink(&root, root.join("docs_link")).unwrap();

        let mut files = Vec::new();
        collect_files_recursive(&root, &mut files).unwrap();

        assert!(files.iter().any(|path| path.ends_with("ok.txt")));
        assert!(!files
            .iter()
            .any(|path| path.starts_with(root.join("docs_link"))));

        let _ = std::fs::remove_dir_all(&root);
    }
}
