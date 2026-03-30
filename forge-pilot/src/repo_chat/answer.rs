use super::retrieval::{RetrievalHit, RetrievalOutcome};
use super::types::{RepoChatAnswer, RepoChatGroundingMode, RepoQuestionRoute};
use crate::bootstrap::{BootstrapCurrentState, BootstrapManifestSnapshot, BootstrapRichness};
use crate::observe::Observation;
use std::collections::BTreeSet;

pub(crate) fn general_non_repo_answer(question: &str) -> RepoChatAnswer {
    RepoChatAnswer {
        answer: format!(
            "That question does not look repo-specific, so repo chat is not grounding it against imported workspace source: {question}"
        ),
        grounded: false,
        abstained: true,
        citations: Vec::new(),
        caveat: Some("Only clear non-repo questions are eligible for provider fallback.".into()),
        manifest_id: None,
        route: RepoQuestionRoute::GeneralNonRepo,
        grounding_mode: RepoChatGroundingMode::ProviderFallbackEligible,
        evidence: Vec::new(),
    }
}

pub(crate) fn missing_import_answer(route: RepoQuestionRoute) -> RepoChatAnswer {
    RepoChatAnswer {
        answer: "I do not have imported source memory for this namespace yet, so I cannot answer grounded repo questions.".into(),
        grounded: false,
        abstained: true,
        citations: Vec::new(),
        caveat: Some("Run bootstrap-source first, then ask again.".into()),
        manifest_id: None,
        route,
        grounding_mode: RepoChatGroundingMode::Ungrounded,
        evidence: Vec::new(),
    }
}

pub(crate) fn missing_manifest_answer(
    route: RepoQuestionRoute,
    current_state: &BootstrapCurrentState,
) -> RepoChatAnswer {
    RepoChatAnswer {
        answer: "The imported current-state object does not resolve to a current manifest snapshot, so I cannot answer grounded repo questions honestly.".into(),
        grounded: false,
        abstained: true,
        citations: Vec::new(),
        caveat: Some(format!(
            "Current-state manifest is unavailable or inconsistent. import_disposition={}",
            current_state.import_disposition
        )),
        manifest_id: current_state.manifest_id.clone(),
        route,
        grounding_mode: RepoChatGroundingMode::Ungrounded,
        evidence: Vec::new(),
    }
}

pub(crate) fn assemble_answer(
    observation: &Observation,
    manifest: &BootstrapManifestSnapshot,
    question: &str,
    retrieval: RetrievalOutcome,
) -> RepoChatAnswer {
    match retrieval.route {
        RepoQuestionRoute::Change => assemble_change_answer(manifest, retrieval),
        RepoQuestionRoute::Ownership => assemble_ownership_answer(manifest, retrieval),
        RepoQuestionRoute::Navigation => assemble_navigation_answer(manifest, retrieval),
        RepoQuestionRoute::Structure => assemble_structure_answer(manifest, retrieval),
        RepoQuestionRoute::DeepSemantic => {
            assemble_deep_semantic_answer(observation, manifest, question, retrieval)
        }
        RepoQuestionRoute::GeneralNonRepo => general_non_repo_answer(question),
    }
}

fn assemble_navigation_answer(
    manifest: &BootstrapManifestSnapshot,
    retrieval: RetrievalOutcome,
) -> RepoChatAnswer {
    if retrieval.evidence.is_empty() {
        return abstained_manifest_answer(
            manifest,
            retrieval.route,
            retrieval.grounding_mode,
            "I cannot ground that navigation question in the current imported source state.",
            retrieval.caveat.or(Some(
                "No current file, symbol, or chunk matched the question.".into(),
            )),
        );
    }

    let lines = retrieval
        .evidence
        .iter()
        .take(3)
        .map(describe_hit)
        .collect::<Vec<_>>()
        .join(" ");
    answer_from_hits(
        manifest,
        retrieval,
        lines,
        Some("Answer is grounded in the current imported workspace-source state.".into()),
    )
}

fn assemble_structure_answer(
    manifest: &BootstrapManifestSnapshot,
    retrieval: RetrievalOutcome,
) -> RepoChatAnswer {
    if retrieval.evidence.is_empty() {
        return abstained_manifest_answer(
            manifest,
            retrieval.route,
            retrieval.grounding_mode,
            "I cannot ground that structure question in the current imported source state.",
            retrieval.caveat.or(Some(
                "No current multi-file structure evidence matched the question.".into(),
            )),
        );
    }

    let paths = retrieval
        .evidence
        .iter()
        .map(|hit| hit.evidence.path.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut answer = format!(
        "In current manifest {}, the relevant structure is spread across {}.",
        manifest.manifest_id,
        paths.join(", ")
    );
    if let Some(first) = retrieval.evidence.first() {
        answer.push(' ');
        answer.push_str(&describe_hit(first));
    }
    answer_from_hits(manifest, retrieval, answer, None)
}

fn assemble_deep_semantic_answer(
    observation: &Observation,
    manifest: &BootstrapManifestSnapshot,
    _question: &str,
    retrieval: RetrievalOutcome,
) -> RepoChatAnswer {
    if retrieval.evidence.is_empty() {
        return abstained_manifest_answer(
            manifest,
            retrieval.route,
            retrieval.grounding_mode,
            "I cannot ground that deeper semantic question in the current imported source state.",
            retrieval.caveat.or(Some(
                "No current symbol, chunk, or file evidence matched the question.".into(),
            )),
        );
    }
    if observation.source_inventory.imported_current_state.richness != BootstrapRichness::Symbolized
        && retrieval.evidence.len() < 2
    {
        return abstained_manifest_answer(
            manifest,
            retrieval.route,
            retrieval.grounding_mode,
            "I do not have enough semantic coverage in the current imported state to answer that deeply and honestly.",
            retrieval.caveat.or(Some("Current state is not fully symbolized for a deeper semantic answer.".into())),
        );
    }

    let answer = retrieval
        .evidence
        .iter()
        .take(3)
        .map(describe_hit)
        .collect::<Vec<_>>()
        .join(" ");
    answer_from_hits(manifest, retrieval, answer, None)
}

fn assemble_ownership_answer(
    manifest: &BootstrapManifestSnapshot,
    retrieval: RetrievalOutcome,
) -> RepoChatAnswer {
    if retrieval.evidence.is_empty() {
        return abstained_manifest_answer(
            manifest,
            retrieval.route,
            retrieval.grounding_mode,
            "I cannot identify ownership from the current imported source state.",
            Some("Imported repo state carries paths and symbols, not human or team ownership metadata.".into()),
        );
    }

    abstained_manifest_answer(
        manifest,
        retrieval.route,
        retrieval.grounding_mode,
        "I can point to the relevant files, but I cannot honestly claim a human or team owner from imported source memory alone.",
        Some("Imported repo state exposes structural location, not explicit ownership truth.".into()),
    )
}

fn assemble_change_answer(
    manifest: &BootstrapManifestSnapshot,
    retrieval: RetrievalOutcome,
) -> RepoChatAnswer {
    let Some(delta) = retrieval.delta.as_ref() else {
        return abstained_manifest_answer(
            manifest,
            retrieval.route,
            retrieval.grounding_mode,
            "I cannot answer that change question honestly because the latest manifest delta is unavailable.",
            Some("Change questions require the latest manifest delta from imported current-state.".into()),
        );
    };

    if delta.new_files.is_empty()
        && delta.changed_files.is_empty()
        && delta.deleted_files.is_empty()
        && delta.source_unchanged_derived_changed_files.is_empty()
    {
        return RepoChatAnswer {
            answer: format!(
                "Latest manifest delta for {} reports no source or derived changes.",
                manifest.manifest_id
            ),
            grounded: true,
            abstained: false,
            citations: vec![super::types::RepoChatCitation {
                path: ".".into(),
                record_kind: "manifest".into(),
                chunk_id: None,
                symbol_id: None,
            }],
            caveat: retrieval.caveat,
            manifest_id: Some(manifest.manifest_id.clone()),
            route: retrieval.route,
            grounding_mode: retrieval.grounding_mode,
            evidence: vec![super::types::RepoChatEvidence {
                manifest_id: manifest.manifest_id.clone(),
                path: ".".into(),
                evidence_type: super::types::RepoChatEvidenceType::Manifest,
                symbol_name: None,
                chunk_id: None,
                line_start: None,
                line_end: None,
                score: 1.0,
                rationale: "Latest manifest delta reports no changes.".into(),
            }],
        };
    }

    let mut parts = vec![format!(
        "Latest manifest delta for {} shows {} new, {} changed, {} deleted, and {} derived-only file changes.",
        manifest.manifest_id,
        delta.new_files.len(),
        delta.changed_files.len(),
        delta.deleted_files.len(),
        delta.source_unchanged_derived_changed_files.len()
    )];
    let selected = retrieval
        .evidence
        .iter()
        .take(3)
        .map(describe_hit)
        .collect::<Vec<_>>();
    if !selected.is_empty() {
        parts.push(selected.join(" "));
    }
    answer_from_hits(manifest, retrieval, parts.join(" "), None)
}

fn answer_from_hits(
    manifest: &BootstrapManifestSnapshot,
    retrieval: RetrievalOutcome,
    answer: String,
    fallback_caveat: Option<String>,
) -> RepoChatAnswer {
    RepoChatAnswer {
        answer,
        grounded: true,
        abstained: false,
        citations: retrieval
            .evidence
            .iter()
            .map(|hit| hit.citation.clone())
            .collect(),
        caveat: retrieval.caveat.or(fallback_caveat),
        manifest_id: Some(manifest.manifest_id.clone()),
        route: retrieval.route,
        grounding_mode: retrieval.grounding_mode,
        evidence: retrieval
            .evidence
            .iter()
            .map(|hit| hit.evidence.clone())
            .collect(),
    }
}

fn abstained_manifest_answer(
    manifest: &BootstrapManifestSnapshot,
    route: RepoQuestionRoute,
    grounding_mode: RepoChatGroundingMode,
    answer: &str,
    caveat: Option<String>,
) -> RepoChatAnswer {
    RepoChatAnswer {
        answer: answer.into(),
        grounded: false,
        abstained: true,
        citations: Vec::new(),
        caveat,
        manifest_id: Some(manifest.manifest_id.clone()),
        route,
        grounding_mode,
        evidence: Vec::new(),
    }
}

fn describe_hit(hit: &RetrievalHit) -> String {
    match hit.record_kind.as_str() {
        "symbol" => {
            let name = hit.evidence.symbol_name.as_deref().unwrap_or("unknown");
            match (hit.evidence.line_start, hit.evidence.line_end) {
                (Some(start), Some(end)) if start == end => {
                    format!(
                        "`{name}` is defined in {} at line {}.",
                        hit.evidence.path, start
                    )
                }
                (Some(start), Some(end)) => {
                    format!(
                        "`{name}` is defined in {} at lines {}-{}.",
                        hit.evidence.path, start, end
                    )
                }
                _ => format!("`{name}` is defined in {}.", hit.evidence.path),
            }
        }
        "chunk" => format!(
            "{} contains relevant chunk evidence: {}.",
            hit.evidence.path, hit.snippet
        ),
        "file" => format!(
            "{} contains relevant file-level evidence: {}.",
            hit.evidence.path, hit.snippet
        ),
        "changed" => format!(
            "{} is marked changed in the latest manifest delta.",
            hit.evidence.path
        ),
        "new" => format!(
            "{} is marked new in the latest manifest delta.",
            hit.evidence.path
        ),
        "deleted" => format!(
            "{} is marked deleted in the latest manifest delta.",
            hit.evidence.path
        ),
        "derived_changed" => {
            format!(
                "{} has derived-only changes in the latest manifest delta.",
                hit.evidence.path
            )
        }
        _ => format!("{} contributes grounded evidence.", hit.evidence.path),
    }
}
