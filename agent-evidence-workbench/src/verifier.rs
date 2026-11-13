use crate::model::{AgentClaim, CheckResult, ClaimStatus};

/// Parse the recorded argv spelling, not arbitrary words anywhere in it.
/// `run_command` executes directly (without a shell) and records a quoted,
/// round-trippable argv. Legacy unquoted shell compositions stay unsupported.
fn program_and_args(command: &str) -> Option<(String, Vec<String>)> {
    let tokens = shlex::split(command)?;
    let (program, args) = tokens.split_first()?;
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "&&" | "||" | ";" | "|" | "&"))
    {
        return None;
    }
    let program = program.rsplit(['/', '\\']).next()?.to_string();
    if program.is_empty() {
        return None;
    }
    Some((program, args.to_vec()))
}

/// Successful help, version, collection and dry-run invocations prove no check.
fn executes_check(args: &[String]) -> bool {
    !args.iter().any(|a| {
        matches!(
            a.as_str(),
            "--help"
                | "-h"
                | "--version"
                | "-V"
                | "--dry-run"
                | "--collect-only"
                | "--co"
                | "--no-run"
                | "--list"
                | "--fixtures"
                | "--fixtures-per-test"
                | "--markers"
        )
    })
}

fn verb(args: &[String]) -> Option<&str> {
    // Cargo's optional toolchain selector precedes its subcommand.
    let args = if args.first().is_some_and(|a| a.starts_with('+')) {
        &args[1..]
    } else {
        args
    };
    args.first().map(String::as_str)
}

fn package_script(args: &[String], expected: &str) -> bool {
    args.first().is_some_and(|a| a == expected)
        || (args.first().is_some_and(|a| a == "run") && args.get(1).is_some_and(|a| a == expected))
}

fn has_input(args: &[String]) -> bool {
    args.iter().any(|a| !a.is_empty() && !a.starts_with('-'))
}

/// Recognize an actual test operation, never a directory or trailing argument.
pub fn is_test_command(command: &str) -> bool {
    let Some((program, args)) = program_and_args(command) else {
        return false;
    };
    if !executes_check(&args) {
        return false;
    }
    match program.as_str() {
        "pytest" => true,
        "cargo" | "go" | "mvn" | "gradle" | "dotnet" | "swift" | "make" => {
            matches!(verb(&args), Some("test" | "tests"))
        }
        "python" | "python3" => {
            args.first().is_some_and(|a| a == "-m") && args.get(1).is_some_and(|a| a == "pytest")
        }
        "npm" | "pnpm" | "yarn" | "bun" => package_script(&args, "test"),
        "run_tests.sh" => true,
        _ => false,
    }
}

/// A compile-only check or arbitrary default Make target is not a build.
pub fn is_build_command(command: &str) -> bool {
    let Some((program, args)) = program_and_args(command) else {
        return false;
    };
    if !executes_check(&args) {
        return false;
    }
    match program.as_str() {
        "cargo" | "go" | "gradle" | "dotnet" | "swift" | "make" => verb(&args) == Some("build"),
        "mvn" => matches!(verb(&args), Some("package" | "install")),
        "npm" | "pnpm" | "yarn" | "bun" => package_script(&args, "build"),
        _ => false,
    }
}

/// Lint/format evidence must run a checking operation, not a formatter write,
/// version probe, language server, or script argument named `lint`.
pub fn is_lint_command(command: &str) -> bool {
    let Some((program, args)) = program_and_args(command) else {
        return false;
    };
    if !executes_check(&args) {
        return false;
    }
    let has_check = args.iter().any(|a| a == "--check");
    if args.iter().any(|a| {
        matches!(
            a.as_str(),
            "--print-config"
                | "--inspect-config"
                | "--env-info"
                | "--init"
                | "--show-settings"
                | "--show-files"
                | "--find-config-path"
                | "--bug-report"
                | "--list-optional"
        )
    }) {
        return false;
    }
    match program.as_str() {
        "cargo" => verb(&args) == Some("clippy") || (verb(&args) == Some("fmt") && has_check),
        "ruff" => verb(&args) == Some("check") || (verb(&args) == Some("format") && has_check),
        "biome" => matches!(verb(&args), Some("check" | "lint")),
        "ty" => verb(&args) == Some("check"),
        "golangci-lint" => verb(&args) == Some("run"),
        "prettier" | "black" => has_check && has_input(&args),
        "eslint" | "mypy" | "flake8" | "shellcheck" | "hadolint" => has_input(&args),
        "npm" | "pnpm" | "yarn" | "bun" => package_script(&args, "lint"),
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
            ClaimStatus::Unsupported
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
