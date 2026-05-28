//! Release readiness, traceability, limitations, debt, and completion audit artifacts.
//!
//! These gates prevent false-green release claims and do not waive canonical proof obligations.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReleaseSurfaceStateV1 {
    Supported,
    Partial,
    Deferred,
    Degraded,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseSurfaceV1 {
    pub surface_id: String,
    pub state: ReleaseSurfaceStateV1,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

impl ReleaseSurfaceV1 {
    pub fn new(
        surface_id: impl Into<String>,
        state: ReleaseSurfaceStateV1,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            surface_id: surface_id.into(),
            state,
            reason: reason.into(),
            command: None,
        }
    }

    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PublicDocFindingV1 {
    pub path: String,
    pub line: u32,
    pub surface_id: String,
    pub reason_code: String,
    pub excerpt: String,
}

impl PublicDocFindingV1 {
    pub fn scaffold_claim(
        path: impl Into<String>,
        line: u32,
        surface_id: impl Into<String>,
        excerpt: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            line,
            surface_id: surface_id.into(),
            reason_code: "public-doc-claims-scaffold-complete".into(),
            excerpt: excerpt.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExampleAppEntryV1 {
    pub example_id: ArtifactId,
    pub path: String,
    pub profile_id: String,
    pub provider_kind: String,
    pub memory_mode: MemoryModeV1,
    pub status: ReleaseSurfaceStateV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ExampleAppEntryV1 {
    pub fn new(
        path: impl Into<String>,
        profile_id: impl Into<String>,
        provider_kind: impl Into<String>,
        memory_mode: MemoryModeV1,
        status: ReleaseSurfaceStateV1,
    ) -> Self {
        Self {
            example_id: display_only_unstable_id("example-app"),
            path: path.into(),
            profile_id: profile_id.into(),
            provider_kind: provider_kind.into(),
            memory_mode,
            status,
            commands: Vec::new(),
            reason_codes: vec!["example-profile-declared".into()],
        }
    }

    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.commands.push(command.into());
        self
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason_codes.push(reason.into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExampleAppManifestV1 {
    pub manifest_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub examples: Vec<ExampleAppEntryV1>,
    pub profiles_covered: Vec<String>,
    pub unsupported_advanced_features: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl ExampleAppManifestV1 {
    pub fn new(
        mut examples: Vec<ExampleAppEntryV1>,
        mut unsupported_advanced_features: Vec<String>,
    ) -> Self {
        examples.sort_by(|left, right| left.path.cmp(&right.path));
        let mut profiles_covered = examples
            .iter()
            .map(|example| example.profile_id.clone())
            .collect::<Vec<_>>();
        profiles_covered.sort();
        profiles_covered.dedup();
        unsupported_advanced_features.sort();
        unsupported_advanced_features.dedup();
        Self {
            manifest_id: display_only_unstable_id("example-app-manifest"),
            kind: ArtifactKindV1::ExampleAppManifest,
            examples,
            profiles_covered,
            unsupported_advanced_features,
            reason_codes: vec!["examples-declare-supported-and-deferred-surfaces".into()],
            generated_at: Utc::now(),
        }
    }

    pub fn covers_profile(&self, profile_id: &str) -> bool {
        self.profiles_covered
            .iter()
            .any(|profile| profile == profile_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InstallSmokeStepV1 {
    pub step: String,
    pub command: String,
    pub passed: bool,
    pub output_digest: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl InstallSmokeStepV1 {
    pub fn passed(step: impl Into<String>, command: impl Into<String>, output: &str) -> Self {
        Self {
            step: step.into(),
            command: command.into(),
            passed: true,
            output_digest: non_authoritative_text_display_digest(output),
            reason_codes: vec!["install-smoke-step-passed".into()],
        }
    }

    pub fn failed(
        step: impl Into<String>,
        command: impl Into<String>,
        output: &str,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            step: step.into(),
            command: command.into(),
            passed: false,
            output_digest: non_authoritative_text_display_digest(output),
            reason_codes: vec![reason.into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InstallSmokeReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub passed: bool,
    pub steps: Vec<InstallSmokeStepV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl InstallSmokeReportV1 {
    pub fn new(steps: Vec<InstallSmokeStepV1>) -> Self {
        let passed = steps.iter().all(|step| step.passed);
        Self {
            receipt_id: display_only_unstable_id("install-smoke"),
            kind: ArtifactKindV1::InstallSmokeReport,
            passed,
            steps,
            reason_codes: if passed {
                vec!["install-smoke-passed".into()]
            } else {
                vec!["install-smoke-failed".into()]
            },
            recorded_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OperatorStatusReportV1 {
    pub report_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub app_id: String,
    pub config_status: String,
    pub provider_route_label: String,
    pub memory_mode: MemoryModeV1,
    pub receipt_store_configured: bool,
    pub doctor: AiDENsDoctorReportV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degraded_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub next_commands: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl OperatorStatusReportV1 {
    pub fn new(
        app_id: impl Into<String>,
        config_status: impl Into<String>,
        provider_route_label: impl Into<String>,
        memory_mode: MemoryModeV1,
        receipt_store_configured: bool,
        doctor: AiDENsDoctorReportV1,
    ) -> Self {
        let mut degraded_modes = Vec::new();
        let mut blocked_modes = Vec::new();
        for truth in doctor.sections.values().flat_map(|section| section.iter()) {
            if truth.states.contains(&CapabilityStateV1::Degraded) {
                degraded_modes.push(truth.capability_id.clone());
            }
            if truth.states.contains(&CapabilityStateV1::BlockedByPolicy)
                || truth.states.contains(&CapabilityStateV1::Unavailable)
            {
                blocked_modes.push(truth.capability_id.clone());
            }
        }
        degraded_modes.sort();
        degraded_modes.dedup();
        blocked_modes.sort();
        blocked_modes.dedup();
        Self {
            report_id: display_only_unstable_id("operator-status"),
            kind: ArtifactKindV1::OperatorStatusReport,
            app_id: app_id.into(),
            config_status: config_status.into(),
            provider_route_label: provider_route_label.into(),
            memory_mode,
            receipt_store_configured,
            doctor,
            degraded_modes,
            blocked_modes,
            next_commands: vec![
                "aidens provider-check --config <config>".into(),
                "aidens inspect-tools --config <config>".into(),
                "aidens receipts list --config <config>".into(),
            ],
            generated_at: Utc::now(),
        }
    }

    pub fn exposes_degraded_modes(&self) -> bool {
        !self.degraded_modes.is_empty() || !self.blocked_modes.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseReadinessReportV1 {
    pub report_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub ready: bool,
    pub surfaces: Vec<ReleaseSurfaceV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub public_doc_findings: Vec<PublicDocFindingV1>,
    pub example_manifest: ExampleAppManifestV1,
    pub install_smoke: InstallSmokeReportV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl ReleaseReadinessReportV1 {
    pub fn new(
        mut surfaces: Vec<ReleaseSurfaceV1>,
        public_doc_findings: Vec<PublicDocFindingV1>,
        example_manifest: ExampleAppManifestV1,
        install_smoke: InstallSmokeReportV1,
    ) -> Self {
        surfaces.sort_by(|left, right| left.surface_id.cmp(&right.surface_id));
        let has_blocked_surface = surfaces
            .iter()
            .any(|surface| surface.state == ReleaseSurfaceStateV1::Blocked);
        let has_degraded_surface = surfaces
            .iter()
            .any(|surface| surface.state == ReleaseSurfaceStateV1::Degraded);
        let ready = public_doc_findings.is_empty()
            && install_smoke.passed
            && !has_blocked_surface
            && !has_degraded_surface;
        let mut reason_codes = if ready {
            vec!["release-readiness-passed".into()]
        } else {
            vec!["release-readiness-blocked".into()]
        };
        if has_blocked_surface {
            reason_codes.push("blocked-surfaces-present".into());
        }
        if has_degraded_surface {
            reason_codes.push("degraded-surfaces-require-explicit-waiver".into());
        }
        reason_codes.sort();
        reason_codes.dedup();
        Self {
            report_id: display_only_unstable_id("release-readiness"),
            kind: ArtifactKindV1::ReleaseReadinessReport,
            ready,
            surfaces,
            public_doc_findings,
            example_manifest,
            install_smoke,
            reason_codes,
            generated_at: Utc::now(),
        }
    }

    pub fn blocks_release(&self) -> bool {
        !self.ready
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CompletionAuditStateV1 {
    Complete,
    NearComplete,
    DeferredHorizon,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PassCompletionStateV1 {
    Done,
    Partial,
    Deferred,
    Blocked,
    Waived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum GateCommandStatusV1 {
    Passed,
    Failed,
    Waived,
    NotRun,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GateCommandResultV1 {
    pub command: String,
    pub status: GateCommandStatusV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub completed_at: DateTime<Utc>,
}

impl GateCommandResultV1 {
    pub fn passed(command: impl Into<String>, evidence_ref: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            status: GateCommandStatusV1::Passed,
            exit_code: Some(0),
            output_digest: None,
            evidence_ref: Some(evidence_ref.into()),
            reason_codes: vec!["gate-command-passed".into()],
            completed_at: Utc::now(),
        }
    }

    pub fn failed(
        command: impl Into<String>,
        exit_code: i32,
        output: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            command: command.into(),
            status: GateCommandStatusV1::Failed,
            exit_code: Some(exit_code),
            output_digest: Some(non_authoritative_text_display_digest(&output.into())),
            evidence_ref: None,
            reason_codes: vec![reason.into()],
            completed_at: Utc::now(),
        }
    }

    pub fn not_run(command: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            status: GateCommandStatusV1::NotRun,
            exit_code: None,
            output_digest: None,
            evidence_ref: None,
            reason_codes: vec![reason.into()],
            completed_at: Utc::now(),
        }
    }

    pub fn waived(command: impl Into<String>, waiver_receipt_id: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            status: GateCommandStatusV1::Waived,
            exit_code: None,
            output_digest: None,
            evidence_ref: Some(waiver_receipt_id.into()),
            reason_codes: vec!["gate-command-waived".into()],
            completed_at: Utc::now(),
        }
    }

    pub fn is_satisfied(&self) -> bool {
        matches!(
            self.status,
            GateCommandStatusV1::Passed | GateCommandStatusV1::Waived
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReleaseArtifactKindV1 {
    Source,
    Document,
    Schema,
    Fixture,
    Example,
    Script,
    Ci,
    Handoff,
    Manifest,
    Generated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseArtifactEntryV1 {
    pub path: String,
    pub artifact_kind: ReleaseArtifactKindV1,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_digest: Option<String>,
    pub byte_len: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ReleaseArtifactEntryV1 {
    pub fn present(
        path: impl Into<String>,
        artifact_kind: ReleaseArtifactKindV1,
        required: bool,
        content_digest: impl Into<String>,
        byte_len: u64,
    ) -> Self {
        Self {
            path: path.into(),
            artifact_kind,
            required,
            content_digest: Some(content_digest.into()),
            byte_len,
            reason_codes: vec!["release-artifact-present".into()],
        }
    }

    pub fn missing(
        path: impl Into<String>,
        artifact_kind: ReleaseArtifactKindV1,
        required: bool,
    ) -> Self {
        Self {
            path: path.into(),
            artifact_kind,
            required,
            content_digest: None,
            byte_len: 0,
            reason_codes: vec!["release-artifact-missing".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseArtifactManifestV1 {
    pub manifest_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub package_name: String,
    pub source_basis: String,
    pub artifacts: Vec<ReleaseArtifactEntryV1>,
    pub missing_required_paths: Vec<String>,
    pub complete: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl ReleaseArtifactManifestV1 {
    pub fn new(
        package_name: impl Into<String>,
        source_basis: impl Into<String>,
        mut artifacts: Vec<ReleaseArtifactEntryV1>,
    ) -> Self {
        artifacts.sort_by(|left, right| left.path.cmp(&right.path));
        let missing_required_paths = artifacts
            .iter()
            .filter(|artifact| artifact.required && artifact.content_digest.is_none())
            .map(|artifact| artifact.path.clone())
            .collect::<Vec<_>>();
        let complete = missing_required_paths.is_empty();
        Self {
            manifest_id: display_only_unstable_id("release-artifact-manifest"),
            kind: ArtifactKindV1::ReleaseArtifactManifest,
            package_name: package_name.into(),
            source_basis: source_basis.into(),
            artifacts,
            missing_required_paths,
            complete,
            reason_codes: if complete {
                vec!["release-artifact-manifest-complete".into()]
            } else {
                vec!["release-artifact-manifest-missing-required".into()]
            },
            generated_at: Utc::now(),
        }
    }

    pub fn blocks_release(&self) -> bool {
        !self.complete
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CrossPassTraceabilityRowV1 {
    pub requirement_id: String,
    pub pass_id: String,
    pub requirement: String,
    pub state: PassCompletionStateV1,
    pub crates: Vec<String>,
    pub artifact_families: Vec<String>,
    pub tests: Vec<String>,
    pub docs: Vec<String>,
    pub acceptance_gates: Vec<String>,
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waiver_receipt_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl CrossPassTraceabilityRowV1 {
    pub fn new(
        requirement_id: impl Into<String>,
        pass_id: impl Into<String>,
        requirement: impl Into<String>,
        state: PassCompletionStateV1,
    ) -> Self {
        Self {
            requirement_id: requirement_id.into(),
            pass_id: pass_id.into(),
            requirement: requirement.into(),
            state,
            crates: Vec::new(),
            artifact_families: Vec::new(),
            tests: Vec::new(),
            docs: Vec::new(),
            acceptance_gates: Vec::new(),
            evidence_refs: Vec::new(),
            waiver_receipt_ids: Vec::new(),
            reason_codes: vec!["traceability-row-declared".into()],
        }
    }

    pub fn with_crates(mut self, crates: Vec<String>) -> Self {
        self.crates = crates;
        self
    }

    pub fn with_artifacts(mut self, artifact_families: Vec<String>) -> Self {
        self.artifact_families = artifact_families;
        self
    }

    pub fn with_tests(mut self, tests: Vec<String>) -> Self {
        self.tests = tests;
        self
    }

    pub fn with_docs(mut self, docs: Vec<String>) -> Self {
        self.docs = docs;
        self
    }

    pub fn with_acceptance_gates(mut self, acceptance_gates: Vec<String>) -> Self {
        self.acceptance_gates = acceptance_gates;
        self
    }

    pub fn with_evidence(mut self, evidence_refs: Vec<String>) -> Self {
        self.evidence_refs = evidence_refs;
        self
    }

    pub fn with_waiver(mut self, waiver_receipt_id: impl Into<String>) -> Self {
        self.waiver_receipt_ids.push(waiver_receipt_id.into());
        self
    }

    pub fn is_satisfied(&self) -> bool {
        matches!(
            self.state,
            PassCompletionStateV1::Done | PassCompletionStateV1::Waived
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CrossPassTraceabilityMatrixV1 {
    pub matrix_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub rows: Vec<CrossPassTraceabilityRowV1>,
    pub uncovered_requirement_ids: Vec<String>,
    pub complete: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl CrossPassTraceabilityMatrixV1 {
    pub fn new(mut rows: Vec<CrossPassTraceabilityRowV1>) -> Self {
        rows.sort_by(|left, right| left.requirement_id.cmp(&right.requirement_id));
        let uncovered_requirement_ids = rows
            .iter()
            .filter(|row| {
                !row.is_satisfied()
                    || row.acceptance_gates.is_empty()
                    || row.docs.is_empty()
                    || row.tests.is_empty()
                    || row.artifact_families.is_empty()
            })
            .map(|row| row.requirement_id.clone())
            .collect::<Vec<_>>();
        let complete = !rows.is_empty() && uncovered_requirement_ids.is_empty();
        Self {
            matrix_id: display_only_unstable_id("cross-pass-traceability-matrix"),
            kind: ArtifactKindV1::CrossPassTraceabilityMatrix,
            rows,
            uncovered_requirement_ids,
            complete,
            reason_codes: if complete {
                vec!["cross-pass-traceability-complete".into()]
            } else {
                vec!["cross-pass-traceability-has-gaps".into()]
            },
            generated_at: Utc::now(),
        }
    }

    pub fn blocks_completion(&self) -> bool {
        !self.complete
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct KnownLimitationV1 {
    pub limitation_id: ArtifactId,
    pub surface_id: String,
    pub state: ReleaseSurfaceStateV1,
    pub description: String,
    pub impact: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workaround: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_after_pass: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl KnownLimitationV1 {
    pub fn deferred(
        surface_id: impl Into<String>,
        description: impl Into<String>,
        impact: impl Into<String>,
    ) -> Self {
        Self {
            limitation_id: display_only_unstable_id("known-limitation"),
            surface_id: surface_id.into(),
            state: ReleaseSurfaceStateV1::Deferred,
            description: description.into(),
            impact: impact.into(),
            workaround: None,
            review_after_pass: None,
            reason_codes: vec!["limitation-deferred-not-hidden".into()],
        }
    }

    pub fn partial(
        surface_id: impl Into<String>,
        description: impl Into<String>,
        impact: impl Into<String>,
    ) -> Self {
        Self {
            limitation_id: display_only_unstable_id("known-limitation"),
            surface_id: surface_id.into(),
            state: ReleaseSurfaceStateV1::Partial,
            description: description.into(),
            impact: impact.into(),
            workaround: None,
            review_after_pass: None,
            reason_codes: vec!["limitation-partial-not-hidden".into()],
        }
    }

    pub fn blocked(
        surface_id: impl Into<String>,
        description: impl Into<String>,
        impact: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            limitation_id: display_only_unstable_id("known-limitation"),
            surface_id: surface_id.into(),
            state: ReleaseSurfaceStateV1::Blocked,
            description: description.into(),
            impact: impact.into(),
            workaround: None,
            review_after_pass: None,
            reason_codes: vec![reason.into()],
        }
    }

    pub fn with_workaround(mut self, workaround: impl Into<String>) -> Self {
        self.workaround = Some(workaround.into());
        self
    }

    pub fn with_review_after_pass(mut self, pass_id: impl Into<String>) -> Self {
        self.review_after_pass = Some(pass_id.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct KnownLimitationsRegisterV1 {
    pub register_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub current: bool,
    pub limitations: Vec<KnownLimitationV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_limitation_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl KnownLimitationsRegisterV1 {
    pub fn new(mut limitations: Vec<KnownLimitationV1>) -> Self {
        limitations.sort_by(|left, right| left.surface_id.cmp(&right.surface_id));
        let blocking_limitation_ids = limitations
            .iter()
            .filter(|limitation| limitation.state == ReleaseSurfaceStateV1::Blocked)
            .map(|limitation| limitation.limitation_id.clone())
            .collect::<Vec<_>>();
        let empty_register = limitations.is_empty();
        let current = true;
        Self {
            register_id: display_only_unstable_id("known-limitations-register"),
            kind: ArtifactKindV1::KnownLimitationsRegister,
            current,
            limitations,
            blocking_limitation_ids,
            reason_codes: if current {
                if empty_register {
                    vec!["known-limitations-empty-register-current".into()]
                } else {
                    vec!["known-limitations-current".into()]
                }
            } else {
                vec!["known-limitations-register-empty".into()]
            },
            generated_at: Utc::now(),
        }
    }

    pub fn blocks_completion(&self) -> bool {
        !self.blocking_limitation_ids.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RegressionDebtStatusV1 {
    Guarded,
    Accepted,
    Deferred,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegressionDebtItemV1 {
    pub debt_id: ArtifactId,
    pub surface_id: String,
    pub status: RegressionDebtStatusV1,
    pub description: String,
    pub detection: String,
    pub guardrail_tests: Vec<String>,
    pub owner: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl RegressionDebtItemV1 {
    pub fn guarded(
        surface_id: impl Into<String>,
        description: impl Into<String>,
        detection: impl Into<String>,
        guardrail_tests: Vec<String>,
    ) -> Self {
        Self {
            debt_id: display_only_unstable_id("regression-debt"),
            surface_id: surface_id.into(),
            status: RegressionDebtStatusV1::Guarded,
            description: description.into(),
            detection: detection.into(),
            guardrail_tests,
            owner: "release".into(),
            reason_codes: vec!["regression-debt-guarded".into()],
        }
    }

    pub fn blocked(
        surface_id: impl Into<String>,
        description: impl Into<String>,
        detection: impl Into<String>,
    ) -> Self {
        Self {
            debt_id: display_only_unstable_id("regression-debt"),
            surface_id: surface_id.into(),
            status: RegressionDebtStatusV1::Blocked,
            description: description.into(),
            detection: detection.into(),
            guardrail_tests: Vec::new(),
            owner: "release".into(),
            reason_codes: vec!["regression-debt-blocks-release".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegressionDebtLedgerV1 {
    pub ledger_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub items: Vec<RegressionDebtItemV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_debt_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl RegressionDebtLedgerV1 {
    pub fn new(mut items: Vec<RegressionDebtItemV1>) -> Self {
        items.sort_by(|left, right| left.surface_id.cmp(&right.surface_id));
        let blocking_debt_ids = items
            .iter()
            .filter(|item| item.status == RegressionDebtStatusV1::Blocked)
            .map(|item| item.debt_id.clone())
            .collect::<Vec<_>>();
        let reason_codes = if blocking_debt_ids.is_empty() {
            vec!["regression-debt-ledger-non-blocking".into()]
        } else {
            vec!["regression-debt-ledger-blocking".into()]
        };
        Self {
            ledger_id: display_only_unstable_id("regression-debt-ledger"),
            kind: ArtifactKindV1::RegressionDebtLedger,
            items,
            blocking_debt_ids,
            reason_codes,
            generated_at: Utc::now(),
        }
    }

    pub fn blocks_completion(&self) -> bool {
        !self.blocking_debt_ids.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CompletionAuditReportV1 {
    pub report_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub completion_state: CompletionAuditStateV1,
    pub release_bar_passed: bool,
    pub source_basis: String,
    pub gate_results: Vec<GateCommandResultV1>,
    pub release_readiness: ReleaseReadinessReportV1,
    pub traceability_matrix: CrossPassTraceabilityMatrixV1,
    pub release_artifact_manifest: ReleaseArtifactManifestV1,
    pub known_limitations: KnownLimitationsRegisterV1,
    pub regression_debt: RegressionDebtLedgerV1,
    pub partial_surfaces: Vec<String>,
    pub deferred_surfaces: Vec<String>,
    pub blocked_surfaces: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waiver_receipt_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl CompletionAuditReportV1 {
    pub fn new(
        source_basis: impl Into<String>,
        gate_results: Vec<GateCommandResultV1>,
        release_readiness: ReleaseReadinessReportV1,
        traceability_matrix: CrossPassTraceabilityMatrixV1,
        release_artifact_manifest: ReleaseArtifactManifestV1,
        known_limitations: KnownLimitationsRegisterV1,
        regression_debt: RegressionDebtLedgerV1,
    ) -> Self {
        let source_basis = source_basis.into();
        let required_common_gate_commands = [
            "cargo fmt --all --check",
            "cargo check --workspace --all-targets --all-features",
            "cargo test --workspace --all-targets --all-features",
            "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        ];
        let verifier_gate_commands = [
            "bash scripts/p24_verify.sh",
            "P24_REQUIRE_CARGO=1 bash scripts/p24_verify.sh",
            "P24_PACKAGE_SELF_REPLAY=target/p24/package/AiDENs-p24-codex-context.zip bash scripts/p24_verify.sh",
            "P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh",
        ];
        let gate_map = gate_results
            .iter()
            .map(|gate| (gate.command.as_str(), gate.is_satisfied()))
            .collect::<BTreeMap<_, _>>();
        let gates_satisfied = required_common_gate_commands
            .iter()
            .all(|command| gate_map.get(command).copied().unwrap_or(false))
            && verifier_gate_commands
                .iter()
                .any(|command| gate_map.get(command).copied().unwrap_or(false));
        let partial_surfaces = release_readiness
            .surfaces
            .iter()
            .filter(|surface| surface.state == ReleaseSurfaceStateV1::Partial)
            .map(|surface| surface.surface_id.clone())
            .chain(
                known_limitations
                    .limitations
                    .iter()
                    .filter(|limitation| limitation.state == ReleaseSurfaceStateV1::Partial)
                    .map(|limitation| limitation.surface_id.clone()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let deferred_surfaces = release_readiness
            .surfaces
            .iter()
            .filter(|surface| surface.state == ReleaseSurfaceStateV1::Deferred)
            .map(|surface| surface.surface_id.clone())
            .chain(
                known_limitations
                    .limitations
                    .iter()
                    .filter(|limitation| limitation.state == ReleaseSurfaceStateV1::Deferred)
                    .map(|limitation| limitation.surface_id.clone()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let blocked_surfaces = release_readiness
            .surfaces
            .iter()
            .filter(|surface| surface.state == ReleaseSurfaceStateV1::Blocked)
            .map(|surface| surface.surface_id.clone())
            .chain(
                known_limitations
                    .limitations
                    .iter()
                    .filter(|limitation| limitation.state == ReleaseSurfaceStateV1::Blocked)
                    .map(|limitation| limitation.surface_id.clone()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let hard_release_bar_clear = gates_satisfied
            && !release_readiness.blocks_release()
            && !traceability_matrix.blocks_completion()
            && !release_artifact_manifest.blocks_release()
            && !known_limitations.blocks_completion()
            && !regression_debt.blocks_completion()
            && blocked_surfaces.is_empty();
        let completion_state = if !hard_release_bar_clear {
            CompletionAuditStateV1::Blocked
        } else if !deferred_surfaces.is_empty() {
            CompletionAuditStateV1::DeferredHorizon
        } else if !partial_surfaces.is_empty() {
            CompletionAuditStateV1::NearComplete
        } else {
            CompletionAuditStateV1::Complete
        };
        let release_bar_passed = completion_state == CompletionAuditStateV1::Complete;
        let reason_codes = if release_bar_passed {
            vec!["release-bar-passed".into()]
        } else if completion_state == CompletionAuditStateV1::DeferredHorizon {
            vec![
                "release-bar-blocked".into(),
                "deferred-horizon-surfaces-disclosed".into(),
            ]
        } else {
            vec!["release-bar-blocked".into()]
        };
        Self {
            report_id: display_only_unstable_id("completion-audit-report"),
            kind: ArtifactKindV1::CompletionAuditReport,
            completion_state,
            release_bar_passed,
            source_basis,
            gate_results,
            release_readiness,
            traceability_matrix,
            release_artifact_manifest,
            known_limitations,
            regression_debt,
            partial_surfaces,
            deferred_surfaces,
            blocked_surfaces,
            waiver_receipt_ids: Vec::new(),
            reason_codes,
            generated_at: Utc::now(),
        }
    }

    pub fn blocks_completion(&self) -> bool {
        !self.release_bar_passed
    }
}
