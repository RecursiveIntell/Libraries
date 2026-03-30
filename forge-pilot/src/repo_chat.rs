#[path = "repo_chat/answer.rs"]
mod answer;
#[path = "repo_chat/provider.rs"]
mod provider;
#[path = "repo_chat/retrieval.rs"]
mod retrieval;
#[path = "repo_chat/routing.rs"]
mod routing;
#[path = "repo_chat/types.rs"]
mod types;

use crate::config::LoopConfig;
use crate::observe::{observe_scope, Observation};
use crate::PilotError;
use knowledge_runtime::KnowledgeRuntime;
use semantic_memory::{MemoryStore, ProjectionQuery};

pub use provider::{
    provider_fallback_allowed, render_terminal_answer, repo_chat_provider_grounding_message,
};
pub use routing::route_question;
pub use types::{
    RepoChatAnswer, RepoChatCitation, RepoChatEvidence, RepoChatEvidenceType,
    RepoChatGroundingMode, RepoQuestionRoute,
};

pub async fn answer_repo_question(
    runtime: &KnowledgeRuntime,
    store: &MemoryStore,
    config: &LoopConfig,
    question: &str,
) -> Result<RepoChatAnswer, PilotError> {
    let observation = observe_scope(runtime, store, config).await?;
    answer_repo_question_from_observation(store, &observation, config, question).await
}

pub async fn answer_repo_question_from_observation(
    store: &MemoryStore,
    observation: &Observation,
    config: &LoopConfig,
    question: &str,
) -> Result<RepoChatAnswer, PilotError> {
    let route = routing::route_question(question);
    if route == RepoQuestionRoute::GeneralNonRepo {
        return Ok(answer::general_non_repo_answer(question));
    }

    if observation
        .source_inventory
        .imported_current_state
        .file_count
        == 0
    {
        return Ok(answer::missing_import_answer(route));
    }

    let Some(manifest) = retrieval::current_manifest_from_observation(observation) else {
        return Ok(answer::missing_manifest_answer(
            route,
            &observation.source_inventory.imported_current_state,
        ));
    };

    let mut query = ProjectionQuery::new(config.scope.key());
    query.limit = config.observation_projection_limit;
    let claim_versions = store.query_claim_versions(query).await?;
    let retrieval =
        retrieval::retrieve_repo_evidence(observation, &manifest, &claim_versions, question);
    Ok(answer::assemble_answer(
        observation,
        &manifest,
        question,
        retrieval,
    ))
}
