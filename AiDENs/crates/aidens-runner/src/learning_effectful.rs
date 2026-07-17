//! Real effectful learning evaluation composed from canonical owner crates.
//!
//! This module owns only orchestration and a derived report. Patch application,
//! execution, verification, and causal attribution remain owner-crate truth.

use aidens_contracts::{
    ArtifactId, CanonicalBackpointerV1, CanonicalToolSideEffectClass, PermitGrantV1,
    PermitUseReportV1,
};
use cea_core::{attribute_effects, AttributedRunResult};
use cea_store::{CeaStore, UpdateResult};
use check_runner::{
    CheckKind, CheckResult, ExecutionBackend, ExecutionBackendKind, ParsedCheckOutput,
    SandboxCapabilityTruthReceiptV1,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use typed_patch::{apply_patch, validate_patch, PatchPolicy, StructuredPatch};
use verification_adjudication::{adjudicate_case, AdjudicationResult, VerificationDisposition};
use verification_calibration::CalibrationSnapshot;
use verification_control::{
    CaseRegion, CheckMethod, CheckPlan, ControlReceipt, PromotionClass, ReversibilityClass,
    VerificationAttempt, VerificationAttemptState, VerificationCase, VerificationCaseClass,
};
use verification_policy::{evaluate_policy, PolicySnapshot};

#[derive(Debug, Clone)]
pub struct EffectfulEvaluationRequest {
    pub fixture: PathBuf,
    pub patch: StructuredPatch,
    pub patch_policy: PatchPolicy,
    pub permit_grant: PermitGrantV1,
    pub permit_use: PermitUseReportV1,
    pub preflight_persisted: bool,
    pub run_id: String,
    pub attempt_id: String,
    pub trial_id: String,
    pub trace_id: String,
    pub recorded_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckEvidenceV1 {
    pub fmt_executed: bool,
    pub fmt_passed: bool,
    pub clippy_executed: bool,
    pub clippy_passed: bool,
    pub test_executed: bool,
    pub test_passed: bool,
    pub output_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectfulEvaluationReportV1 {
    pub schema: String,
    pub execution_mode: String,
    pub before_tree_digest: String,
    pub after_tree_digest: String,
    pub patch_digest: String,
    pub permit_use_receipt_id: String,
    pub sandbox_capability: SandboxCapabilityTruthReceiptV1,
    pub checks: CheckEvidenceV1,
    pub verification: AdjudicationResult,
    pub cea_run_hash: String,
    pub causal_triple_count: usize,
    pub cea_persisted: bool,
    pub cea_update_disposition: String,
    pub cea_backpointer: CanonicalBackpointerV1,
    pub verified: bool,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum EffectfulEvaluationError {
    #[error("sealed container execution is required")]
    SealedBackendRequired,
    #[error("preflight receipt must be durable before effect")]
    PreflightMissing,
    #[error("effect permit is invalid: {0}")]
    PermitInvalid(String),
    #[error("patch rejected: {0}")]
    PatchRejected(String),
    #[error("owner operation failed: {0}")]
    Owner(String),
}

pub async fn evaluate_effectful<B, F>(
    request: EffectfulEvaluationRequest,
    backend: &B,
    capability_receipt: F,
) -> Result<EffectfulEvaluationReportV1, EffectfulEvaluationError>
where
    B: ExecutionBackend,
    F: Fn() -> Option<SandboxCapabilityTruthReceiptV1>,
{
    evaluate_effectful_core(request, backend, capability_receipt)
        .await
        .map(|(report, _, _)| report)
}

/// Execute the canonical effectful path and durably apply its attribution to
/// the canonical CEA store. Repeated evaluation of the same material run is
/// owner-idempotent and remains publication-complete.
pub async fn evaluate_effectful_persisted<B, F, S>(
    request: EffectfulEvaluationRequest,
    backend: &B,
    capability_receipt: F,
    cea_store: &S,
) -> Result<EffectfulEvaluationReportV1, EffectfulEvaluationError>
where
    B: ExecutionBackend,
    F: Fn() -> Option<SandboxCapabilityTruthReceiptV1>,
    S: CeaStore,
{
    let eval_id = request.trial_id.clone();
    let version_id = request.patch.patch_id.to_string();
    let (mut report, attributed, execution_verified) =
        evaluate_effectful_core(request, backend, capability_receipt).await?;
    let update = cea_store::update_graph(cea_store, &attributed, &eval_id, &version_id, 0.85)
        .map_err(owner_error)?;
    report.cea_persisted = true;
    report.cea_update_disposition = match update {
        UpdateResult::Applied { .. } => "applied",
        UpdateResult::AlreadyProcessed => "already_processed",
    }
    .into();
    report.cea_backpointer = CanonicalBackpointerV1::external(
        "cea-store",
        "AttributedRunResult",
        "causal-attribution-record",
        report.cea_run_hash.clone(),
    );
    report.verified = execution_verified;
    report.reason_codes = if execution_verified {
        Vec::new()
    } else {
        vec!["independent-verification-not-positive".into()]
    };
    Ok(report)
}

async fn evaluate_effectful_core<B, F>(
    request: EffectfulEvaluationRequest,
    backend: &B,
    capability_receipt: F,
) -> Result<(EffectfulEvaluationReportV1, AttributedRunResult, bool), EffectfulEvaluationError>
where
    B: ExecutionBackend,
    F: Fn() -> Option<SandboxCapabilityTruthReceiptV1>,
{
    if backend.kind() != ExecutionBackendKind::Container {
        return Err(EffectfulEvaluationError::SealedBackendRequired);
    }
    if !request.preflight_persisted {
        return Err(EffectfulEvaluationError::PreflightMissing);
    }
    validate_effect_permit(&request)?;

    let patch_validation = validate_patch(&request.patch, &request.patch_policy);
    if !patch_validation.ok {
        return Err(EffectfulEvaluationError::PatchRejected(
            patch_validation
                .violations
                .iter()
                .map(|violation| violation.message.clone())
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }

    let workspace = backend
        .prepare_workspace(&request.fixture)
        .await
        .map_err(owner_error)?;
    let before_tree_digest = digest_tree(&workspace.host_path)?;
    let patch_fs = sandbox_workspace::LocalPatchFs::new(&workspace.host_path);
    let line_map = apply_patch(&request.patch, &patch_fs).map_err(owner_error)?;
    let after_tree_digest = digest_tree(&workspace.host_path)?;
    let patch_digest = digest_json(&request.patch)?;

    let fmt = backend
        .run_command(
            &workspace.host_path,
            "cargo",
            &["fmt", "--", "--check"],
            &[],
            0,
        )
        .await
        .map_err(owner_error)?;
    let clippy = backend
        .run_command(
            &workspace.host_path,
            "cargo",
            &["clippy", "--", "-D", "warnings"],
            &[],
            0,
        )
        .await
        .map_err(owner_error)?;
    let test = backend
        .run_command(&workspace.host_path, "cargo", &["test"], &[], 0)
        .await
        .map_err(owner_error)?;

    let capability = capability_receipt().ok_or_else(|| {
        EffectfulEvaluationError::Owner("sealed capability receipt missing after execution".into())
    })?;
    if !capability.verify() {
        return Err(EffectfulEvaluationError::Owner(
            "sealed capability receipt failed owner verification".into(),
        ));
    }

    let checks = CheckResult {
        fmt_pass: fmt.exit_code == 0,
        clippy_pass: clippy.exit_code == 0,
        test_pass: test.exit_code == 0,
        fmt_output: parsed(CheckKind::Fmt, &fmt),
        clippy_output: parsed(CheckKind::Clippy, &clippy),
        test_output: parsed(CheckKind::Test, &test),
        total_duration_ms: fmt
            .duration_ms
            .saturating_add(clippy.duration_ms)
            .saturating_add(test.duration_ms),
    };
    let triples = attribute_effects(&request.patch, &checks, &line_map, 12).map_err(owner_error)?;
    let attributed = AttributedRunResult::new(triples, checks.clone());
    let verification = adjudicate_real_checks(&request, &checks);
    let check_evidence = CheckEvidenceV1 {
        fmt_executed: true,
        fmt_passed: checks.fmt_pass,
        clippy_executed: true,
        clippy_passed: checks.clippy_pass,
        test_executed: true,
        test_passed: checks.test_pass,
        output_digest: digest_bytes(
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}",
                fmt.stdout, fmt.stderr, clippy.stdout, clippy.stderr, test.stdout, test.stderr
            )
            .as_bytes(),
        ),
    };
    let execution_verified = backend.has_live_execution_evidence()
        && checks.all_pass()
        && verification.disposition == VerificationDisposition::EligibleForPromotion;
    let report = EffectfulEvaluationReportV1 {
        schema: "AiDENsEffectfulEvaluationReportV1".into(),
        execution_mode: "real_sandbox".into(),
        before_tree_digest,
        after_tree_digest,
        patch_digest,
        permit_use_receipt_id: request.permit_use.receipt_id.as_str().to_string(),
        sandbox_capability: capability,
        checks: check_evidence,
        verification,
        cea_run_hash: attributed.run_hash.clone(),
        causal_triple_count: attributed.triples.len(),
        cea_persisted: false,
        cea_update_disposition: "not_persisted".into(),
        cea_backpointer: CanonicalBackpointerV1::external(
            "cea-core",
            "AttributedRunResult",
            "causal-attribution-unpersisted",
            attributed.run_hash.clone(),
        ),
        verified: false,
        reason_codes: vec!["causal-attribution-not-persisted".into()],
    };
    Ok((report, attributed, execution_verified))
}

fn validate_effect_permit(
    request: &EffectfulEvaluationRequest,
) -> Result<(), EffectfulEvaluationError> {
    let logical_root = request.fixture.to_string_lossy();
    let run_id = ArtifactId::new(request.run_id.clone());
    let attempt_id = ArtifactId::new(request.attempt_id.clone());
    let durable = |value: &ArtifactId| {
        !value.as_str().trim().is_empty() && !value.as_str().contains("local-process-seq")
    };
    let valid = request.permit_use.allowed
        && request.permit_use.tool_id == "aidens:patch-apply:1"
        && request.permit_use.risk_class == CanonicalToolSideEffectClass::Write
        && request.permit_use.sandbox_root == logical_root
        && request.permit_use.run_id.as_ref() == Some(&run_id)
        && request.permit_use.attempt_id.as_ref() == Some(&attempt_id)
        && request.permit_use.permit_id == request.permit_grant.permit_id
        && durable(&request.permit_use.receipt_id)
        && durable(&request.permit_use.permit_id)
        && request.permit_grant.matches_scope(
            &CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            &logical_root,
            Some(&run_id),
            Some(&attempt_id),
        );
    if valid {
        Ok(())
    } else {
        Err(EffectfulEvaluationError::PermitInvalid(
            "owner permit/use receipt is not durable, current, allowed, and run/attempt/scope-bound"
                .into(),
        ))
    }
}

fn adjudicate_real_checks(
    request: &EffectfulEvaluationRequest,
    checks: &CheckResult,
) -> AdjudicationResult {
    let case = VerificationCase::new(
        VerificationCaseClass::UnverifiedClaimVersion,
        CaseRegion {
            namespace: "aidens-learning".into(),
            scope_key: Some(stack_ids::ScopeKey::namespace_only("aidens-learning")),
            target_key: request.patch.patch_id.to_string(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: Some(stack_ids::ClaimVersionId::new(
                request.patch.patch_id.to_string(),
            )),
            as_of_recorded_at: None,
        },
        stack_ids::TraceCtx::from_trace_id(request.trace_id.clone()),
        stack_ids::AttemptId::new(request.attempt_id.clone()),
        request.recorded_at.clone(),
        false,
        false,
    );
    let plan = CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::PairedPatch,
        vec![
            "cargo_fmt".into(),
            "cargo_clippy".into(),
            "cargo_test".into(),
        ],
        PromotionClass::P1,
        ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "sealed-cargo-checks-v1",
        serde_json::json!({"source": "check-runner-owner"}),
    );
    let attempt = VerificationAttempt::completed(
        case.case_id.clone(),
        plan.plan_id.clone(),
        case.attempt_id.clone(),
        Some(stack_ids::TrialId::new(request.trial_id.clone())),
        if checks.all_pass() {
            VerificationAttemptState::Succeeded
        } else {
            VerificationAttemptState::Failed
        },
        false,
        false,
        request.recorded_at.clone(),
        request.recorded_at.clone(),
        Some(
            if checks.all_pass() {
                "sealed-checks-passed"
            } else {
                "sealed-checks-failed"
            }
            .into(),
        ),
    );
    let control = ControlReceipt::new_case_execution(
        &case,
        &plan,
        &attempt,
        checks.all_pass(),
        serde_json::json!({"check_result": checks.all_pass()}),
    );
    let policy = PolicySnapshot::permissive("aidens-learning-policy-v1", &request.recorded_at);
    let decision = evaluate_policy(&policy, &case, &plan, &[], false, false);
    let calibration = CalibrationSnapshot::evaluate(
        case.case_id.clone(),
        request.recorded_at.clone(),
        true,
        true,
        500_000,
        100_000,
        Vec::new(),
    );
    adjudicate_case(
        &case,
        &plan,
        &attempt,
        &control,
        &decision,
        &calibration,
        false,
        false,
        false,
    )
}

fn parsed(kind: CheckKind, output: &check_runner::CommandOutput) -> ParsedCheckOutput {
    ParsedCheckOutput {
        check_kind: kind,
        exit_code: output.exit_code,
        effects: Vec::new(),
        raw_stdout: output.stdout.clone(),
        raw_stderr: output.stderr.clone(),
    }
}

fn digest_tree(root: &Path) -> Result<String, EffectfulEvaluationError> {
    let mut files = walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .collect::<Vec<_>>();
    files.sort_by_key(|entry| entry.path().to_path_buf());
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"aidens-workspace-tree-v1\0");
    for entry in files {
        let relative = entry.path().strip_prefix(root).map_err(owner_error)?;
        let bytes = std::fs::read(entry.path()).map_err(owner_error)?;
        hash_framed(&mut hasher, relative.to_string_lossy().as_bytes());
        hash_framed(&mut hasher, &bytes);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn digest_json(value: &impl Serialize) -> Result<String, EffectfulEvaluationError> {
    serde_json::to_vec(value)
        .map(|bytes| digest_bytes(&bytes))
        .map_err(owner_error)
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn hash_framed(hasher: &mut blake3::Hasher, value: &[u8]) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value);
}

fn owner_error(error: impl std::fmt::Display) -> EffectfulEvaluationError {
    EffectfulEvaluationError::Owner(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidens_contracts::{ArtifactId, CanonicalToolSideEffectClass, PermitUseReportV1};
    use check_runner::{
        BackendConfig, CommandOutput, ExecutionBackendKind, HostBackend, LogBundle, RunnerError,
        Workspace,
    };

    struct FakeSealedBackend;

    #[async_trait::async_trait]
    impl ExecutionBackend for FakeSealedBackend {
        fn kind(&self) -> ExecutionBackendKind {
            ExecutionBackendKind::Container
        }

        async fn prepare_workspace(&self, fixture: &Path) -> Result<Workspace, RunnerError> {
            Ok(sandbox_workspace::prepare_workspace(fixture)?)
        }

        async fn run_command(
            &self,
            _workspace: &Path,
            _program: &str,
            _args: &[&str],
            _env: &[(&str, &str)],
            _timeout_secs: u64,
        ) -> Result<CommandOutput, RunnerError> {
            Ok(CommandOutput {
                stdout: "owner-check-passed".into(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 1,
            })
        }

        async fn collect_logs(
            &self,
            fmt: &CommandOutput,
            clippy: &CommandOutput,
            test: &CommandOutput,
        ) -> Result<LogBundle, RunnerError> {
            Ok(LogBundle {
                fmt_stdout: fmt.stdout.clone(),
                fmt_stderr: fmt.stderr.clone(),
                clippy_stdout: clippy.stdout.clone(),
                clippy_stderr: clippy.stderr.clone(),
                test_stdout: test.stdout.clone(),
                test_stderr: test.stderr.clone(),
                timings: check_runner::CommandTimings {
                    fmt_ms: fmt.duration_ms,
                    clippy_ms: clippy.duration_ms,
                    test_ms: test.duration_ms,
                },
            })
        }
    }

    fn valid_capability_receipt() -> SandboxCapabilityTruthReceiptV1 {
        let digest = format!("sha256:{}", "a".repeat(64));
        let mechanisms = [
            "cap_drop_all",
            "controlled_workspace_mount",
            "cpu_limit",
            "memory_limit",
            "network_none",
            "no_new_privileges",
            "pids_limit",
            "read_only_rootfs",
            "tmpfs_tmp",
            "userns_keep_id",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        let mut receipt = SandboxCapabilityTruthReceiptV1 {
            schema: "SandboxCapabilityTruthReceiptV1".into(),
            execution_mode: "sealed_local".into(),
            runtime: "podman".into(),
            requested_image: format!("rust@{digest}"),
            resolved_image_digest: digest,
            container_id: "test-container".into(),
            runtime_observation_digest: format!("blake3:{}", "b".repeat(64)),
            rootless_required: true,
            rootless_observed: true,
            mechanisms,
            memory_limit: "64m".into(),
            cpu_limit: "1".into(),
            content_digest: String::new(),
        };
        let material = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            receipt.schema,
            receipt.execution_mode,
            receipt.runtime,
            receipt.requested_image,
            receipt.resolved_image_digest,
            receipt.container_id,
            receipt.runtime_observation_digest,
            receipt.rootless_required,
            receipt.rootless_observed,
            receipt.mechanisms.join("\n"),
            receipt.memory_limit,
            receipt.cpu_limit,
        );
        receipt.content_digest = digest_bytes(material.as_bytes());
        assert!(receipt.verify());
        receipt
    }

    fn rejected_request() -> EffectfulEvaluationRequest {
        let patch: StructuredPatch = serde_json::from_value(serde_json::json!({
            "patch_id": "00000000-0000-4000-8000-000000000001",
            "summary": "bounded fixture patch",
            "edits": [{
                "path": "src/lib.rs",
                "ops": [{"Replace": {"range": {"start": 0, "end_exclusive": 1}, "lines": ["pub fn answer() -> u32 { 42 }"]}}],
                "mode": "Modify"
            }],
            "notes": []
        }))
        .unwrap();
        EffectfulEvaluationRequest {
            fixture: PathBuf::from("fixture"),
            patch,
            patch_policy: PatchPolicy {
                forbidden_paths: vec![".git".into()],
                allow_test_modifications: false,
                max_files_changed: 2,
                max_total_lines_changed: 20,
                max_lines_changed_per_file: 20,
            },
            permit_grant: PermitGrantV1::scoped(
                CanonicalToolSideEffectClass::Write,
                "aidens:patch-apply:1",
                "fixture",
                "test-operator",
            ),
            permit_use: PermitUseReportV1::denied(
                ArtifactId::new("permit:material"),
                "aidens:patch-apply:1",
                CanonicalToolSideEffectClass::Write,
                "fixture",
                "test-denial",
            ),
            preflight_persisted: true,
            run_id: "run:material".into(),
            attempt_id: "attempt:material".into(),
            trial_id: "trial:material".into(),
            trace_id: "0af7651916cd43dd8448eb211c80319c".into(),
            recorded_at: "2026-07-16T00:00:00Z".into(),
        }
    }

    #[tokio::test]
    async fn host_backend_can_never_satisfy_real_sandbox_evaluation() {
        let backend = HostBackend::new(&BackendConfig {
            mode: "host".into(),
            execution_backend_preference: "host".into(),
            container_runtime_preference: "none".into(),
            sealed_allow_host_backend: false,
            rust_image: "unused".into(),
            command_timeout_secs: 1,
            memory_limit: "64m".into(),
            cpu_limit: "1".into(),
        });
        let error = evaluate_effectful(rejected_request(), &backend, || None)
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            EffectfulEvaluationError::SealedBackendRequired
        ));
    }

    #[tokio::test]
    async fn real_evaluation_composes_patch_checks_verification_and_cea() {
        let fixture = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(fixture.path().join("src")).unwrap();
        std::fs::write(
            fixture.path().join("Cargo.toml"),
            "[package]\nname='fixture'\nversion='0.1.0'\nedition='2021'\n",
        )
        .unwrap();
        std::fs::write(
            fixture.path().join("src/lib.rs"),
            "pub fn answer() -> u32 { 0 }\n",
        )
        .unwrap();
        let patch: StructuredPatch = serde_json::from_value(serde_json::json!({
            "patch_id": "00000000-0000-4000-8000-000000000002",
            "summary": "bounded fixture patch",
            "edits": [{
                "path": "src/lib.rs",
                "ops": [{"Replace": {"range": {"start": 1, "end_exclusive": 2}, "lines": ["pub fn answer() -> u32 { 42 }"]}}],
                "mode": "Modify"
            }],
            "notes": []
        }))
        .unwrap();
        let root = fixture.path().to_string_lossy().to_string();
        let run_id = ArtifactId::new("run:material");
        let attempt_id = ArtifactId::new("attempt:material");
        let mut grant = PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            root.clone(),
            "operator",
        );
        grant.permit_id = ArtifactId::new("permit:material");
        grant.run_id = Some(run_id.clone());
        grant.attempt_id = Some(attempt_id.clone());
        let mut permit_use = PermitUseReportV1::allowed(
            &grant,
            "aidens:patch-apply:1",
            root,
            Some(run_id),
            Some(attempt_id),
        );
        permit_use.receipt_id = ArtifactId::new("permit-use:material");
        let request = EffectfulEvaluationRequest {
            fixture: fixture.path().to_path_buf(),
            patch,
            patch_policy: PatchPolicy {
                forbidden_paths: vec![".git".into()],
                allow_test_modifications: false,
                max_files_changed: 2,
                max_total_lines_changed: 20,
                max_lines_changed_per_file: 20,
            },
            permit_grant: grant,
            permit_use,
            preflight_persisted: true,
            run_id: "run:material".into(),
            attempt_id: "attempt:material".into(),
            trial_id: "trial:material".into(),
            trace_id: "0af7651916cd43dd8448eb211c80319c".into(),
            recorded_at: "2026-07-16T00:00:00Z".into(),
        };
        let capability = valid_capability_receipt();
        let report = evaluate_effectful(request, &FakeSealedBackend, || Some(capability.clone()))
            .await
            .unwrap();
        assert!(
            !report.verified,
            "synthetic backend evidence must not be publication-complete"
        );
        assert_eq!(report.execution_mode, "real_sandbox");
        assert_ne!(report.before_tree_digest, report.after_tree_digest);
        assert_eq!(
            report.verification.disposition,
            VerificationDisposition::EligibleForPromotion
        );
        assert!(!report.cea_run_hash.is_empty());
    }

    #[tokio::test]
    async fn effectful_evaluation_persists_cea_idempotently_through_owner_store() {
        let fixture = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(fixture.path().join("src")).unwrap();
        std::fs::write(
            fixture.path().join("Cargo.toml"),
            "[package]\nname='fixture'\nversion='0.1.0'\nedition='2021'\n",
        )
        .unwrap();
        std::fs::write(
            fixture.path().join("src/lib.rs"),
            "pub fn answer() -> u32 { 0 }\n",
        )
        .unwrap();
        let build_request = || {
            let patch: StructuredPatch = serde_json::from_value(serde_json::json!({
                "patch_id": "00000000-0000-4000-8000-000000000004",
                "summary": "bounded fixture patch",
                "edits": [{
                    "path": "src/lib.rs",
                    "ops": [{"Replace": {"range": {"start": 1, "end_exclusive": 2}, "lines": ["pub fn answer() -> u32 { 42 }"]}}],
                    "mode": "Modify"
                }],
                "notes": []
            }))
            .unwrap();
            let root = fixture.path().to_string_lossy().to_string();
            let run_id = ArtifactId::new("run:persisted-material");
            let attempt_id = ArtifactId::new("attempt:persisted-material");
            let mut grant = PermitGrantV1::scoped(
                CanonicalToolSideEffectClass::Write,
                "aidens:patch-apply:1",
                root.clone(),
                "operator",
            );
            grant.permit_id = ArtifactId::new("permit:persisted-material");
            grant.run_id = Some(run_id.clone());
            grant.attempt_id = Some(attempt_id.clone());
            let mut permit_use = PermitUseReportV1::allowed(
                &grant,
                "aidens:patch-apply:1",
                root,
                Some(run_id),
                Some(attempt_id),
            );
            permit_use.receipt_id = ArtifactId::new("permit-use:persisted-material");
            EffectfulEvaluationRequest {
                fixture: fixture.path().to_path_buf(),
                patch,
                patch_policy: PatchPolicy {
                    forbidden_paths: vec![".git".into()],
                    allow_test_modifications: false,
                    max_files_changed: 2,
                    max_total_lines_changed: 20,
                    max_lines_changed_per_file: 20,
                },
                permit_grant: grant,
                permit_use,
                preflight_persisted: true,
                run_id: "run:persisted-material".into(),
                attempt_id: "attempt:persisted-material".into(),
                trial_id: "trial:persisted-material".into(),
                trace_id: "0af7651916cd43dd8448eb211c80319c".into(),
                recorded_at: "2026-07-16T00:00:00Z".into(),
            }
        };
        let store_dir = tempfile::tempdir().unwrap();
        let store = cea_sqlite::SqliteCeaStore::open(&store_dir.path().join("cea.sqlite")).unwrap();
        let capability = valid_capability_receipt();

        let first = evaluate_effectful_persisted(
            build_request(),
            &FakeSealedBackend,
            || Some(capability.clone()),
            &store,
        )
        .await
        .unwrap();
        let second = evaluate_effectful_persisted(
            build_request(),
            &FakeSealedBackend,
            || Some(capability.clone()),
            &store,
        )
        .await
        .unwrap();

        assert!(first.cea_persisted);
        assert_eq!(first.cea_update_disposition, "applied");
        assert_eq!(second.cea_update_disposition, "already_processed");
        assert_eq!(
            first.cea_backpointer.external_id.as_deref(),
            Some(first.cea_run_hash.as_str())
        );
    }

    #[tokio::test]
    #[ignore = "requires an explicitly supplied digest-pinned Rust image and live rootless Podman"]
    async fn live_rootless_podman_executes_the_effectful_vertical_slice() {
        let image = std::env::var("AIDENS_LIVE_RUST_IMAGE")
            .expect("AIDENS_LIVE_RUST_IMAGE must contain a digest-pinned Rust image");
        let fixture = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(fixture.path().join("src")).unwrap();
        std::fs::write(
            fixture.path().join("Cargo.toml"),
            "[package]\nname='live_fixture'\nversion='0.1.0'\nedition='2021'\n",
        )
        .unwrap();
        std::fs::write(
            fixture.path().join("src/lib.rs"),
            "pub fn answer() -> u32 { 0 }\n",
        )
        .unwrap();
        let patch: StructuredPatch = serde_json::from_value(serde_json::json!({
            "patch_id": "00000000-0000-4000-8000-000000000003",
            "summary": "live bounded fixture patch",
            "edits": [{
                "path": "src/lib.rs",
                "ops": [{"Replace": {"range": {"start": 1, "end_exclusive": 2}, "lines": [
                    "pub fn answer() -> u32 {",
                    "    42",
                    "}"
                ]}}],
                "mode": "Modify"
            }],
            "notes": []
        }))
        .unwrap();
        let root = fixture.path().to_string_lossy().to_string();
        let run_id = ArtifactId::new("run:live-podman-material");
        let attempt_id = ArtifactId::new("attempt:live-podman-material");
        let mut grant = PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            root.clone(),
            "test-operator",
        );
        grant.permit_id = ArtifactId::new("permit:live-podman-material");
        grant.run_id = Some(run_id.clone());
        grant.attempt_id = Some(attempt_id.clone());
        let mut permit_use = PermitUseReportV1::allowed(
            &grant,
            "aidens:patch-apply:1",
            root,
            Some(run_id),
            Some(attempt_id),
        );
        permit_use.receipt_id = ArtifactId::new("permit-use:live-podman-material");
        let request = EffectfulEvaluationRequest {
            fixture: fixture.path().to_path_buf(),
            patch,
            patch_policy: PatchPolicy {
                forbidden_paths: vec![".git".into()],
                allow_test_modifications: false,
                max_files_changed: 2,
                max_total_lines_changed: 20,
                max_lines_changed_per_file: 20,
            },
            permit_grant: grant,
            permit_use,
            preflight_persisted: true,
            run_id: "run:live-podman-material".into(),
            attempt_id: "attempt:live-podman-material".into(),
            trial_id: "trial:live-podman-material".into(),
            trace_id: "0af7651916cd43dd8448eb211c80319c".into(),
            recorded_at: "2026-07-16T00:00:00Z".into(),
        };
        let backend = check_runner::ContainerBackend::new(&BackendConfig {
            mode: "sealed_local".into(),
            execution_backend_preference: "container".into(),
            container_runtime_preference: "podman".into(),
            sealed_allow_host_backend: false,
            rust_image: image,
            command_timeout_secs: 180,
            memory_limit: "1g".into(),
            cpu_limit: "2".into(),
        })
        .unwrap();
        let store_dir = tempfile::tempdir().unwrap();
        let store = cea_sqlite::SqliteCeaStore::open(&store_dir.path().join("cea.sqlite")).unwrap();
        let report = evaluate_effectful_persisted(
            request,
            &backend,
            || backend.capability_truth_receipt(),
            &store,
        )
        .await
        .unwrap();
        assert!(report.verified, "live report was not verified: {report:#?}");
        assert!(report.cea_persisted);
        assert!(report.sandbox_capability.verify());
        assert!(report.checks.fmt_executed);
        assert!(report.checks.clippy_executed);
        assert!(report.checks.test_executed);
    }
}
