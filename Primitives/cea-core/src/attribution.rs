use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use check_runner::{CheckResult, EffectSignature, ParsedCheckOutput};
use typed_patch::{Anchor, EditOp, LineAttributionMap, StructuredPatch};

use crate::error::CeaCoreError;
use crate::scope::infer_scope;
use crate::types::{
    AnchorKind, EditOpKind, EditOpSignature, EvidenceKind, FileIndex, ObservationIdentity, OpIndex,
};

const RUN_HASH_VERSION: &[u8] = b"cea-core:run-hash:v2";
const IDENTIFIED_RUN_HASH_VERSION: &[u8] = b"cea-core:run-hash:v3";
const OBSERVATION_KEY_VERSION: &[u8] = b"cea-core:observation-key:v1";
const CAUSE_HASH_VERSION: &[u8] = b"cea-core:cause:v2";
const EFFECT_HASH_VERSION: &[u8] = b"cea-core:effect:v2";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AttributionTriple {
    /// The attributed edit operation.
    pub cause: EditOpSignature,
    /// The observed effect emitted by a checker.
    pub effect: EffectSignature,
    /// Absolute line distance used during candidate scoring.
    pub distance: i32,
    /// Fractional contribution for this cause/effect pair. Weights are normalized per effect.
    pub weight: f64,
}

#[derive(Debug, Clone)]
pub struct AttributedRunResult {
    pub triples: Vec<AttributionTriple>,
    pub check_result: CheckResult,
    pub run_hash: String,
    /// Stable idempotency key derived only from execution identity.
    ///
    /// The run hash still binds observed content for integrity. Replays with
    /// conflicting content but the same observation identity must not update
    /// learning twice.
    pub observation_key: Option<String>,
    /// Typed provenance retained so storage can enforce the telemetry boundary.
    pub evidence_kind: EvidenceKind,
}

impl AttributedRunResult {
    pub fn new(triples: Vec<AttributionTriple>, check_result: CheckResult) -> Self {
        let mut run = Self {
            triples,
            check_result,
            run_hash: String::new(),
            observation_key: None,
            evidence_kind: EvidenceKind::Observational,
        };
        run.run_hash = compute_run_hash(&run);
        run
    }

    /// Construct a run whose hash is bound to its independently executed
    /// observation identity and evidence grade.
    pub fn with_observation(
        triples: Vec<AttributionTriple>,
        check_result: CheckResult,
        evidence_kind: EvidenceKind,
        observation_identity: ObservationIdentity,
    ) -> Self {
        let mut run = Self {
            triples,
            check_result,
            run_hash: String::new(),
            observation_key: Some(compute_observation_key(&observation_identity)),
            evidence_kind,
        };
        run.run_hash =
            compute_run_hash_with_observation(&run, evidence_kind, &observation_identity);
        run
    }

    /// Return the stable key used by storage-level replay protection.
    pub fn idempotency_key(&self) -> &str {
        self.observation_key.as_deref().unwrap_or(&self.run_hash)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AttributionConfig {
    /// The baseline same-file distance cut-off.
    pub max_line_distance: u32,
    /// Radius multiplier applied before candidate selection.
    pub candidate_radius_multiplier: u32,
    /// Maximum causes emitted for any single effect.
    pub top_k_causes: usize,
    /// Softmax temperature used to normalize candidate scores.
    pub softmax_temperature: f64,
    /// Distance decay constant used by [`attribution_score_with_confidence`].
    pub distance_decay: f64,
    /// Confidence assigned when a line cannot be resolved exactly.
    pub missing_line_confidence: f64,
    /// Exponential decay factor applied to graph edge weights during ingest.
    pub decay_factor: f64,
}

impl Default for AttributionConfig {
    fn default() -> Self {
        Self {
            max_line_distance: 12,
            candidate_radius_multiplier: 2,
            top_k_causes: 5,
            softmax_temperature: 0.75,
            distance_decay: 8.0,
            missing_line_confidence: 0.35,
            decay_factor: 0.995,
        }
    }
}

impl AttributionConfig {
    pub fn validate(&self) -> Result<(), CeaCoreError> {
        if self.candidate_radius_multiplier == 0 {
            return Err(CeaCoreError::InvalidConfig(
                "candidate_radius_multiplier must be >= 1".to_string(),
            ));
        }
        if self.top_k_causes == 0 {
            return Err(CeaCoreError::InvalidConfig(
                "top_k_causes must be >= 1".to_string(),
            ));
        }
        if !self.softmax_temperature.is_finite() || self.softmax_temperature <= 0.0 {
            return Err(CeaCoreError::InvalidConfig(
                "softmax_temperature must be finite and > 0".to_string(),
            ));
        }
        if !self.distance_decay.is_finite() || self.distance_decay <= 0.0 {
            return Err(CeaCoreError::InvalidConfig(
                "distance_decay must be finite and > 0".to_string(),
            ));
        }
        if !self.missing_line_confidence.is_finite()
            || !(0.0..=1.0).contains(&self.missing_line_confidence)
        {
            return Err(CeaCoreError::InvalidConfig(
                "missing_line_confidence must be in [0, 1]".to_string(),
            ));
        }
        if !self.decay_factor.is_finite() || !(0.0..=1.0).contains(&self.decay_factor) {
            return Err(CeaCoreError::InvalidConfig(
                "decay_factor must be in [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct EffectOccurrence {
    signature: EffectSignature,
    file: Option<String>,
    line: Option<u32>,
}

#[derive(Debug, Clone)]
struct CauseCandidate {
    signature: EditOpSignature,
    file: String,
    line: Option<u32>,
    mapping_confidence: f64,
}

#[derive(Debug, Clone)]
struct WeightedCandidate {
    cause: EditOpSignature,
    distance: i32,
    score: f64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LineMapIndex {
    pub mappings: HashMap<String, HashMap<u32, u32>>,
    resolved_anchors: HashMap<String, HashMap<usize, u32>>,
}

impl From<&LineAttributionMap> for LineMapIndex {
    fn from(value: &LineAttributionMap) -> Self {
        Self {
            mappings: value
                .mappings
                .iter()
                .map(|(file, mappings)| {
                    let indexed = mappings.iter().copied().collect::<HashMap<_, _>>();
                    (file.clone(), indexed)
                })
                .collect(),
            resolved_anchors: value.resolved_anchors.iter().fold(
                HashMap::new(),
                |mut acc, ((file, op_index), line)| {
                    acc.entry(file.clone())
                        .or_insert_with(HashMap::new)
                        .insert(*op_index, *line);
                    acc
                },
            ),
        }
    }
}

impl LineMapIndex {
    fn lookup_mapped_line(&self, file_key: &str, original_line: u32) -> Option<u32> {
        self.mappings
            .get(file_key)
            .and_then(|mappings| mappings.get(&original_line).copied())
    }

    fn resolved_anchor(&self, file_key: &str, op_index: usize) -> Option<u32> {
        self.resolved_anchors
            .get(file_key)
            .and_then(|anchors| anchors.get(&op_index).copied())
    }
}

pub(crate) fn build_line_map_index(line_map: &LineAttributionMap) -> LineMapIndex {
    LineMapIndex::from(line_map)
}

pub fn build_edit_op_signature(
    op: &EditOp,
    op_index: usize,
    _total_ops: usize,
    file_index: usize,
    _total_files: usize,
    file_extension: &str,
) -> Result<EditOpSignature, CeaCoreError> {
    let context_lines = op.context_lines();
    let context = context_lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    let op_index = u32::try_from(op_index).map_err(|_| {
        CeaCoreError::InvalidInput(format!("op_index {op_index} does not fit into u32"))
    })?;
    let file_index = u32::try_from(file_index).map_err(|_| {
        CeaCoreError::InvalidInput(format!("file_index {file_index} does not fit into u32"))
    })?;

    let lines_added = u32::try_from(op.lines_added()).map_err(|_| {
        CeaCoreError::InvalidInput(format!(
            "lines_added {} does not fit into u32",
            op.lines_added()
        ))
    })?;
    let lines_removed = u32::try_from(op.lines_removed()).map_err(|_| {
        CeaCoreError::InvalidInput(format!(
            "lines_removed {} does not fit into u32",
            op.lines_removed()
        ))
    })?;

    Ok(EditOpSignature {
        op_kind: match op {
            EditOp::Insert { .. } => EditOpKind::Insert,
            EditOp::Delete { .. } => EditOpKind::Delete,
            EditOp::Replace { .. } => EditOpKind::Replace,
        },
        anchor_kind: match op {
            EditOp::Insert {
                anchor: Anchor::AfterLine { .. },
                ..
            } => AnchorKind::AfterLine,
            EditOp::Insert {
                anchor: Anchor::BeforeLine { .. },
                ..
            } => AnchorKind::BeforeLine,
            EditOp::Insert {
                anchor: Anchor::AfterMatch { .. },
                ..
            } => AnchorKind::AfterMatch,
            EditOp::Insert {
                anchor: Anchor::BeforeMatch { .. },
                ..
            } => AnchorKind::BeforeMatch,
            EditOp::Delete { .. } | EditOp::Replace { .. } => AnchorKind::Range,
        },
        lines_added,
        lines_removed,
        context_hash: blake3::hash(context.as_bytes()).to_hex().to_string(),
        file_extension: file_extension.to_ascii_lowercase(),
        scope_tag: infer_scope(&context_lines),
        op_index: OpIndex(op_index),
        file_index: FileIndex(file_index),
    })
}

pub fn edit_op_node_id(signature: &EditOpSignature) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CAUSE_HASH_VERSION);
    hasher.update(signature.op_kind.as_str().as_bytes());
    hasher.update(signature.anchor_kind.as_str().as_bytes());
    hasher.update(&signature.lines_added.to_le_bytes());
    hasher.update(&signature.lines_removed.to_le_bytes());
    hasher.update(signature.context_hash.as_bytes());
    hasher.update(signature.file_extension.as_bytes());
    hasher.update(signature.scope_tag.as_str().as_bytes());
    hasher.update(&signature.op_index.0.to_le_bytes());
    hasher.update(&signature.file_index.0.to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

pub fn effect_node_id(signature: &EffectSignature) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(EFFECT_HASH_VERSION);
    hasher.update(signature.check_kind.as_bytes());
    hasher.update(signature.outcome.as_bytes());
    hasher.update(signature.severity.as_bytes());
    hasher.update(signature.message_class.as_bytes());
    hasher.update(
        &signature
            .line_offset_from_edit
            .unwrap_or(i32::MIN)
            .to_le_bytes(),
    );
    hasher.finalize().to_hex().to_string()
}

pub fn global_pass_effect_signature(check_kind: &str) -> EffectSignature {
    EffectSignature {
        check_kind: check_kind.to_string(),
        outcome: "pass".to_string(),
        severity: "pass".to_string(),
        message_class: "global_pass".to_string(),
        line_offset_from_edit: None,
    }
}

pub fn attribute_effects(
    patch: &StructuredPatch,
    check_result: &CheckResult,
    line_map: &LineAttributionMap,
    max_line_distance: u32,
) -> Result<Vec<AttributionTriple>, CeaCoreError> {
    let config = AttributionConfig {
        max_line_distance,
        ..AttributionConfig::default()
    };
    attribute_effects_with_config(patch, check_result, line_map, &config)
}

pub fn attribute_effects_with_config(
    patch: &StructuredPatch,
    check_result: &CheckResult,
    line_map: &LineAttributionMap,
    config: &AttributionConfig,
) -> Result<Vec<AttributionTriple>, CeaCoreError> {
    config.validate()?;

    let line_map_index = build_line_map_index(line_map);
    let changed_files = patch
        .edits
        .iter()
        .map(|edit| edit.path.to_string_lossy().to_string())
        .collect::<HashSet<_>>();
    let causes = collect_cause_candidates(patch, &line_map_index, config)?;
    let mut triples = Vec::new();

    for effect in collect_effect_occurrences(check_result) {
        let weighted = select_weighted_candidates(&causes, &changed_files, &effect, config);
        triples.extend(weighted.into_iter().map(|candidate| AttributionTriple {
            cause: candidate.cause,
            effect: effect.signature.clone(),
            distance: candidate.distance,
            weight: candidate.score,
        }));
    }

    triples.sort_by(canonical_triple_cmp);
    Ok(triples)
}

pub fn compute_run_hash(result: &AttributedRunResult) -> String {
    compute_run_hash_inner(result, None)
}

fn compute_run_hash_with_observation(
    result: &AttributedRunResult,
    evidence_kind: EvidenceKind,
    identity: &ObservationIdentity,
) -> String {
    compute_run_hash_inner(result, Some((evidence_kind, identity)))
}

fn compute_observation_key(identity: &ObservationIdentity) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(OBSERVATION_KEY_VERSION);
    update_optional_hash_field(&mut hasher, Some(&identity.observation_id));
    update_optional_hash_field(&mut hasher, Some(&identity.run_id));
    update_optional_hash_field(&mut hasher, Some(&identity.trial_id));
    update_optional_hash_field(&mut hasher, identity.patch_digest.as_deref());
    update_optional_hash_field(&mut hasher, identity.base_digest.as_deref());
    update_optional_hash_field(&mut hasher, identity.config_digest.as_deref());
    hasher.finalize().to_hex().to_string()
}

fn compute_run_hash_inner(
    result: &AttributedRunResult,
    observation: Option<(EvidenceKind, &ObservationIdentity)>,
) -> String {
    let mut hasher = blake3::Hasher::new();
    let identified = observation.is_some();
    hasher.update(if identified {
        IDENTIFIED_RUN_HASH_VERSION
    } else {
        RUN_HASH_VERSION
    });

    if let Some((evidence_kind, identity)) = observation {
        hasher.update(evidence_kind_label(evidence_kind).as_bytes());
        update_optional_hash_field(&mut hasher, Some(&identity.observation_id));
        update_optional_hash_field(&mut hasher, Some(&identity.run_id));
        update_optional_hash_field(&mut hasher, Some(&identity.trial_id));
        update_optional_hash_field(&mut hasher, identity.patch_digest.as_deref());
        update_optional_hash_field(&mut hasher, identity.base_digest.as_deref());
        update_optional_hash_field(&mut hasher, identity.config_digest.as_deref());
    }

    for category in check_categories(&result.check_result) {
        hasher.update(category.check_kind.as_bytes());
        hasher.update(&[u8::from(category.passed)]);
        let effect_count = u64::try_from(category.output.effects.len()).unwrap_or(u64::MAX);
        hasher.update(&effect_count.to_le_bytes());
    }

    let mut effects = collect_effect_occurrences(&result.check_result);
    effects.sort_by(canonical_effect_cmp);
    for effect in effects {
        hasher.update(effect_node_id(&effect.signature).as_bytes());
        if let Some(file) = effect.file {
            hasher.update(file.as_bytes());
        }
        hasher.update(&effect.line.unwrap_or(u32::MAX).to_le_bytes());
    }

    let mut canonical = result.triples.clone();
    canonical.sort_by(canonical_triple_cmp);
    for triple in canonical {
        hasher.update(edit_op_node_id(&triple.cause).as_bytes());
        hasher.update(effect_node_id(&triple.effect).as_bytes());
        hasher.update(&triple.distance.to_le_bytes());
        hasher.update(&weight_bucket(triple.weight).to_le_bytes());
    }

    hasher.finalize().to_hex().to_string()
}

fn evidence_kind_label(kind: EvidenceKind) -> &'static str {
    match kind {
        EvidenceKind::Observational => "observational",
        EvidenceKind::PairedInterventional => "paired_interventional",
        EvidenceKind::Ablation => "ablation",
        EvidenceKind::Counterfactual => "counterfactual",
        EvidenceKind::SyntheticTelemetry => "synthetic_telemetry",
    }
}

fn update_optional_hash_field(hasher: &mut blake3::Hasher, value: Option<&str>) {
    match value {
        Some(value) => {
            hasher.update(&[1]);
            hasher.update(&(value.len() as u64).to_le_bytes());
            hasher.update(value.as_bytes());
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

pub fn attribution_score(distance: i32, severity: &str, decay_factor: f64) -> f64 {
    attribution_score_with_confidence(distance, severity, decay_factor, 1.0)
}

pub fn attribution_score_with_confidence(
    distance: i32,
    severity: &str,
    decay_factor: f64,
    mapping_confidence: f64,
) -> f64 {
    let base = match severity {
        "error" => 1.0,
        "warning" => 0.75,
        "test_fail" => 0.9,
        "pass" => 0.2,
        _ => 0.5,
    };

    let distance = distance.unsigned_abs() as f64;
    let decay = 1.0 / (1.0 + (distance / decay_factor.max(f64::EPSILON)));
    (base * decay * mapping_confidence.clamp(0.0, 1.0)).max(0.0)
}

fn collect_cause_candidates(
    patch: &StructuredPatch,
    line_map_index: &LineMapIndex,
    config: &AttributionConfig,
) -> Result<Vec<CauseCandidate>, CeaCoreError> {
    let mut causes = Vec::new();

    for (file_index, edit) in patch.edits.iter().enumerate() {
        let extension = edit
            .path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let file_key = edit.path.to_string_lossy().to_string();

        for (op_index, op) in edit.ops.iter().enumerate() {
            let signature = build_edit_op_signature(
                op,
                op_index,
                edit.ops.len(),
                file_index,
                patch.edits.len(),
                extension,
            )?;
            let (line, mapping_confidence) =
                op_line_position(op, &file_key, op_index, line_map_index, config);

            causes.push(CauseCandidate {
                signature,
                file: file_key.clone(),
                line,
                mapping_confidence,
            });
        }
    }

    Ok(causes)
}

fn op_line_position(
    op: &EditOp,
    file_key: &str,
    op_index: usize,
    line_map_index: &LineMapIndex,
    config: &AttributionConfig,
) -> (Option<u32>, f64) {
    match op {
        EditOp::Insert { anchor, .. } => match anchor {
            Anchor::AfterLine { line, .. } => {
                let mapped = line_map_index.lookup_mapped_line(file_key, *line);
                (
                    Some(mapped.unwrap_or(*line).saturating_add(1)),
                    if mapped.is_some() {
                        1.0
                    } else {
                        config.missing_line_confidence
                    },
                )
            }
            Anchor::BeforeLine { line, .. } => {
                let mapped = line_map_index.lookup_mapped_line(file_key, *line);
                (
                    Some(mapped.unwrap_or(*line)),
                    if mapped.is_some() {
                        1.0
                    } else {
                        config.missing_line_confidence
                    },
                )
            }
            Anchor::AfterMatch { .. } | Anchor::BeforeMatch { .. } => {
                let resolved = line_map_index.resolved_anchor(file_key, op_index);
                (
                    resolved,
                    if resolved.is_some() {
                        1.0
                    } else {
                        config.missing_line_confidence
                    },
                )
            }
        },
        EditOp::Delete { range } | EditOp::Replace { range, .. } => {
            let mapped = line_map_index.lookup_mapped_line(file_key, range.start);
            (
                Some(mapped.unwrap_or(range.start)),
                if mapped.is_some() {
                    1.0
                } else {
                    config.missing_line_confidence
                },
            )
        }
    }
}

fn select_weighted_candidates(
    causes: &[CauseCandidate],
    changed_files: &HashSet<String>,
    effect: &EffectOccurrence,
    config: &AttributionConfig,
) -> Vec<WeightedCandidate> {
    let candidate_radius = config
        .max_line_distance
        .saturating_mul(config.candidate_radius_multiplier);
    let mut candidates = causes
        .iter()
        .filter_map(|cause| {
            if let Some(effect_file) = effect.file.as_deref() {
                if !changed_files.contains(effect_file) || cause.file != effect_file {
                    return None;
                }
            }

            let distance = candidate_distance(effect, cause, config.max_line_distance);
            if distance > candidate_radius as i32 {
                return None;
            }

            let score = attribution_score_with_confidence(
                distance,
                &effect.signature.severity,
                config.distance_decay,
                cause.mapping_confidence,
            );

            Some(WeightedCandidate {
                cause: cause.signature.clone(),
                distance,
                score,
            })
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| edit_op_node_id(&left.cause).cmp(&edit_op_node_id(&right.cause)))
            .then_with(|| left.distance.cmp(&right.distance))
    });
    candidates.truncate(config.top_k_causes);

    normalize_candidates(&mut candidates, config.softmax_temperature);
    candidates
}

fn normalize_candidates(candidates: &mut [WeightedCandidate], temperature: f64) {
    if candidates.is_empty() {
        return;
    }

    let logits = candidates
        .iter()
        .map(|candidate| candidate.score / temperature)
        .collect::<Vec<_>>();
    let max_logit = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut partition = 0.0;
    let mut weights = Vec::with_capacity(logits.len());
    for logit in logits {
        let value = if logit.is_finite() {
            (logit - max_logit).exp()
        } else {
            0.0
        };
        partition += value;
        weights.push(value);
    }

    if partition <= f64::EPSILON {
        let uniform = 1.0 / candidates.len() as f64;
        for candidate in candidates {
            candidate.score = uniform;
        }
        return;
    }

    for (candidate, weight) in candidates.iter_mut().zip(weights) {
        candidate.score = weight / partition;
    }
}

fn candidate_distance(
    effect: &EffectOccurrence,
    cause: &CauseCandidate,
    max_line_distance: u32,
) -> i32 {
    match (effect.line, cause.line) {
        (Some(effect_line), Some(cause_line)) => (effect_line as i32 - cause_line as i32).abs(),
        (Some(_), None) | (None, Some(_)) => max_line_distance as i32,
        (None, None) => 0,
    }
}

fn collect_effect_occurrences(check_result: &CheckResult) -> Vec<EffectOccurrence> {
    let mut effects = check_categories(check_result)
        .into_iter()
        .flat_map(|category| {
            category
                .output
                .effects
                .iter()
                .map(|effect| EffectOccurrence {
                    signature: effect.sig.clone(),
                    file: effect
                        .file
                        .as_ref()
                        .map(|path| path.to_string_lossy().to_string()),
                    line: effect.line,
                })
        })
        .collect::<Vec<_>>();

    for category in check_categories(check_result) {
        if category.passed && category.output.effects.is_empty() {
            effects.push(EffectOccurrence {
                signature: global_pass_effect_signature(category.check_kind),
                file: None,
                line: None,
            });
        }
    }

    effects
}

struct CheckCategory<'a> {
    check_kind: &'static str,
    passed: bool,
    output: &'a ParsedCheckOutput,
}

fn check_categories(check_result: &CheckResult) -> [CheckCategory<'_>; 3] {
    [
        CheckCategory {
            check_kind: "fmt",
            passed: check_result.fmt_pass,
            output: &check_result.fmt_output,
        },
        CheckCategory {
            check_kind: "clippy",
            passed: check_result.clippy_pass,
            output: &check_result.clippy_output,
        },
        CheckCategory {
            check_kind: "test",
            passed: check_result.test_pass,
            output: &check_result.test_output,
        },
    ]
}

fn canonical_effect_cmp(left: &EffectOccurrence, right: &EffectOccurrence) -> Ordering {
    effect_node_id(&left.signature)
        .cmp(&effect_node_id(&right.signature))
        .then_with(|| left.file.cmp(&right.file))
        .then_with(|| left.line.cmp(&right.line))
}

fn canonical_triple_cmp(left: &AttributionTriple, right: &AttributionTriple) -> Ordering {
    edit_op_node_id(&left.cause)
        .cmp(&edit_op_node_id(&right.cause))
        .then_with(|| effect_node_id(&left.effect).cmp(&effect_node_id(&right.effect)))
        .then_with(|| left.distance.cmp(&right.distance))
        .then_with(|| weight_bucket(left.weight).cmp(&weight_bucket(right.weight)))
}

fn weight_bucket(weight: f64) -> u64 {
    (weight.clamp(0.0, 1.0) * 1_000_000_000.0).round() as u64
}
