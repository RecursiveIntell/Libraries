//! Configuration types for the forge-pilot OODA loop.
//!
//! `LoopConfig` carries all tuning knobs, scope identity, workspace paths,
//! runtime settings, and operator-provided patch plan seeds.

use forge_engine::{ExperimentConfig, ForgeConfig, StructuredPatch};
use knowledge_runtime::{RuntimeConfig, Scope};
use verification_policy::{ApprovalRecord, PolicySnapshot};

/// An operator-provided patch plan seed that overrides automatic plan selection.
#[derive(Debug, Clone)]
pub struct PatchPlanSeed {
    pub target_key: String,
    pub fixture_path: String,
    pub patch: StructuredPatch,
    pub experiment_config: ExperimentConfig,
    pub description: String,
}

/// Full configuration for an OODA loop run.
///
/// Includes budget limits, scope identity, workspace paths, runtime and
/// Forge engine settings, policy snapshots, and optional patch seeds.
#[derive(Debug, Clone)]
pub struct LoopConfig {
    pub max_iterations: u32,
    pub time_budget_secs: u64,
    pub cooldown_secs: u64,
    pub halt_urgency_threshold: f64,
    pub max_retries_per_target: u32,
    pub allow_advisory_only_steps: bool,
    pub scope: Scope,
    pub workspace_path: String,
    pub memory_dir: String,
    pub forge_db_path: String,
    pub include_hyperedges: bool,
    pub oracle_max_iterations: u32,
    pub minimal_perturbation_budget: usize,
    pub observation_import_limit: usize,
    pub observation_projection_limit: usize,
    pub runtime_config: RuntimeConfig,
    pub forge_config: ForgeConfig,
    pub patch_plan_seeds: Vec<PatchPlanSeed>,
    pub policy_snapshots: Vec<PolicySnapshot>,
    pub approval_records: Vec<ApprovalRecord>,
    pub calibration_abstention_threshold_micros: u64,
}

impl LoopConfig {
    /// Builds the default loop configuration for a scope and workspace path.
    pub fn default_for_scope(scope: Scope, workspace_path: impl Into<String>) -> Self {
        Self {
            scope: scope.clone(),
            workspace_path: workspace_path.into(),
            runtime_config: RuntimeConfig {
                default_scope: scope,
                ..RuntimeConfig {
                    default_scope: Scope::new("default"),
                    query: Default::default(),
                    entity: Default::default(),
                    projection: Default::default(),
                    strict_temporal: false,
                    strict_scope: false,
                }
            },
            ..Self::default()
        }
    }

    /// Returns the configured patch seed for a target key, if one exists.
    pub fn patch_seed_for(&self, target_key: &str) -> Option<&PatchPlanSeed> {
        self.patch_plan_seeds
            .iter()
            .find(|seed| seed.target_key == target_key)
            .or_else(|| {
                let target_family = patch_seed_target_family(target_key)?;
                self.patch_plan_seeds.iter().find(|seed| {
                    patch_seed_target_family(seed.target_key.as_str()) == Some(target_family)
                })
            })
    }
}

fn patch_seed_target_family(target_key: &str) -> Option<&str> {
    target_key.split_once(':').map(|(family, _)| family)
}

impl Default for LoopConfig {
    fn default() -> Self {
        let default_scope = Scope::new("default");
        Self {
            max_iterations: 4,
            time_budget_secs: 30,
            cooldown_secs: 0,
            halt_urgency_threshold: 0.25,
            max_retries_per_target: 2,
            allow_advisory_only_steps: false,
            scope: default_scope.clone(),
            workspace_path: ".".into(),
            memory_dir: "./memory".into(),
            forge_db_path: "./forge.db".into(),
            include_hyperedges: true,
            oracle_max_iterations: 4,
            minimal_perturbation_budget: 2,
            observation_import_limit: 16,
            observation_projection_limit: 128,
            runtime_config: RuntimeConfig {
                default_scope,
                query: Default::default(),
                entity: Default::default(),
                projection: Default::default(),
                strict_temporal: false,
                strict_scope: false,
            },
            forge_config: ForgeConfig::default(),
            patch_plan_seeds: Vec::new(),
            policy_snapshots: vec![PolicySnapshot::permissive(
                "forge-pilot.default",
                "2026-03-12T00:00:00Z",
            )],
            approval_records: Vec::new(),
            calibration_abstention_threshold_micros: 500_000,
        }
    }
}
