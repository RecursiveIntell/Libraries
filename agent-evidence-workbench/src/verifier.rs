use crate::model::{AgentClaim, CheckResult, ClaimStatus};
pub async fn verify_claims(
    claims: &[AgentClaim],
    checks: &[CheckResult],
    diff: &str,
) -> Vec<ClaimStatus> {
    claims
        .iter()
        .map(|c| match c.normalized_predicate.as_str() {
            "tests_pass" => {
                if checks
                    .iter()
                    .any(|x| x.passed && x.command.to_lowercase().contains("test"))
                {
                    ClaimStatus::Verified
                } else {
                    ClaimStatus::Unsupported
                }
            }
            "build_succeeds" => {
                if checks
                    .iter()
                    .any(|x| x.passed && x.command.to_lowercase().contains("build"))
                {
                    ClaimStatus::Verified
                } else {
                    ClaimStatus::Unsupported
                }
            }
            "lint_clean" => {
                if checks.iter().any(|x| {
                    x.passed
                        && ["lint", "clippy", "fmt", "format"]
                            .iter()
                            .any(|p| x.command.to_lowercase().contains(p))
                }) {
                    ClaimStatus::Verified
                } else {
                    ClaimStatus::Unsupported
                }
            }
            "fixed" | "added_test" => {
                if diff.trim().is_empty() {
                    ClaimStatus::Unsupported
                } else {
                    ClaimStatus::Partial
                }
            }
            _ => ClaimStatus::Unsupported,
        })
        .collect()
}
