use aidens_cli::{doctor, plan_validate};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate has workspace parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn recall_extraction_reports_and_templates_exist() {
    let root = root();
    for relative in [
        "docs/06_RECALL_SOURCE_TOUCH_MAP.md",
        "docs/15_CURRENT_RECALL_AUDIT_SUMMARY.md",
        "examples/configs/coding-agent.toml",
        "examples/configs/daemon-safe.toml",
        "examples/coding-agent/README.md",
        "examples/templates/coding-agent-lane.template.md",
        "examples/templates/daemon-safe-operator.template.md",
    ] {
        assert!(root.join(relative).is_file(), "{relative} must exist");
    }
}

#[test]
fn recall_extraction_configs_are_operator_usable() {
    let root = root();
    for relative in [
        "examples/configs/coding-agent.toml",
        "examples/configs/daemon-safe.toml",
    ] {
        let config = root.join(relative);
        let config = config.display().to_string();
        let validation = plan_validate(&config).expect("example config validates");
        assert!(validation.contains("valid:"));
        let doctor = doctor(Some(config)).expect("doctor report renders");
        assert!(doctor.contains("provider"));
        assert!(doctor.contains("receipts"));
    }
}

#[test]
fn recall_extraction_does_not_add_recall_dependencies() {
    let root = root();
    let manifest_paths = std::fs::read_dir(root.join("crates"))
        .expect("crates dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("Cargo.toml"))
        .filter(|path| path.is_file())
        .chain(std::iter::once(root.join("Cargo.toml")))
        .collect::<Vec<_>>();

    for manifest in manifest_paths {
        let text = std::fs::read_to_string(&manifest).expect("manifest readable");
        assert_no_recall_dependency(&manifest, &text);
    }
}

#[test]
fn recall_templates_do_not_embed_app_specific_runtime_assumptions() {
    let root = root();
    for relative in [
        "examples/templates/coding-agent-lane.template.md",
        "examples/templates/daemon-safe-operator.template.md",
        "examples/configs/daemon-safe.toml",
    ] {
        let path = root.join(relative);
        let text = std::fs::read_to_string(&path).expect("template readable");
        let lower = text.to_ascii_lowercase();
        for forbidden in [
            "recallsession",
            "recall_workspace_",
            ".recall-coding",
            "scheduler.sqlite",
            "tauri",
        ] {
            assert!(
                !lower.contains(forbidden),
                "{} must not contain app-specific assumption {forbidden}",
                path.display()
            );
        }
    }
}

fn assert_no_recall_dependency(path: &Path, text: &str) {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();
        assert!(
            !lower.contains("/coding/recall")
                && !lower.contains("../recall")
                && !lower.starts_with("recall-")
                && !lower.contains(" recall-"),
            "{} must not depend on Recall/Recall-Coding: {trimmed}",
            path.display()
        );
    }
}
