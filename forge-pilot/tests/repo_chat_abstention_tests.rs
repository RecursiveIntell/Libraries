mod common;

use common::{
    base_loop_config, open_forge_store, open_memory_store, point_config_at_dir, resources, tempdir,
    write_source_file,
};
use forge_pilot::{answer_repo_question, bootstrap_source_workspace, RepoQuestionRoute};
use knowledge_runtime::Scope;

#[tokio::test]
async fn repo_chat_ownership_answers_abstain_honestly() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn repo_chat_ready() -> bool { true }\n",
    );

    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("repo-chat-ownership");
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
        "Who owns repo_chat_ready in this repo?",
    )
    .await
    .unwrap();

    assert_eq!(answer.route, RepoQuestionRoute::Ownership);
    assert!(answer.abstained);
    assert!(!answer.grounded);
    assert!(answer
        .caveat
        .as_deref()
        .unwrap_or_default()
        .contains("ownership"));
}
