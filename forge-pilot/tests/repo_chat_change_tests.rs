mod common;

use common::{
    base_loop_config, open_forge_store, open_memory_store, point_config_at_dir, resources, tempdir,
    write_source_file,
};
use forge_pilot::{answer_repo_question, bootstrap_source_workspace, RepoQuestionRoute};
use knowledge_runtime::Scope;
use std::fs;

#[tokio::test]
async fn repo_chat_change_questions_use_latest_manifest_delta() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn before_change() -> bool { true }\n",
    );

    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("repo-chat-change");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();

    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn after_change() -> bool { false }\n",
    );
    write_source_file(dir.path(), "src/new.rs", "pub fn new_file() {}\n");
    fs::remove_file(dir.path().join("src/new.rs")).unwrap();
    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();

    let resources = resources(memory_store, forge_store, &config);
    let answer = answer_repo_question(
        &resources.runtime,
        &resources.memory_store,
        &config,
        "What changed in src/lib.rs?",
    )
    .await
    .unwrap();

    assert_eq!(answer.route, RepoQuestionRoute::Change);
    assert!(answer.grounded);
    assert!(answer.answer.contains("Latest manifest delta"));
    assert!(answer.evidence.iter().any(|item| item.path == "src/lib.rs"));
    assert!(answer
        .evidence
        .iter()
        .any(|item| item.rationale.contains("Latest manifest delta")));
}
