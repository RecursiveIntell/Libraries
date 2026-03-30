mod common;

use forge_pilot::{route_question, RepoQuestionRoute};

#[test]
fn repo_chat_routes_questions_explicitly() {
    assert_eq!(
        route_question("Where is repo_chat_ready defined?"),
        RepoQuestionRoute::Navigation
    );
    assert_eq!(
        route_question("What changed in src/lib.rs?"),
        RepoQuestionRoute::Change
    );
    assert_eq!(
        route_question("How is the bootstrap pipeline structured across files?"),
        RepoQuestionRoute::Structure
    );
    assert_eq!(
        route_question("Who owns the bootstrap import path?"),
        RepoQuestionRoute::Ownership
    );
    assert_eq!(
        route_question("How does manifest filtering work in the repo chat code?"),
        RepoQuestionRoute::DeepSemantic
    );
    assert_eq!(
        route_question("What is the capital of France?"),
        RepoQuestionRoute::GeneralNonRepo
    );
}
