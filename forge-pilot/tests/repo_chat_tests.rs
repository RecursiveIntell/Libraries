mod common;

use common::{
    base_loop_config, open_forge_store, open_memory_store, point_config_at_dir, resources, tempdir,
    write_source_file,
};
use forge_pilot::{answer_repo_question, bootstrap_source_workspace, RepoQuestionRoute};
use knowledge_runtime::Scope;

#[tokio::test]
async fn repo_chat_answers_with_grounded_workspace_source_citations() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn repo_chat_ready() -> bool { true }\n",
    );

    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("repo-chat-grounded");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();

    let resources = resources(memory_store, forge_store, &config);
    let answer = answer_repo_question(
        &resources.runtime,
        &resources.memory_store,
        &config,
        "Where is repo_chat_ready defined?",
    )
    .await
    .unwrap();

    assert!(answer.grounded);
    assert!(!answer.abstained);
    assert_eq!(answer.route, RepoQuestionRoute::Navigation);
    assert!(answer.manifest_id.is_some());
    assert!(answer
        .citations
        .iter()
        .any(|citation| citation.path == "src/lib.rs"));
    assert!(answer
        .evidence
        .iter()
        .any(|evidence| evidence.path == "src/lib.rs"));
}

#[tokio::test]
async fn repo_chat_abstains_when_imported_source_memory_is_missing() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("repo-chat-empty");
    let mut config = base_loop_config(scope);
    point_config_at_dir(&mut config, dir.path());
    let resources = resources(memory_store, forge_store, &config);

    let answer = answer_repo_question(
        &resources.runtime,
        &resources.memory_store,
        &config,
        "What files define the API?",
    )
    .await
    .unwrap();

    assert!(answer.abstained);
    assert!(!answer.grounded);
}

#[tokio::test]
async fn repo_chat_filters_historical_imports_out_of_current_state_answers() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn old_name() -> bool { true }\n",
    );

    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("repo-chat-current-filter");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();

    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn new_name() -> bool { true }\n",
    );
    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();

    let resources = resources(memory_store, forge_store, &config);
    let old_answer = answer_repo_question(
        &resources.runtime,
        &resources.memory_store,
        &config,
        "Where is old_name defined?",
    )
    .await
    .unwrap();
    let new_answer = answer_repo_question(
        &resources.runtime,
        &resources.memory_store,
        &config,
        "Where is new_name defined?",
    )
    .await
    .unwrap();

    assert!(old_answer.abstained);
    assert!(new_answer.grounded);
    assert!(new_answer
        .citations
        .iter()
        .any(|citation| citation.path == "src/lib.rs"));
}
