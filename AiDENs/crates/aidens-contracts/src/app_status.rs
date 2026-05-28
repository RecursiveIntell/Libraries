use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AiDENsAppPlanV1 {
    pub app_id: String,
    pub profile_id: String,
    pub provider_required: bool,
    pub memory_mode: MemoryModeV1,
    pub receipt_level: ReportLevelV1,
    pub dangerous_auto_approval: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risk_disclosures: Vec<RiskDisclosureV1>,
    pub enabled_tool_bundles: Vec<String>,
    pub disabled_tool_bundles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AiDENsCompiledPlanV1 {
    pub plan_id: ArtifactId,
    pub plan: AiDENsAppPlanV1,
    pub provider_route: ProviderRouteReportV1,
    pub tool_exposure: ToolExposureSetV1,
    pub doctor: AiDENsDoctorReportV1,
    pub config_apply_receipt: ConfigApplyReportV1,
    pub parity_report: PlanRuntimeParityReportV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AiDENsDoctorReportV1 {
    pub report_id: ArtifactId,
    pub app_id: String,
    pub sections: BTreeMap<String, Vec<RuntimeCapabilityTruthV1>>,
}

impl AiDENsDoctorReportV1 {
    pub fn new(
        app_id: impl Into<String>,
        sections: BTreeMap<String, Vec<RuntimeCapabilityTruthV1>>,
    ) -> Self {
        Self {
            report_id: display_only_unstable_id("doctor-report"),
            app_id: app_id.into(),
            sections,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ApiHonestyOutcomeV1 {
    Honored,
    Rejected,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApiHonestyReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub surface: String,
    pub accepted_inputs: Vec<String>,
    pub honored_inputs: Vec<String>,
    pub rejected_inputs: Vec<String>,
    pub outcome: ApiHonestyOutcomeV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub checked_at: DateTime<Utc>,
}

impl ApiHonestyReportV1 {
    pub fn honored(
        surface: impl Into<String>,
        accepted_inputs: Vec<String>,
        honored_inputs: Vec<String>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("api-honesty"),
            kind: ArtifactKindV1::ApiHonesty,
            surface: surface.into(),
            accepted_inputs,
            honored_inputs,
            rejected_inputs: Vec::new(),
            outcome: ApiHonestyOutcomeV1::Honored,
            reason_codes: Vec::new(),
            checked_at: Utc::now(),
        }
    }

    pub fn rejected(
        surface: impl Into<String>,
        accepted_inputs: Vec<String>,
        rejected_inputs: Vec<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("api-honesty"),
            kind: ArtifactKindV1::ApiHonesty,
            surface: surface.into(),
            accepted_inputs,
            honored_inputs: Vec::new(),
            rejected_inputs,
            outcome: ApiHonestyOutcomeV1::Rejected,
            reason_codes: vec![reason.into()],
            checked_at: Utc::now(),
        }
    }

    pub fn blocked(
        surface: impl Into<String>,
        accepted_inputs: Vec<String>,
        rejected_inputs: Vec<String>,
        reason: impl Into<String>,
    ) -> Self {
        let mut receipt = Self::rejected(surface, accepted_inputs, rejected_inputs, reason);
        receipt.outcome = ApiHonestyOutcomeV1::Blocked;
        receipt
    }

    pub fn all_inputs_honored(&self) -> bool {
        self.outcome == ApiHonestyOutcomeV1::Honored
            && self
                .accepted_inputs
                .iter()
                .all(|input| self.honored_inputs.contains(input))
            && self.rejected_inputs.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigApplyReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub app_id: String,
    pub config_source: String,
    pub provider_route: ProviderRouteReportV1,
    pub tool_exposure: ToolExposureSetV1,
    pub memory_mode: MemoryModeV1,
    pub receipt_level: ReportLevelV1,
    pub enabled_tool_bundles: Vec<String>,
    pub sandbox_root: Option<String>,
    pub applied: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub applied_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConfigApplyReportDraftV1 {
    pub app_id: String,
    pub config_source: String,
    pub provider_route: ProviderRouteReportV1,
    pub tool_exposure: ToolExposureSetV1,
    pub memory_mode: MemoryModeV1,
    pub receipt_level: ReportLevelV1,
    pub enabled_tool_bundles: Vec<String>,
    pub sandbox_root: Option<String>,
    pub applied: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ConfigApplyReportV1 {
    pub fn new(draft: ConfigApplyReportDraftV1) -> Self {
        Self {
            receipt_id: display_only_unstable_id("config-apply"),
            kind: ArtifactKindV1::ConfigApply,
            app_id: draft.app_id,
            config_source: draft.config_source,
            provider_route: draft.provider_route,
            tool_exposure: draft.tool_exposure,
            memory_mode: draft.memory_mode,
            receipt_level: draft.receipt_level,
            enabled_tool_bundles: draft.enabled_tool_bundles,
            sandbox_root: draft.sandbox_root,
            applied: draft.applied,
            reason_codes: draft.reason_codes,
            applied_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PlanRuntimeParityCheckKindV1 {
    ProviderRoute,
    ToolExposure,
    MemoryMode,
    ScaffoldState,
}

impl fmt::Display for PlanRuntimeParityCheckKindV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ProviderRoute => "provider-route",
            Self::ToolExposure => "tool-exposure",
            Self::MemoryMode => "memory-mode",
            Self::ScaffoldState => "scaffold-state",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PlanRuntimeParityCheckV1 {
    pub check: PlanRuntimeParityCheckKindV1,
    pub expected: String,
    pub observed: String,
    pub passed: bool,
}

impl PlanRuntimeParityCheckV1 {
    pub fn new(
        check: PlanRuntimeParityCheckKindV1,
        expected: impl Into<String>,
        observed: impl Into<String>,
    ) -> Self {
        let expected = expected.into();
        let observed = observed.into();
        Self {
            check,
            passed: expected == observed,
            expected,
            observed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PlanRuntimeParityReportV1 {
    pub report_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub app_id: String,
    pub checks: Vec<PlanRuntimeParityCheckV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mismatches: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl PlanRuntimeParityReportV1 {
    pub fn new(app_id: impl Into<String>, checks: Vec<PlanRuntimeParityCheckV1>) -> Self {
        let mismatches = checks
            .iter()
            .filter(|check| !check.passed)
            .map(|check| {
                format!(
                    "{} expected '{}' but observed '{}'",
                    check.check, check.expected, check.observed
                )
            })
            .collect();
        Self {
            report_id: display_only_unstable_id("plan-runtime-parity"),
            kind: ArtifactKindV1::PlanRuntimeParity,
            app_id: app_id.into(),
            checks,
            mismatches,
            generated_at: Utc::now(),
        }
    }

    pub fn is_passing(&self) -> bool {
        self.mismatches.is_empty() && self.checks.iter().all(|check| check.passed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CrateImplementationStatusV1 {
    Implemented,
    Partial,
    ScaffoldOnly,
}

impl fmt::Display for CrateImplementationStatusV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Implemented => "implemented",
            Self::Partial => "partial",
            Self::ScaffoldOnly => "scaffold-only",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceBasisLockV1 {
    pub lock_id: ArtifactId,
    pub snapshot_date: String,
    pub source_archive: String,
    pub research_archive: String,
    pub extraction_root: String,
    pub workspace_crates: u32,
    pub rust_files: u32,
    pub approximate_rust_loc: u32,
    pub scaffold_only_files: u32,
    pub research_files: u32,
    pub locked_at: DateTime<Utc>,
}

impl SourceBasisLockV1 {
    pub fn current_20260426() -> Self {
        Self {
            lock_id: display_only_unstable_id("source-basis-lock"),
            snapshot_date: "2026-04-26".into(),
            source_archive: "libraries-source-clean-20260426.zip".into(),
            research_archive: "Full Provenance+ Research 4/24/26.zip".into(),
            extraction_root: "/mnt/data/aidens_20260426".into(),
            workspace_crates: 31,
            rust_files: 37,
            approximate_rust_loc: 5126,
            scaffold_only_files: 15,
            research_files: 52,
            locked_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CrateSurfaceStatusV1 {
    pub crate_name: String,
    pub status: CrateImplementationStatusV1,
    pub rust_files: u32,
    pub approximate_rust_loc: u32,
    pub note: String,
}

impl CrateSurfaceStatusV1 {
    pub fn new(
        crate_name: impl Into<String>,
        status: CrateImplementationStatusV1,
        rust_files: u32,
        approximate_rust_loc: u32,
        note: impl Into<String>,
    ) -> Self {
        Self {
            crate_name: crate_name.into(),
            status,
            rust_files,
            approximate_rust_loc,
            note: note.into(),
        }
    }

    pub fn allows_scaffold_marker(&self) -> bool {
        self.status == CrateImplementationStatusV1::ScaffoldOnly
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ScaffoldSurfaceReportV1 {
    pub report_id: ArtifactId,
    pub generated_at: DateTime<Utc>,
    pub crates: Vec<CrateSurfaceStatusV1>,
}

impl ScaffoldSurfaceReportV1 {
    pub fn new(crates: Vec<CrateSurfaceStatusV1>) -> Self {
        Self {
            report_id: display_only_unstable_id("scaffold-surface-report"),
            generated_at: Utc::now(),
            crates,
        }
    }

    pub fn scaffold_only_crates(&self) -> Vec<&str> {
        self.crates
            .iter()
            .filter(|surface| surface.status == CrateImplementationStatusV1::ScaffoldOnly)
            .map(|surface| surface.crate_name.as_str())
            .collect()
    }

    pub fn allows_scaffold_marker_for(&self, crate_name: &str) -> bool {
        self.crates
            .iter()
            .find(|surface| surface.crate_name == crate_name)
            .is_some_and(CrateSurfaceStatusV1::allows_scaffold_marker)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FakeReadyFindingV1 {
    pub finding_id: ArtifactId,
    pub surface: String,
    pub pattern: String,
    pub reason: String,
    pub blocked: bool,
    pub found_at: DateTime<Utc>,
}

impl FakeReadyFindingV1 {
    pub fn blocking(
        surface: impl Into<String>,
        pattern: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            finding_id: display_only_unstable_id("fake-ready-finding"),
            surface: surface.into(),
            pattern: pattern.into(),
            reason: reason.into(),
            blocked: true,
            found_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SuperPassDispositionV1 {
    Pending,
    InProgress,
    Done,
    Blocked,
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SuperPassStatusV1 {
    pub status_id: ArtifactId,
    pub current_pass: String,
    pub disposition: SuperPassDispositionV1,
    pub source_basis: SourceBasisLockV1,
    pub scaffold_report: ScaffoldSurfaceReportV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fake_ready_findings: Vec<FakeReadyFindingV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

impl SuperPassStatusV1 {
    pub fn new(
        current_pass: impl Into<String>,
        disposition: SuperPassDispositionV1,
        source_basis: SourceBasisLockV1,
        scaffold_report: ScaffoldSurfaceReportV1,
        fake_ready_findings: Vec<FakeReadyFindingV1>,
        blockers: Vec<String>,
    ) -> Self {
        Self {
            status_id: display_only_unstable_id("super-pass-status"),
            current_pass: current_pass.into(),
            disposition,
            source_basis,
            scaffold_report,
            fake_ready_findings,
            blockers,
            updated_at: Utc::now(),
        }
    }

    pub fn is_blocked(&self) -> bool {
        self.disposition == SuperPassDispositionV1::Blocked
            || !self.blockers.is_empty()
            || self
                .fake_ready_findings
                .iter()
                .any(|finding| finding.blocked)
    }
}

impl AiDENsAppPlanV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.app_id.trim().is_empty() {
            return Err("app_id must not be empty".into());
        }
        if self.dangerous_auto_approval {
            return Err("dangerous_auto_approval requires explicit advanced override".into());
        }
        Ok(())
    }

    pub fn human_summary(&self) -> String {
        format!(
            "{} [{}]: provider_required={}, memory_mode={}, receipt_level={}",
            self.app_id,
            self.profile_id,
            self.provider_required,
            self.memory_mode,
            self.receipt_level
        )
    }

    pub fn risk_summary(&self) -> String {
        if self.risk_disclosures.is_empty() {
            return "No risky capabilities are granted by default.".into();
        }
        self.risk_disclosures
            .iter()
            .map(|risk| {
                format!(
                    "{}: granted_by_default={}, permit_required={} ({})",
                    risk.risk_class, risk.granted_by_default, risk.permit_required, risk.reason
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RiskDisclosureV1 {
    pub risk_class: CanonicalToolSideEffectClass,
    pub granted_by_default: bool,
    pub permit_required: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MemoryModeV1 {
    Disabled,
    Optional,
    Required,
}

impl fmt::Display for MemoryModeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Disabled => "disabled",
            Self::Optional => "optional",
            Self::Required => "required",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReportLevelV1 {
    Minimal,
    Standard,
    Full,
}

impl fmt::Display for ReportLevelV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Minimal => "minimal",
            Self::Standard => "standard",
            Self::Full => "full",
        })
    }
}
