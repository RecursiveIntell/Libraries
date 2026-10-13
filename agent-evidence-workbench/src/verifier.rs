use crate::model::{AgentClaim, CheckResult, ClaimStatus};

/// Exact-token command identity for claim verification.
///
/// Substring matching is **not** admitted here: a passing command whose text
/// merely *contains* "test"/"build"/"lint" must never promote a claim to
/// `Verified` (a `contest-helper` script is not a test run, `prebuild-all` is
/// not a build, `lintel-check` is not a lint). The previous implementation used
/// `command.to_lowercase().contains(..)`, which allowed exactly that promotion
/// (R0 finding B-02).
///
/// Conservative bias: composite commands (shell separators) are rejected for
/// promotion purposes — claim evidence must be a single-purpose command whose
/// program and verb are unambiguous.
const SHELL_SEPARATORS: [char; 5] = [';', '|', '&', '\n', '\r'];

fn tokens(command: &str) -> Vec<String> {
    command
        .split_whitespace()
        .map(|t| {
            t.trim_matches(|c: char| c == '"' || c == '\'' || c == '`' || c == ',')
                .to_lowercase()
        })
        .filter(|t| !t.is_empty())
        .collect()
}

/// Split a command into its program and argument tokens; `None` when the
/// command is composite (contains shell separators).
fn program_and_args(command: &str) -> Option<(String, Vec<String>)> {
    if command.chars().any(|c| SHELL_SEPARATORS.contains(&c)) {
        return None;
    }
    let t = tokens(command);
    let (program, args) = t.split_first()?;
    Some((program.clone(), args.to_vec()))
}

/// True only for commands that are, by exact program/verb identity, a test run.
pub fn is_test_command(command: &str) -> bool {
    let Some((program, args)) = program_and_args(command) else {
        return false;
    };
    let has = |v: &str| args.iter().any(|a| a == v);
    match program.as_str() {
        "pytest" => true,
        "cargo" | "go" | "mvn" | "gradle" | "dotnet" | "swift" | "make" => {
            has("test") || has("tests")
        }
        "python" | "python3" => args.windows(2).any(|w| w[0] == "-m" && w[1] == "pytest"),
        "npm" | "pnpm" | "yarn" | "bun" | "npx" | "node" => has("test") || has("tests"),
        // Canonical repository runners (e.g. scripts/run_tests.sh) are test runs
        // by identity of the program path, never by substring of the whole line.
        _ if program.starts_with("./scripts/") || program.ends_with("run_tests.sh") => true,
        _ => false,
    }
}

/// True only for commands that are, by exact program/verb identity, a build.
pub fn is_build_command(command: &str) -> bool {
    let Some((program, args)) = program_and_args(command) else {
        return false;
    };
    let has = |v: &str| args.iter().any(|a| a == v);
    match program.as_str() {
        "cargo" => has("build") || has("check"),
        "go" | "mvn" | "gradle" | "dotnet" | "swift" | "npm" | "pnpm" | "yarn" | "bun" => {
            has("build")
        }
        "make" => args.is_empty() || has("build") || has("all"),
        _ => false,
    }
}

/// True only for commands that are, by exact program/verb identity, a lint/format check.
pub fn is_lint_command(command: &str) -> bool {
    let Some((program, args)) = program_and_args(command) else {
        return false;
    };
    let has = |v: &str| args.iter().any(|a| a == v);
    match program.as_str() {
        "cargo" => has("clippy") || has("fmt"),
        "ruff" | "eslint" | "biome" | "prettier" | "golangci-lint" | "shellcheck" | "hadolint"
        | "ty" | "mypy" | "flake8" | "black" => true,
        "npm" | "pnpm" | "yarn" | "bun" => has("lint") || has("format"),
        _ => false,
    }
}

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
                    .any(|x| x.passed && is_test_command(&x.command))
                {
                    ClaimStatus::Verified
                } else {
                    ClaimStatus::Unsupported
                }
            }
            "build_succeeds" => {
                if checks
                    .iter()
                    .any(|x| x.passed && is_build_command(&x.command))
                {
                    ClaimStatus::Verified
                } else {
                    ClaimStatus::Unsupported
                }
            }
            "lint_clean" => {
                if checks
                    .iter()
                    .any(|x| x.passed && is_lint_command(&x.command))
                {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AgentClaim;

    fn claim(predicate: &str) -> AgentClaim {
        AgentClaim {
            id: "c1".to_string(),
            text: predicate.to_string(),
            normalized_predicate: predicate.to_string(),
            source_quote: String::new(),
            source_location: None,
            status: ClaimStatus::NotChecked,
        }
    }

    fn check(command: &str, passed: bool) -> CheckResult {
        CheckResult {
            command: command.to_string(),
            exit_code: Some(if passed { 0 } else { 1 }),
            stdout: String::new(),
            stderr: String::new(),
            stdout_digest: String::new(),
            stderr_digest: String::new(),
            duration_ms: 1,
            passed,
        }
    }

    fn verify(predicate: &str, commands: &[(&str, bool)]) -> ClaimStatus {
        let checks: Vec<CheckResult> = commands.iter().map(|(c, p)| check(c, *p)).collect();
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime");
        rt.block_on(verify_claims(&[claim(predicate)], &checks, ""))
            .remove(0)
    }

    // --- B-02 regression: substring traps must NOT promote ---
    #[test]
    fn substring_traps_do_not_promote() {
        assert_eq!(
            verify("tests_pass", &[("contest-helper --run", true)]),
            ClaimStatus::Unsupported
        );
        assert_eq!(
            verify("tests_pass", &[("git checkout -b latest-tests", true)]),
            ClaimStatus::Unsupported
        );
        assert_eq!(
            verify("build_succeeds", &[("prebuild-all", true)]),
            ClaimStatus::Unsupported
        );
        assert_eq!(
            verify("build_succeeds", &[("echo rebuilding", true)]),
            ClaimStatus::Unsupported
        );
        assert_eq!(
            verify("lint_clean", &[("lintel-check", true)]),
            ClaimStatus::Unsupported
        );
        assert_eq!(
            verify("lint_clean", &[("echo formatted!", true)]),
            ClaimStatus::Unsupported
        );
    }

    // --- genuine commands still promote ---
    #[test]
    fn genuine_commands_promote() {
        assert_eq!(
            verify("tests_pass", &[("cargo test", true)]),
            ClaimStatus::Verified
        );
        assert_eq!(
            verify("tests_pass", &[("pytest -q", true)]),
            ClaimStatus::Verified
        );
        assert_eq!(
            verify("tests_pass", &[("python3 -m pytest tests/", true)]),
            ClaimStatus::Verified
        );
        assert_eq!(
            verify("tests_pass", &[("./scripts/run_tests.sh tests/x.py", true)]),
            ClaimStatus::Verified
        );
        assert_eq!(
            verify("build_succeeds", &[("cargo build --release", true)]),
            ClaimStatus::Verified
        );
        assert_eq!(
            verify("build_succeeds", &[("cargo check", true)]),
            ClaimStatus::Verified
        );
        assert_eq!(
            verify("lint_clean", &[("cargo clippy -- -D warnings", true)]),
            ClaimStatus::Verified
        );
        assert_eq!(
            verify("lint_clean", &[("ruff check .", true)]),
            ClaimStatus::Verified
        );
    }

    // --- failed commands never promote; composite commands are rejected ---
    #[test]
    fn failed_and_composite_commands_do_not_promote() {
        assert_eq!(
            verify("tests_pass", &[("cargo test", false)]),
            ClaimStatus::Unsupported
        );
        assert_eq!(
            verify("tests_pass", &[("cargo test && echo done", true)]),
            ClaimStatus::Unsupported
        );
        assert_eq!(
            verify("build_succeeds", &[("cargo build; true", true)]),
            ClaimStatus::Unsupported
        );
    }

    // --- diff-based predicates unchanged ---
    #[test]
    fn diff_predicates_unchanged() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let empty = rt.block_on(verify_claims(&[claim("fixed")], &[], ""));
        assert_eq!(empty, vec![ClaimStatus::Unsupported]);
        let some = rt.block_on(verify_claims(&[claim("added_test")], &[], "diff --git a b"));
        assert_eq!(some, vec![ClaimStatus::Partial]);
    }
}
