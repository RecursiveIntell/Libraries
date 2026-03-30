use super::types::RepoQuestionRoute;

/// Classifies a question into the repo-chat routing surface.
pub fn route_question(question: &str) -> RepoQuestionRoute {
    let lower = question.to_ascii_lowercase();
    let repo_signal = has_repo_signal(&lower);

    if has_any(
        &lower,
        &[
            "change",
            "changed",
            "delta",
            "diff",
            "modified",
            "updated",
            "removed",
            "deleted",
            "new file",
            "what changed",
        ],
    ) && repo_signal
    {
        return RepoQuestionRoute::Change;
    }

    if has_any(
        &lower,
        &[
            "owner",
            "owns",
            "ownership",
            "responsible",
            "maintains",
            "maintainer",
        ],
    ) && repo_signal
    {
        return RepoQuestionRoute::Ownership;
    }

    if has_any(
        &lower,
        &[
            "structure",
            "structured",
            "architecture",
            "organized",
            "organization",
            "layout",
            "fit together",
            "module layout",
        ],
    ) && repo_signal
    {
        return RepoQuestionRoute::Structure;
    }

    if has_any(
        &lower,
        &[
            "where",
            "which file",
            "which path",
            "defined",
            "definition",
            "located",
            "contains",
            "find",
            "path",
        ],
    ) && repo_signal
    {
        return RepoQuestionRoute::Navigation;
    }

    if repo_signal {
        return RepoQuestionRoute::DeepSemantic;
    }

    RepoQuestionRoute::GeneralNonRepo
}

fn has_repo_signal(question: &str) -> bool {
    question.contains('/')
        || question.contains("::")
        || question.contains('_')
        || question.contains(".rs")
        || question.contains(".ts")
        || question.contains(".tsx")
        || question.contains(".py")
        || has_any(
            question,
            &[
                "repo",
                "repository",
                "code",
                "file",
                "path",
                "symbol",
                "function",
                "struct",
                "enum",
                "module",
                "manifest",
                "workspace",
                "crate",
                "impl",
                "source",
            ],
        )
}

fn has_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
