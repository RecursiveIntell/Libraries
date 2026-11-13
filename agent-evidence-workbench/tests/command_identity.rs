use agent_evidence_workbench::verifier::{is_build_command, is_lint_command, is_test_command};

#[test]
fn script_directory_is_not_test_authority() {
    assert!(!is_test_command("./scripts/apply_overlay_into_repo.sh"));
    assert!(!is_test_command("./scripts/contest-helper"));
    assert!(!is_test_command("./scripts/not_run_tests.sh"));
    assert!(is_test_command(
        "./scripts/run_tests.sh tests/test_example.py"
    ));
}

#[test]
fn trailing_arguments_do_not_change_the_operation() {
    for command in [
        "cargo run -- test",
        "cargo metadata --format-version 1 test",
        "go run test",
        "node app.js test",
        "python3 -c 'print(1)' -m pytest",
        "npm exec test",
    ] {
        assert!(!is_test_command(command), "{command}");
    }
    assert!(!is_build_command("cargo run -- build"));
    assert!(!is_lint_command("npm exec lint"));
    assert!(is_test_command("cargo +stable test --lib"));
    assert!(is_test_command("npm run test -- --runInBand"));
    assert!(is_build_command("npm run build"));
}

#[test]
fn cargo_check_is_not_build_evidence() {
    assert!(!is_build_command("cargo check --all-targets"));
    assert!(is_build_command("cargo build --release"));
}

#[test]
fn lint_identity_requires_a_checking_operation() {
    for command in [
        "ruff version",
        "ruff server",
        "prettier --version",
        "eslint --version",
        "cargo fmt",
        "ruff format .",
        "black .",
        "golangci-lint version",
        "ty server",
        "biome version",
    ] {
        assert!(!is_lint_command(command), "{command}");
    }
    for command in [
        "ruff check .",
        "ruff format --check .",
        "cargo fmt --all -- --check",
        "prettier --check .",
        "black --check .",
        "golangci-lint run",
        "ty check .",
        "eslint .",
    ] {
        assert!(is_lint_command(command), "{command}");
    }
}

#[test]
fn literal_regex_and_quoted_arguments_survive() {
    assert!(is_test_command("go test -run 'TestFoo|TestBar' ./..."));
    assert!(is_test_command("go test -run TestFoo|TestBar ./..."));
    assert!(is_test_command("pytest -k 'foo & bar'"));
    assert!(!is_test_command("sh -c 'cargo test && true'"));
    assert!(!is_test_command("cargo test && echo done"));
    assert!(!is_test_command("pytest 'unterminated"));
}

#[test]
fn absolute_paths_preserve_tool_identity() {
    assert!(is_test_command("/usr/bin/cargo test"));
    assert!(is_test_command("/workspace/.venv/bin/pytest -q"));
    assert!(is_test_command(
        "'/workspace with spaces/.venv/bin/pytest' -q"
    ));
    assert!(is_build_command("/usr/bin/cargo build"));
    assert!(is_lint_command("/workspace/.venv/bin/ruff check ."));
}

#[test]
fn arbitrary_make_defaults_do_not_verify_a_build() {
    assert!(!is_build_command("make"));
    assert!(!is_build_command("make all"));
    assert!(!is_build_command("make gate"));
    assert!(is_build_command("make build"));
}

#[test]
fn successful_nonexecuting_invocations_do_not_promote() {
    for command in [
        "pytest --version",
        "pytest --collect-only",
        "cargo test --no-run",
        "cargo test --help",
        "npm test --dry-run",
    ] {
        assert!(!is_test_command(command), "{command}");
    }
    assert!(!is_build_command("cargo build --help"));
    assert!(!is_lint_command("ruff check --help"));
    assert!(!is_lint_command("ruff check -h"));
    assert!(!is_test_command("cargo test -- --list"));
    assert!(!is_test_command("pytest --fixtures"));
    for command in [
        "eslint --print-config file.ts",
        "ruff check --show-settings .",
        "ruff check --show-files .",
        "prettier --check --find-config-path file.ts",
        "flake8 --bug-report .",
    ] {
        assert!(!is_lint_command(command), "{command}");
    }
}

#[tokio::test]
async fn recorded_command_round_trips_actual_argv() {
    use agent_evidence_workbench::run::run_command;
    let cwd = tempfile::tempdir().unwrap();
    let arguments = vec![
        "TestFoo|TestBar".to_string(),
        "a & b; c".to_string(),
        "".to_string(),
        "'quoted'".to_string(),
    ];
    let result = run_command("echo", &arguments, cwd.path()).await.unwrap();
    assert!(result.passed);
    let parsed = shlex::split(&result.command).unwrap();
    assert_eq!(parsed[0], "echo");
    assert_eq!(&parsed[1..], &arguments);
}
