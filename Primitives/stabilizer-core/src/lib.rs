//! # stabilizer-core
//!
//! Attempt-phase and delta-policy primitives for `forge-engine`.
//!
//! This crate owns the bounded policy objects that step an attempt through
//! innovate/stabilize/clamp phases. It does not execute patches, run checks, or
//! decide archive/promotion outcomes.

//! # stabilizer-core
//!
//! Attempt-phase and novelty helpers for iterative patch generation.
//!
//! This Tier 3 crate models retry/stabilization phases and derives patch-level
//! strategy signals. It does not own execution, scoring, or persistence.

use std::collections::BTreeSet;

use typed_patch::{EditOp, FileMode, StructuredPatch};

pub type StrategyTag = String;

#[derive(Debug, thiserror::Error)]
pub enum StabilizerError {
    #[error("stabilizer: all attempts exhausted")]
    Exhausted,
}

#[derive(Debug, Clone)]
pub struct StabilizerConfig {
    pub force_family: String,
    pub force_minimal_diff: bool,
    pub stabilize_weight_factor: f64,
}

#[derive(Debug, Clone)]
pub struct DeltaPolicy {
    pub delta_amp_default: f64,
    pub delta_amp_stabilize1: f64,
    pub delta_amp_stabilize2: f64,
    pub delta_amp_clamp: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AttemptPhase {
    Innovative = 0,
    Stabilize1 = 1,
    Stabilize2 = 2,
    Clamp = 3,
}

#[derive(Debug, Clone)]
pub struct AttemptOverrides {
    pub phase: AttemptPhase,
    pub delta_amplitude: f64,
    pub force_family: Option<String>,
    pub force_minimal_diff: bool,
    pub weight_factor: f64,
}

#[derive(Debug, Clone)]
pub struct Stabilizer {
    delta_policy: DeltaPolicy,
    current_phase_index: usize,
    config: StabilizerConfig,
}

impl DeltaPolicy {
    pub fn new(
        delta_amp_default: f64,
        delta_amp_stabilize1: f64,
        delta_amp_stabilize2: f64,
        delta_amp_clamp: f64,
    ) -> Self {
        Self {
            delta_amp_default,
            delta_amp_stabilize1,
            delta_amp_stabilize2,
            delta_amp_clamp,
        }
    }

    pub fn amplitude_for_phase(&self, phase: AttemptPhase) -> f64 {
        match phase {
            AttemptPhase::Innovative => self.delta_amp_default,
            AttemptPhase::Stabilize1 => self.delta_amp_stabilize1,
            AttemptPhase::Stabilize2 => self.delta_amp_stabilize2,
            AttemptPhase::Clamp => self.delta_amp_clamp,
        }
    }
}

impl AttemptPhase {
    pub fn all() -> &'static [AttemptPhase] {
        &[
            AttemptPhase::Innovative,
            AttemptPhase::Stabilize1,
            AttemptPhase::Stabilize2,
            AttemptPhase::Clamp,
        ]
    }
}

impl Stabilizer {
    pub fn new(delta_policy: DeltaPolicy, config: StabilizerConfig) -> Self {
        Self {
            delta_policy,
            current_phase_index: 0,
            config,
        }
    }

    pub fn next_attempt(&mut self) -> Result<AttemptOverrides, StabilizerError> {
        let phases = AttemptPhase::all();
        if self.current_phase_index >= phases.len() {
            return Err(StabilizerError::Exhausted);
        }

        let phase = phases[self.current_phase_index];
        let delta_amplitude = self.delta_policy.amplitude_for_phase(phase);
        let force_family = match phase {
            AttemptPhase::Stabilize1 | AttemptPhase::Stabilize2 => {
                Some(self.config.force_family.clone())
            }
            _ => None,
        };
        let force_minimal_diff = matches!(phase, AttemptPhase::Stabilize2 | AttemptPhase::Clamp)
            && self.config.force_minimal_diff;
        let weight_factor = match phase {
            AttemptPhase::Stabilize1 | AttemptPhase::Stabilize2 => {
                self.config.stabilize_weight_factor
            }
            _ => 1.0,
        };

        self.current_phase_index += 1;

        Ok(AttemptOverrides {
            phase,
            delta_amplitude,
            force_family,
            force_minimal_diff,
            weight_factor,
        })
    }

    pub fn has_next(&self) -> bool {
        self.current_phase_index < AttemptPhase::all().len()
    }

    pub fn reset(&mut self) {
        self.current_phase_index = 0;
    }

    pub fn current_index(&self) -> usize {
        self.current_phase_index
    }
}

pub fn extract_strategy_tags(patch: &StructuredPatch) -> Vec<StrategyTag> {
    let mut tags = BTreeSet::new();

    let file_count = patch.edits.len();
    if file_count == 1 {
        tags.insert("single_file".to_string());
    }
    if file_count > 3 {
        tags.insert("multi_file".to_string());
    }

    let mut insert_count = 0_usize;
    let mut replace_count = 0_usize;
    let mut delete_count = 0_usize;

    for edit in &patch.edits {
        for op in &edit.ops {
            match op {
                EditOp::Insert { .. } => insert_count += 1,
                EditOp::Replace { .. } => replace_count += 1,
                EditOp::Delete { .. } => delete_count += 1,
            }
        }
    }

    if replace_count > insert_count + delete_count {
        tags.insert("replace_heavy".to_string());
    }
    if insert_count > replace_count * 2 {
        tags.insert("insert_heavy".to_string());
    }
    if delete_count > 0 && insert_count == 0 {
        tags.insert("deletion_only".to_string());
    }

    for edit in &patch.edits {
        if edit.mode == Some(FileMode::Create) {
            tags.insert("new_file".to_string());
        }
        if edit.mode == Some(FileMode::Delete) {
            tags.insert("file_deletion".to_string());
        }

        for op in &edit.ops {
            let context = op.context_lines().join(" ");
            if context.contains("fn ") {
                tags.insert("fn_level_edit".to_string());
            }
            if context.contains("impl ") {
                tags.insert("impl_level_edit".to_string());
            }
            if context.contains("trait ") {
                tags.insert("trait_level_edit".to_string());
            }
            if context.contains("mod ") {
                tags.insert("mod_level_edit".to_string());
            }
            if context.contains("macro_rules!") {
                tags.insert("macro_level_edit".to_string());
            }
            if context.contains("async ") {
                tags.insert("async_boundary".to_string());
            }
            if context.contains("-> Result") {
                tags.insert("error_type_edit".to_string());
            }
        }
    }

    if tags.contains("new_file")
        && (tags.contains("fn_level_edit") || tags.contains("impl_level_edit"))
    {
        tags.insert("module_split".to_string());
    }

    tags.into_iter().collect()
}

pub fn compute_tag_novelty(current_tags: &[String], recent_tags_union: &BTreeSet<String>) -> f64 {
    if recent_tags_union.is_empty() {
        return 1.0;
    }

    let current_set: BTreeSet<&str> = current_tags.iter().map(|tag| tag.as_str()).collect();
    let recent_set: BTreeSet<&str> = recent_tags_union.iter().map(|tag| tag.as_str()).collect();

    let intersection = current_set.intersection(&recent_set).count();
    let union = current_set.union(&recent_set).count();
    if union == 0 {
        return 1.0;
    }

    1.0 - (intersection as f64 / union as f64)
}

pub fn determine_approach_family(tags: &[String]) -> &'static str {
    let has = |tag: &str| tags.iter().any(|value| value == tag);

    if (has("replace_heavy") || (has("fn_level_edit") && tags.len() <= 2) || has("single_file"))
        && (has("replace_heavy") || has("fn_level_edit"))
    {
        return "mechanical";
    }

    if has("extract_function") || has("introduce_trait") || has("module_split") {
        return "pattern_refactor";
    }

    if has("new_file") || has("trait_level_edit") || has("multi_file") {
        return "architectural";
    }

    if tags.iter().any(|tag| tag.contains("perf")) || (has("async_boundary") && has("multi_file")) {
        return "perf";
    }

    if has("error_type_edit") || has("macro_level_edit") {
        return "safety";
    }

    "mechanical"
}

#[cfg(test)]
mod wave1_tests {
    use super::*;
    use typed_patch::{Anchor, EditOp, FileEdit, FileMode, StructuredPatch};

    fn sample_patch() -> StructuredPatch {
        StructuredPatch {
            patch_id: uuid::Uuid::new_v4(),
            summary: "add helper".into(),
            edits: vec![FileEdit {
                path: "src/helper.rs".into(),
                mode: Some(FileMode::Create),
                ops: vec![EditOp::Insert {
                    anchor: Anchor::AfterLine {
                        line: 0,
                        context_before: vec!["fn helper() {".into()],
                        context_after: vec!["}".into()],
                    },
                    lines: vec!["    println!(\"ok\");".into()],
                }],
            }],
            notes: vec![],
        }
    }

    #[test]
    fn next_attempt_advances_through_all_phases_then_exhausts() {
        let mut stabilizer = Stabilizer::new(
            DeltaPolicy::new(1.0, 0.5, 0.25, 0.1),
            StabilizerConfig {
                force_family: "mechanical".into(),
                force_minimal_diff: true,
                stabilize_weight_factor: 0.8,
            },
        );

        let innovative = stabilizer.next_attempt().unwrap();
        assert_eq!(innovative.phase, AttemptPhase::Innovative);
        assert_eq!(innovative.delta_amplitude, 1.0);
        assert!(innovative.force_family.is_none());

        let stabilize1 = stabilizer.next_attempt().unwrap();
        assert_eq!(stabilize1.phase, AttemptPhase::Stabilize1);
        assert_eq!(stabilize1.force_family.as_deref(), Some("mechanical"));
        assert!(!stabilize1.force_minimal_diff);

        let stabilize2 = stabilizer.next_attempt().unwrap();
        assert_eq!(stabilize2.phase, AttemptPhase::Stabilize2);
        assert!(stabilize2.force_minimal_diff);

        let clamp = stabilizer.next_attempt().unwrap();
        assert_eq!(clamp.phase, AttemptPhase::Clamp);
        assert!(clamp.force_minimal_diff);

        assert!(matches!(
            stabilizer.next_attempt(),
            Err(StabilizerError::Exhausted)
        ));
    }

    #[test]
    fn strategy_tags_capture_patch_shape_and_family() {
        let patch = sample_patch();
        let tags = extract_strategy_tags(&patch);

        assert!(tags.contains(&"single_file".to_string()));
        assert!(tags.contains(&"new_file".to_string()));
        assert!(tags.contains(&"fn_level_edit".to_string()));
        assert!(tags.contains(&"module_split".to_string()));
        assert_eq!(determine_approach_family(&tags), "mechanical");

        let recent = BTreeSet::from(["new_file".to_string(), "single_file".to_string()]);
        let novelty = compute_tag_novelty(&tags, &recent);
        assert!(novelty > 0.0);
        assert!(novelty < 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typed_patch::{Anchor, FileEdit};

    fn sample_patch() -> StructuredPatch {
        StructuredPatch {
            patch_id: uuid::Uuid::nil(),
            summary: "add helper".into(),
            edits: vec![FileEdit {
                path: "src/helper.rs".into(),
                mode: Some(FileMode::Create),
                ops: vec![EditOp::Insert {
                    anchor: Anchor::AfterLine {
                        line: 1,
                        context_before: vec!["fn helper() {".into()],
                        context_after: vec!["}".into()],
                    },
                    lines: vec!["    println!(\"hi\");".into()],
                }],
            }],
            notes: vec![],
        }
    }

    fn sample_stabilizer() -> Stabilizer {
        Stabilizer::new(
            DeltaPolicy::new(1.0, 0.5, 0.25, 0.1),
            StabilizerConfig {
                force_family: "mechanical".into(),
                force_minimal_diff: true,
                stabilize_weight_factor: 0.7,
            },
        )
    }

    #[test]
    fn next_attempt_progresses_through_all_phases_then_exhausts() {
        let mut stabilizer = sample_stabilizer();

        let innovative = stabilizer.next_attempt().unwrap();
        assert_eq!(innovative.phase, AttemptPhase::Innovative);
        assert_eq!(innovative.delta_amplitude, 1.0);
        assert_eq!(innovative.force_family, None);
        assert!(!innovative.force_minimal_diff);
        assert_eq!(innovative.weight_factor, 1.0);

        let stabilize1 = stabilizer.next_attempt().unwrap();
        assert_eq!(stabilize1.phase, AttemptPhase::Stabilize1);
        assert_eq!(stabilize1.delta_amplitude, 0.5);
        assert_eq!(stabilize1.force_family.as_deref(), Some("mechanical"));
        assert!(!stabilize1.force_minimal_diff);
        assert_eq!(stabilize1.weight_factor, 0.7);

        let stabilize2 = stabilizer.next_attempt().unwrap();
        assert_eq!(stabilize2.phase, AttemptPhase::Stabilize2);
        assert_eq!(stabilize2.delta_amplitude, 0.25);
        assert_eq!(stabilize2.force_family.as_deref(), Some("mechanical"));
        assert!(stabilize2.force_minimal_diff);
        assert_eq!(stabilize2.weight_factor, 0.7);

        let clamp = stabilizer.next_attempt().unwrap();
        assert_eq!(clamp.phase, AttemptPhase::Clamp);
        assert_eq!(clamp.delta_amplitude, 0.1);
        assert_eq!(clamp.force_family, None);
        assert!(clamp.force_minimal_diff);
        assert_eq!(clamp.weight_factor, 1.0);

        assert!(!stabilizer.has_next());
        assert!(matches!(
            stabilizer.next_attempt(),
            Err(StabilizerError::Exhausted)
        ));
    }

    #[test]
    fn reset_rewinds_phase_sequence() {
        let mut stabilizer = sample_stabilizer();
        let _ = stabilizer.next_attempt().unwrap();
        let _ = stabilizer.next_attempt().unwrap();
        assert_eq!(stabilizer.current_index(), 2);

        stabilizer.reset();

        assert_eq!(stabilizer.current_index(), 0);
        assert_eq!(
            stabilizer.next_attempt().unwrap().phase,
            AttemptPhase::Innovative
        );
    }

    #[test]
    fn extract_strategy_tags_detects_patch_shape_and_context() {
        let tags = extract_strategy_tags(&sample_patch());

        assert!(tags.iter().any(|tag| tag == "single_file"));
        assert!(tags.iter().any(|tag| tag == "new_file"));
        assert!(tags.iter().any(|tag| tag == "fn_level_edit"));
        assert!(tags.iter().any(|tag| tag == "module_split"));
    }

    #[test]
    fn novelty_and_approach_family_follow_tag_overlap() {
        let novelty = compute_tag_novelty(
            &["replace_heavy".into(), "single_file".into()],
            &BTreeSet::from(["single_file".into(), "async_boundary".into()]),
        );
        assert!(novelty > 0.0 && novelty < 1.0);

        assert_eq!(
            determine_approach_family(&["replace_heavy".into(), "fn_level_edit".into()]),
            "mechanical"
        );
        assert_eq!(
            determine_approach_family(&["extract_function".into(), "multi_file".into()]),
            "pattern_refactor"
        );
        assert_eq!(
            determine_approach_family(&["trait_level_edit".into(), "new_file".into()]),
            "architectural"
        );
        assert_eq!(
            determine_approach_family(&["error_type_edit".into()]),
            "safety"
        );
    }
}
