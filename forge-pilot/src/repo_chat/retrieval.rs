use super::routing::route_question;
use super::types::{
    RepoChatCitation, RepoChatEvidence, RepoChatEvidenceType, RepoChatGroundingMode,
    RepoQuestionRoute,
};
use crate::bootstrap::types::BootstrapManifestFile;
use crate::bootstrap::{
    manifest_delta_from_current_state, manifest_from_current_state, BootstrapCurrentState,
    BootstrapManifestDelta, BootstrapManifestSnapshot, BootstrapRichness,
};
use crate::observe::Observation;
use semantic_memory::ProjectionClaimVersion;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub(crate) struct RetrievalHit {
    pub evidence: RepoChatEvidence,
    pub citation: RepoChatCitation,
    pub record_kind: String,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RetrievalOutcome {
    pub route: RepoQuestionRoute,
    pub grounding_mode: RepoChatGroundingMode,
    pub evidence: Vec<RetrievalHit>,
    pub delta: Option<BootstrapManifestDelta>,
    pub caveat: Option<String>,
}

pub(crate) fn current_manifest_from_observation(
    observation: &Observation,
) -> Option<BootstrapManifestSnapshot> {
    manifest_from_current_state(
        &observation.source_inventory.imported_current_state,
        &observation.recent_imports,
    )
}

pub(crate) fn retrieve_repo_evidence(
    observation: &Observation,
    manifest: &BootstrapManifestSnapshot,
    claim_versions: &[ProjectionClaimVersion],
    question: &str,
) -> RetrievalOutcome {
    let route = route_question(question);
    let question_lower = question.to_ascii_lowercase();
    let tokens = question_tokens(question);
    let current_state = &observation.source_inventory.imported_current_state;
    let grounding_mode = grounding_mode_for(route, current_state, observation);
    let delta = manifest_delta_from_current_state(current_state, &observation.recent_imports);
    let mut hits = match route {
        RepoQuestionRoute::Change => retrieve_change_hits(
            manifest,
            claim_versions,
            &question_lower,
            &tokens,
            delta.as_ref(),
        ),
        RepoQuestionRoute::Navigation => {
            retrieve_ranked_hits(manifest, claim_versions, &question_lower, &tokens, route)
        }
        RepoQuestionRoute::Structure => {
            retrieve_ranked_hits(manifest, claim_versions, &question_lower, &tokens, route)
        }
        RepoQuestionRoute::Ownership => {
            retrieve_ranked_hits(manifest, claim_versions, &question_lower, &tokens, route)
        }
        RepoQuestionRoute::DeepSemantic => {
            retrieve_ranked_hits(manifest, claim_versions, &question_lower, &tokens, route)
        }
        RepoQuestionRoute::GeneralNonRepo => Vec::new(),
    };
    if route == RepoQuestionRoute::Structure {
        hits = dedupe_by_path(hits, 4);
    } else {
        hits.truncate(4);
    }

    RetrievalOutcome {
        route,
        grounding_mode,
        evidence: hits,
        delta,
        caveat: substrate_caveat(route, current_state, observation),
    }
}

fn retrieve_change_hits(
    manifest: &BootstrapManifestSnapshot,
    claim_versions: &[ProjectionClaimVersion],
    question_lower: &str,
    tokens: &[String],
    delta: Option<&BootstrapManifestDelta>,
) -> Vec<RetrievalHit> {
    let Some(delta) = delta else {
        return Vec::new();
    };

    let mut hits = Vec::new();
    let mut push_delta_hits =
        |files: &[BootstrapManifestFile], label: &str, score_bias: f32, evidence_type| {
            for file in files {
                let score = path_overlap_score(&file.path, question_lower, tokens) + score_bias;
                if score > 0.0 || tokens.is_empty() {
                    hits.push(RetrievalHit {
                        evidence: RepoChatEvidence {
                            manifest_id: manifest.manifest_id.clone(),
                            path: file.path.clone(),
                            evidence_type,
                            symbol_name: None,
                            chunk_id: None,
                            line_start: None,
                            line_end: None,
                            score,
                            rationale: format!("Latest manifest delta marks this path as {label}."),
                        },
                        citation: RepoChatCitation {
                            path: file.path.clone(),
                            record_kind: label.to_string(),
                            chunk_id: None,
                            symbol_id: None,
                        },
                        record_kind: label.to_string(),
                        snippet: format!("{label} file {}", file.path),
                    });
                }
            }
        };

    push_delta_hits(
        &delta.changed_files,
        "changed",
        4.0,
        RepoChatEvidenceType::Delta,
    );
    push_delta_hits(&delta.new_files, "new", 3.5, RepoChatEvidenceType::Delta);
    push_delta_hits(
        &delta.deleted_files,
        "deleted",
        3.0,
        RepoChatEvidenceType::Deletion,
    );
    push_delta_hits(
        &delta.source_unchanged_derived_changed_files,
        "derived_changed",
        2.5,
        RepoChatEvidenceType::Delta,
    );

    if hits.is_empty() {
        hits.push(RetrievalHit {
            evidence: RepoChatEvidence {
                manifest_id: manifest.manifest_id.clone(),
                path: ".".into(),
                evidence_type: RepoChatEvidenceType::Manifest,
                symbol_name: None,
                chunk_id: None,
                line_start: None,
                line_end: None,
                score: 1.0,
                rationale: "Latest manifest delta is present but no path-specific change matched the question tokens.".into(),
            },
            citation: RepoChatCitation {
                path: ".".into(),
                record_kind: "manifest".into(),
                chunk_id: None,
                symbol_id: None,
            },
            record_kind: "manifest".into(),
            snippet: "latest manifest delta".into(),
        });
    }

    let current_hits = retrieve_ranked_hits(
        manifest,
        claim_versions,
        question_lower,
        tokens,
        RepoQuestionRoute::Navigation,
    );
    let changed_paths = hits
        .iter()
        .map(|hit| hit.evidence.path.clone())
        .collect::<BTreeSet<_>>();
    hits.extend(
        current_hits
            .into_iter()
            .filter(|hit| changed_paths.contains(&hit.evidence.path))
            .take(2),
    );
    hits.sort_by(|left, right| {
        right
            .evidence
            .score
            .partial_cmp(&left.evidence.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.evidence.path.cmp(&right.evidence.path))
    });
    hits
}

fn retrieve_ranked_hits(
    manifest: &BootstrapManifestSnapshot,
    claim_versions: &[ProjectionClaimVersion],
    question_lower: &str,
    tokens: &[String],
    route: RepoQuestionRoute,
) -> Vec<RetrievalHit> {
    let mut hits = claim_versions
        .iter()
        .filter_map(|claim| score_claim(manifest, claim, question_lower, tokens, route))
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| {
        right
            .evidence
            .score
            .partial_cmp(&left.evidence.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.evidence.path.cmp(&right.evidence.path))
    });
    hits
}

fn dedupe_by_path(hits: Vec<RetrievalHit>, limit: usize) -> Vec<RetrievalHit> {
    let mut paths = BTreeSet::new();
    let mut kept = Vec::new();
    for hit in hits {
        if paths.insert(hit.evidence.path.clone()) {
            kept.push(hit);
        }
        if kept.len() >= limit {
            break;
        }
    }
    kept
}

fn score_claim(
    manifest: &BootstrapManifestSnapshot,
    claim: &ProjectionClaimVersion,
    question_lower: &str,
    tokens: &[String],
    route: RepoQuestionRoute,
) -> Option<RetrievalHit> {
    let meta = claim
        .metadata
        .as_ref()?
        .get(crate::bootstrap::types::BOOTSTRAP_SOURCE_V2_METADATA_KEY)?;
    let record_kind = meta.get("record_kind")?.as_str()?.to_string();
    if record_kind == "manifest" || record_kind == "deletion" {
        return None;
    }
    let manifest_id = meta.get("manifest_id")?.as_str()?;
    if manifest_id != manifest.manifest_id {
        return None;
    }
    let path = meta.get("path")?.as_str()?.to_string();
    if !claim_matches_manifest(manifest, &record_kind, &path, meta) {
        return None;
    }

    let symbol_name = meta
        .get("symbol_name")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let chunk_id = meta
        .get("chunk_id")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let symbol_id = meta
        .get("symbol_id")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let line_start = meta
        .get("line_start")
        .and_then(Value::as_u64)
        .map(|v| v as usize)
        .or_else(|| {
            meta.get("start_line")
                .and_then(Value::as_u64)
                .map(|v| v as usize)
        });
    let line_end = meta
        .get("line_end")
        .and_then(Value::as_u64)
        .map(|v| v as usize)
        .or_else(|| {
            meta.get("end_line")
                .and_then(Value::as_u64)
                .map(|v| v as usize)
        });
    let path_score = path_overlap_score(&path, question_lower, tokens);
    let content_score = content_overlap_score(&claim.content, tokens);
    let symbol_score = symbol_name
        .as_deref()
        .map(|name| {
            content_overlap_score(name, tokens)
                + if question_lower.contains(name) {
                    4.0
                } else {
                    0.0
                }
        })
        .unwrap_or(0.0);
    let lexical_score = path_score + content_score + symbol_score;
    if lexical_score <= 0.0 {
        return None;
    }
    let route_bias = route_bias(route, &record_kind);
    let score = lexical_score + route_bias;

    Some(RetrievalHit {
        evidence: RepoChatEvidence {
            manifest_id: manifest.manifest_id.clone(),
            path: path.clone(),
            evidence_type: evidence_type_for(&record_kind),
            symbol_name: symbol_name.clone(),
            chunk_id: chunk_id.clone(),
            line_start,
            line_end,
            score,
            rationale: build_rationale(
                &record_kind,
                &path,
                symbol_name.as_deref(),
                line_start,
                line_end,
            ),
        },
        citation: RepoChatCitation {
            path: path.clone(),
            record_kind: record_kind.clone(),
            chunk_id: chunk_id.clone(),
            symbol_id: symbol_id.clone(),
        },
        record_kind,
        snippet: compact_snippet(&claim.content),
    })
}

fn claim_matches_manifest(
    manifest: &BootstrapManifestSnapshot,
    record_kind: &str,
    path: &str,
    meta: &Value,
) -> bool {
    let Some(file) = manifest.files.iter().find(|file| file.path == path) else {
        return false;
    };
    match record_kind {
        "file" => {
            file.content_digest
                == meta
                    .get("content_digest")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
        }
        "chunk" => file
            .chunk_ids
            .iter()
            .any(|id| Some(id.as_str()) == meta.get("chunk_id").and_then(Value::as_str)),
        "symbol" => file
            .symbol_ids
            .iter()
            .any(|id| Some(id.as_str()) == meta.get("symbol_id").and_then(Value::as_str)),
        _ => false,
    }
}

fn grounding_mode_for(
    route: RepoQuestionRoute,
    current_state: &BootstrapCurrentState,
    observation: &Observation,
) -> RepoChatGroundingMode {
    if observation.thin_export_active() || observation.missing_kernel_payload() {
        return RepoChatGroundingMode::CurrentStateDegraded;
    }
    match current_state.richness {
        BootstrapRichness::Thin => RepoChatGroundingMode::ThinImportedState,
        BootstrapRichness::Chunked if route == RepoQuestionRoute::DeepSemantic => {
            RepoChatGroundingMode::CurrentStateDegraded
        }
        _ => RepoChatGroundingMode::CurrentStateImported,
    }
}

fn substrate_caveat(
    route: RepoQuestionRoute,
    current_state: &BootstrapCurrentState,
    observation: &Observation,
) -> Option<String> {
    if observation.missing_kernel_payload() {
        return Some(
            "The latest import is missing kernel payload details, so the answer stays bounded to imported claim text and manifest truth.".into(),
        );
    }
    if observation.thin_export_active() {
        return Some(
            "The latest import is thin, so semantic coverage is reduced and repo-chat will abstain earlier.".into(),
        );
    }
    match (route, current_state.richness) {
        (RepoQuestionRoute::DeepSemantic, BootstrapRichness::Chunked) => Some(
            "Current imported state is chunked but not fully symbolized, so deep semantic answers remain heuristic.".into(),
        ),
        (_, BootstrapRichness::Thin) => Some(
            "Current imported state is thin, so only limited file-level grounding is available.".into(),
        ),
        _ => None,
    }
}

fn route_bias(route: RepoQuestionRoute, record_kind: &str) -> f32 {
    match (route, record_kind) {
        (RepoQuestionRoute::Navigation, "symbol") => 5.0,
        (RepoQuestionRoute::Navigation, "file") => 3.0,
        (RepoQuestionRoute::Navigation, "chunk") => 2.0,
        (RepoQuestionRoute::Structure, "file") => 4.0,
        (RepoQuestionRoute::Structure, "chunk") => 2.5,
        (RepoQuestionRoute::Ownership, "file") => 3.0,
        (RepoQuestionRoute::Ownership, "symbol") => 2.0,
        (RepoQuestionRoute::DeepSemantic, "symbol") => 4.0,
        (RepoQuestionRoute::DeepSemantic, "chunk") => 3.0,
        (RepoQuestionRoute::DeepSemantic, "file") => 1.5,
        _ => 0.0,
    }
}

fn evidence_type_for(record_kind: &str) -> RepoChatEvidenceType {
    match record_kind {
        "file" => RepoChatEvidenceType::FilePath,
        "symbol" => RepoChatEvidenceType::Symbol,
        "chunk" => RepoChatEvidenceType::Chunk,
        "deletion" => RepoChatEvidenceType::Deletion,
        _ => RepoChatEvidenceType::Manifest,
    }
}

fn build_rationale(
    record_kind: &str,
    path: &str,
    symbol_name: Option<&str>,
    line_start: Option<usize>,
    line_end: Option<usize>,
) -> String {
    match record_kind {
        "symbol" => format!(
            "Matched symbol {} in {}{}.",
            symbol_name.unwrap_or("unknown"),
            path,
            line_span_suffix(line_start, line_end)
        ),
        "chunk" => format!(
            "Matched chunk evidence in {}{}.",
            path,
            line_span_suffix(line_start, line_end)
        ),
        "file" => format!("Matched current file path {}.", path),
        other => format!("Matched current {other} evidence in {path}."),
    }
}

fn line_span_suffix(line_start: Option<usize>, line_end: Option<usize>) -> String {
    match (line_start, line_end) {
        (Some(start), Some(end)) if start == end => format!(" at line {start}"),
        (Some(start), Some(end)) => format!(" at lines {start}-{end}"),
        _ => String::new(),
    }
}

fn path_overlap_score(path: &str, question_lower: &str, tokens: &[String]) -> f32 {
    let mut score = 0.0;
    let path_lower = path.to_ascii_lowercase();
    if question_lower.contains(&path_lower) {
        score += 8.0;
    }
    if let Some(file_name) = path_lower.rsplit('/').next() {
        if question_lower.contains(file_name) {
            score += 4.0;
        }
    }
    score + content_overlap_score(&path_lower, tokens)
}

fn content_overlap_score(content: &str, tokens: &[String]) -> f32 {
    let haystack = content.to_ascii_lowercase();
    tokens
        .iter()
        .map(|token| if haystack.contains(token) { 1.0 } else { 0.0 })
        .sum()
}

fn compact_snippet(content: &str) -> String {
    let single = content.replace('\n', " ");
    single
        .chars()
        .take(160)
        .collect::<String>()
        .trim()
        .to_string()
}

fn question_tokens(question: &str) -> Vec<String> {
    question
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '.')
        .filter(|part| part.len() >= 2)
        .map(|part| part.to_ascii_lowercase())
        .collect()
}
