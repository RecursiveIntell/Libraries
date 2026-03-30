use serde::{Deserialize, Serialize};

/// The single source of truth for all Forge runtime behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeConfig {
    /// "standard" or "sealed_local"
    #[serde(default = "default_mode")]
    pub mode: String,

    /// "auto" | "host" | "container"
    #[serde(default = "default_auto")]
    pub execution_backend_preference: String,

    /// "auto" | "docker" | "podman" | "nerdctl"
    #[serde(default = "default_auto")]
    pub container_runtime_preference: String,

    #[serde(default)]
    pub allow_test_modifications: bool,

    #[serde(default)]
    pub sealed_allow_host_backend: bool,

    #[serde(default = "default_forbidden_paths")]
    pub forbidden_paths: Vec<String>,

    #[serde(default)]
    pub caps: CapsConfig,

    #[serde(default)]
    pub mindstate: MindstateConfig,

    #[serde(default)]
    pub novelty: NoveltyConfig,

    #[serde(default)]
    pub stabilization: StabilizationConfig,

    #[serde(default)]
    pub container: ContainerConfig,

    #[serde(default)]
    pub lab: LabConfig,

    #[serde(default)]
    pub cea: CeaConfig,

    #[serde(default)]
    pub danger: DangerConfig,

    #[serde(default)]
    pub limits: ForgeLimits,

    #[serde(default)]
    pub workspace: crate::baseline::WorkspacePolicy,

    #[serde(default)]
    pub statistics: crate::experiment::StatisticsPolicy,

    #[serde(default)]
    pub comparability: crate::baseline::ComparabilityPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsConfig {
    #[serde(default = "default_max_files")]
    pub max_files_changed: usize,
    #[serde(default = "default_max_total_lines")]
    pub max_total_lines_changed: usize,
    #[serde(default = "default_max_lines_per_file")]
    pub max_lines_changed_per_file: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindstateConfig {
    #[serde(default = "default_token_budget")]
    pub token_budget: usize,
    #[serde(default = "default_evidence_budget")]
    pub evidence_budget: usize,
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoveltyConfig {
    #[serde(default = "default_delta_amp_default")]
    pub delta_amp_default: f64,
    #[serde(default = "default_delta_amp_stabilize1")]
    pub delta_amp_stabilize1: f64,
    #[serde(default = "default_delta_amp_stabilize2")]
    pub delta_amp_stabilize2: f64,
    #[serde(default = "default_delta_amp_clamp")]
    pub delta_amp_clamp: f64,
    #[serde(default = "default_orthogonality_target")]
    pub orthogonality_target: f64,
    #[serde(default = "default_min_traces")]
    pub min_traces_for_orthogonality: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilizationConfig {
    #[serde(default = "default_max_attempts")]
    pub max_attempts: usize,
    #[serde(default = "default_stabilize1_force_family")]
    pub stabilize1_force_family: String,
    #[serde(default = "default_true")]
    pub stabilize2_force_minimal_diff: bool,
    #[serde(default = "default_stabilize_weight_factor")]
    pub increase_stabilize_weight_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    #[serde(default = "default_rust_image")]
    pub rust_image: String,
    #[serde(default = "default_command_timeout")]
    pub command_timeout_secs: u64,
    #[serde(default = "default_memory_limit")]
    pub memory_limit: String,
    #[serde(default = "default_cpu_limit")]
    pub cpu_limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabConfig {
    #[serde(default = "default_batch_size")]
    pub generation_batch_size: usize,
    #[serde(default = "default_eval_parallelism")]
    pub eval_parallelism: usize,
    #[serde(default = "default_min_pass_rate")]
    pub promotion_min_suite_pass_rate: f64,
    #[serde(default = "default_min_improvement")]
    pub promotion_min_weighted_improvement: f64,
    #[serde(default)]
    pub archive: ArchiveConfig,
    #[serde(default)]
    pub allow_raw_spec: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    #[serde(default = "default_novelty_bins")]
    pub novelty_bins: Vec<NoveltyBin>,
    #[serde(default = "default_stability_variance_threshold")]
    pub stability_variance_threshold: f64,
    #[serde(default = "default_approach_families")]
    pub approach_families: Vec<String>,
    #[serde(default = "default_correctness_gate")]
    pub correctness_gate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoveltyBin {
    pub name: String,
    pub lo: f64,
    pub hi: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeaConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// MUST be false by default — requires explicit opt-in.
    #[serde(default)]
    pub enable_zero_shot: bool,
    #[serde(default = "default_zero_shot_coverage")]
    pub zero_shot_coverage_threshold: f64,
    #[serde(default = "default_risk_confidence")]
    pub risk_confidence_threshold: f64,
    #[serde(default = "default_max_line_distance")]
    pub max_line_distance_for_attribution: u32,
    #[serde(default = "default_attribution_decay")]
    pub attribution_decay_factor: f64,
    #[serde(default = "default_causal_drift_threshold")]
    pub causal_drift_warning_threshold: f64,
    #[serde(default = "default_min_runs")]
    pub min_runs_before_prediction: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DangerConfig {
    #[serde(default)]
    pub allow_semantic_memory_write: bool,
}

// --- Default value functions ---

fn default_mode() -> String {
    "standard".to_string()
}
fn default_auto() -> String {
    "auto".to_string()
}
fn default_forbidden_paths() -> Vec<String> {
    vec![
        "tests/**".to_string(),
        "**/*_test.rs".to_string(),
        "**/fixtures/**".to_string(),
        "**/*.snap".to_string(),
        "Cargo.lock".to_string(),
        ".github/**".to_string(),
    ]
}
fn default_max_files() -> usize {
    8
}
fn default_max_total_lines() -> usize {
    400
}
fn default_max_lines_per_file() -> usize {
    200
}
fn default_token_budget() -> usize {
    1800
}
fn default_evidence_budget() -> usize {
    8
}
fn default_max_steps() -> usize {
    8
}
fn default_delta_amp_default() -> f64 {
    0.7
}
fn default_delta_amp_stabilize1() -> f64 {
    0.2
}
fn default_delta_amp_stabilize2() -> f64 {
    0.1
}
fn default_delta_amp_clamp() -> f64 {
    0.0
}
fn default_orthogonality_target() -> f64 {
    0.10
}
fn default_min_traces() -> usize {
    2
}
fn default_max_attempts() -> usize {
    4
}
fn default_stabilize1_force_family() -> String {
    "mechanical".to_string()
}
fn default_true() -> bool {
    true
}
fn default_stabilize_weight_factor() -> f64 {
    2.5
}
fn default_rust_image() -> String {
    "rust:1.78-slim".to_string()
}
fn default_command_timeout() -> u64 {
    120
}
fn default_memory_limit() -> String {
    "2g".to_string()
}
fn default_cpu_limit() -> String {
    "2.0".to_string()
}
fn default_batch_size() -> usize {
    32
}
fn default_eval_parallelism() -> usize {
    4
}
fn default_min_pass_rate() -> f64 {
    0.95
}
fn default_min_improvement() -> f64 {
    0.05
}
fn default_novelty_bins() -> Vec<NoveltyBin> {
    vec![
        NoveltyBin {
            name: "low".to_string(),
            lo: 0.00,
            hi: 0.33,
        },
        NoveltyBin {
            name: "med".to_string(),
            lo: 0.33,
            hi: 0.66,
        },
        NoveltyBin {
            name: "high".to_string(),
            lo: 0.66,
            hi: 1.00,
        },
    ]
}
fn default_stability_variance_threshold() -> f64 {
    0.15
}
fn default_approach_families() -> Vec<String> {
    vec![
        "mechanical".to_string(),
        "pattern_refactor".to_string(),
        "architectural".to_string(),
        "perf".to_string(),
        "safety".to_string(),
    ]
}
fn default_correctness_gate() -> f64 {
    0.95
}
fn default_zero_shot_coverage() -> f64 {
    0.80
}
fn default_risk_confidence() -> f64 {
    0.60
}
fn default_max_line_distance() -> u32 {
    50
}
fn default_attribution_decay() -> f64 {
    10.0
}
fn default_causal_drift_threshold() -> f64 {
    0.25
}
fn default_min_runs() -> usize {
    5
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            execution_backend_preference: default_auto(),
            container_runtime_preference: default_auto(),
            allow_test_modifications: false,
            sealed_allow_host_backend: false,
            forbidden_paths: default_forbidden_paths(),
            caps: CapsConfig::default(),
            mindstate: MindstateConfig::default(),
            novelty: NoveltyConfig::default(),
            stabilization: StabilizationConfig::default(),
            container: ContainerConfig::default(),
            lab: LabConfig::default(),
            cea: CeaConfig::default(),
            danger: DangerConfig::default(),
            limits: ForgeLimits::default(),
            workspace: crate::baseline::WorkspacePolicy::default(),
            statistics: crate::experiment::StatisticsPolicy::default(),
            comparability: crate::baseline::ComparabilityPolicy::default(),
        }
    }
}

impl Default for CapsConfig {
    fn default() -> Self {
        Self {
            max_files_changed: default_max_files(),
            max_total_lines_changed: default_max_total_lines(),
            max_lines_changed_per_file: default_max_lines_per_file(),
        }
    }
}

impl Default for MindstateConfig {
    fn default() -> Self {
        Self {
            token_budget: default_token_budget(),
            evidence_budget: default_evidence_budget(),
            max_steps: default_max_steps(),
        }
    }
}

impl Default for NoveltyConfig {
    fn default() -> Self {
        Self {
            delta_amp_default: default_delta_amp_default(),
            delta_amp_stabilize1: default_delta_amp_stabilize1(),
            delta_amp_stabilize2: default_delta_amp_stabilize2(),
            delta_amp_clamp: default_delta_amp_clamp(),
            orthogonality_target: default_orthogonality_target(),
            min_traces_for_orthogonality: default_min_traces(),
        }
    }
}

impl Default for StabilizationConfig {
    fn default() -> Self {
        Self {
            max_attempts: default_max_attempts(),
            stabilize1_force_family: default_stabilize1_force_family(),
            stabilize2_force_minimal_diff: true,
            increase_stabilize_weight_factor: default_stabilize_weight_factor(),
        }
    }
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            rust_image: default_rust_image(),
            command_timeout_secs: default_command_timeout(),
            memory_limit: default_memory_limit(),
            cpu_limit: default_cpu_limit(),
        }
    }
}

impl Default for LabConfig {
    fn default() -> Self {
        Self {
            generation_batch_size: default_batch_size(),
            eval_parallelism: default_eval_parallelism(),
            promotion_min_suite_pass_rate: default_min_pass_rate(),
            promotion_min_weighted_improvement: default_min_improvement(),
            archive: ArchiveConfig::default(),
            allow_raw_spec: false,
        }
    }
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            novelty_bins: default_novelty_bins(),
            stability_variance_threshold: default_stability_variance_threshold(),
            approach_families: default_approach_families(),
            correctness_gate: default_correctness_gate(),
        }
    }
}

impl Default for CeaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            enable_zero_shot: false,
            zero_shot_coverage_threshold: default_zero_shot_coverage(),
            risk_confidence_threshold: default_risk_confidence(),
            max_line_distance_for_attribution: default_max_line_distance(),
            attribution_decay_factor: default_attribution_decay(),
            causal_drift_warning_threshold: default_causal_drift_threshold(),
            min_runs_before_prediction: default_min_runs(),
        }
    }
}

/// Resource limits for the forge runtime.
///
/// Violations are hard errors: no ExperimentEvidenceBundle is produced on limit violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeLimits {
    /// Maximum number of hypotheses per evidence bundle.
    #[serde(default = "default_max_hypotheses")]
    pub max_hypotheses: usize,
    /// Maximum number of concurrent experiments.
    #[serde(default = "default_max_experiments")]
    pub max_concurrent_experiments: usize,
    /// Maximum evidence bundles retained per candidate.
    #[serde(default = "default_max_bundles")]
    pub max_bundles_per_candidate: usize,
    /// Maximum verification steps per plan.
    #[serde(default = "default_max_verification_steps")]
    pub max_verification_steps: usize,
    /// Maximum database size in bytes (0 = unlimited).
    #[serde(default)]
    pub max_db_bytes: u64,
    /// Maximum retained log bytes per run (0 = unlimited).
    #[serde(default)]
    pub max_retained_logs_bytes: u64,
    /// Maximum retained artifacts count.
    #[serde(default = "default_max_artifacts")]
    pub max_retained_artifacts: usize,
    /// Failed run retention in days (0 = no retention).
    #[serde(default = "default_failure_retention_days")]
    pub failed_run_retention_days: u32,
    /// Maximum files in a patch (enforced at patch ingest).
    #[serde(default = "default_max_patch_files")]
    pub max_patch_files: usize,
    /// Maximum total bytes in a patch (enforced at patch ingest).
    #[serde(default = "default_max_patch_bytes")]
    pub max_patch_bytes: u64,
    /// Maximum runtime in seconds for a single check (enforced in LocalExperimentRunner).
    #[serde(default = "default_max_check_runtime")]
    pub max_check_runtime_secs: u64,
    /// Maximum output bytes from a single check (truncated if exceeded).
    #[serde(default = "default_max_check_output_bytes")]
    pub max_check_output_bytes: u64,
    /// Maximum nodes in the CEA graph before mutation commit.
    #[serde(default = "default_max_graph_nodes")]
    pub max_graph_nodes: usize,
    /// Maximum edges in the CEA graph before mutation commit.
    #[serde(default = "default_max_graph_edges")]
    pub max_graph_edges: usize,
}

fn default_max_hypotheses() -> usize {
    100
}
fn default_max_experiments() -> usize {
    4
}
fn default_max_bundles() -> usize {
    50
}
fn default_max_verification_steps() -> usize {
    20
}
fn default_max_artifacts() -> usize {
    100
}
fn default_failure_retention_days() -> u32 {
    30
}
fn default_max_patch_files() -> usize {
    16
}
fn default_max_patch_bytes() -> u64 {
    1_000_000 // 1 MB
}
fn default_max_check_runtime() -> u64 {
    300 // 5 minutes
}
fn default_max_check_output_bytes() -> u64 {
    10_000_000 // 10 MB
}
fn default_max_graph_nodes() -> usize {
    10_000
}
fn default_max_graph_edges() -> usize {
    50_000
}

impl Default for ForgeLimits {
    fn default() -> Self {
        Self {
            max_hypotheses: default_max_hypotheses(),
            max_concurrent_experiments: default_max_experiments(),
            max_bundles_per_candidate: default_max_bundles(),
            max_verification_steps: default_max_verification_steps(),
            max_db_bytes: 0,
            max_retained_logs_bytes: 0,
            max_retained_artifacts: default_max_artifacts(),
            failed_run_retention_days: default_failure_retention_days(),
            max_patch_files: default_max_patch_files(),
            max_patch_bytes: default_max_patch_bytes(),
            max_check_runtime_secs: default_max_check_runtime(),
            max_check_output_bytes: default_max_check_output_bytes(),
            max_graph_nodes: default_max_graph_nodes(),
            max_graph_edges: default_max_graph_edges(),
        }
    }
}
