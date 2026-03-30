use super::types::{RepoChatAnswer, RepoQuestionRoute};

/// Returns true when provider fallback is still allowed for the answer.
pub fn provider_fallback_allowed(answer: &RepoChatAnswer) -> bool {
    answer.route == RepoQuestionRoute::GeneralNonRepo
}

/// Builds the provider-grounding message for a repo-chat answer.
pub fn repo_chat_provider_grounding_message(answer: &RepoChatAnswer) -> String {
    if answer.evidence.is_empty() {
        return format!(
            "Repo grounding unavailable. route={:?} grounding_mode={:?} caveat={}",
            answer.route,
            answer.grounding_mode,
            answer.caveat.clone().unwrap_or_else(|| "none".into())
        );
    }

    let evidence = answer
        .evidence
        .iter()
        .map(|item| {
            let mut line = format!(
                "{} [{} score={:.1}] {}",
                item.path,
                serde_json::to_string(&item.evidence_type).unwrap_or_else(|_| "\"unknown\"".into()),
                item.score,
                item.rationale
            );
            if let Some(name) = &item.symbol_name {
                line.push_str(&format!(" symbol={name}"));
            }
            if let Some(chunk_id) = &item.chunk_id {
                line.push_str(&format!(" chunk={chunk_id}"));
            }
            line
        })
        .collect::<Vec<_>>()
        .join("; ");

    format!(
        "Repo grounding available. manifest_id={} route={:?} grounding_mode={:?} answer={} evidence={}",
        answer.manifest_id.clone().unwrap_or_else(|| "none".into()),
        answer.route,
        answer.grounding_mode,
        answer.answer,
        evidence
    )
}

/// Renders a repo-chat answer for terminal display.
pub fn render_terminal_answer(answer: &RepoChatAnswer) -> String {
    let mut reply = answer.answer.clone();
    reply.push_str(&format!(
        "\n\nRoute: {:?}\nGrounding: {:?}",
        answer.route, answer.grounding_mode
    ));
    if let Some(manifest_id) = &answer.manifest_id {
        reply.push_str(&format!("\nManifest: {manifest_id}"));
    }
    if !answer.citations.is_empty() {
        reply.push_str("\nCitations: ");
        reply.push_str(
            &answer
                .citations
                .iter()
                .map(|citation| {
                    format!(
                        "{} [{}{}{}]",
                        citation.path,
                        citation.record_kind,
                        citation
                            .chunk_id
                            .as_ref()
                            .map(|id| format!(" chunk={id}"))
                            .unwrap_or_default(),
                        citation
                            .symbol_id
                            .as_ref()
                            .map(|id| format!(" symbol={id}"))
                            .unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join("; "),
        );
    }
    if let Some(caveat) = &answer.caveat {
        reply.push_str("\nCaveat: ");
        reply.push_str(caveat);
    }
    reply
}
