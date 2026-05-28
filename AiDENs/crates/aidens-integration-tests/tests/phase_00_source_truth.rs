use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn aidens_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("aidens workspace root")
        .to_path_buf()
}

fn run_script(root: &Path, script_name: &str) {
    let script = root.join("scripts").join(script_name);
    let output = Command::new("bash")
        .arg(&script)
        .arg(root)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", script.display()));

    assert!(
        output.status.success(),
        "{} failed\nstdout:\n{}\nstderr:\n{}",
        script.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn source_paths_verified() {
    let root = aidens_root();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("AiDENs Cargo.toml");

    assert!(
        manifest.contains(r#"stack-ids = { version = "0.1.0", path = "../stack-ids" }"#),
        "AiDENs must depend on the canonical sibling stack-ids crate"
    );
    assert!(
        !manifest.contains("Libraries 2/stack-ids")
            && !manifest.contains("Libraries2/stack-ids")
            && !manifest.contains("libraries2/stack-ids")
            && !manifest.contains("repo_overlay/stack-ids")
            && !manifest.contains("scaffolds/stack-ids"),
        "AiDENs must not depend on any Libraries2 stack-ids path"
    );

    for sibling in [
        "stack-ids",
        "semantic-memory-forge",
        "forge-memory-bridge",
        "semantic-memory",
        "knowledge-runtime",
        "llm-tool-runtime",
    ] {
        assert!(
            root.join("..").join(sibling).join("Cargo.toml").exists(),
            "expected canonical sibling crate ../{sibling}"
        );
    }

    run_script(&root, "assert_stack_paths.sh");
}

#[test]
fn docs_updated_for_current_dependencies() {
    let root = aidens_root();

    for doc in [
        "SOURCE_BASIS.md",
        "CANONICAL_OWNER_MAP.md",
        "SHADOW_SEMANTICS_AUDIT.md",
        "ACCEPTANCE_GATES.md",
        "TESTKIT_TARGETS.md",
    ] {
        assert!(root.join(doc).exists(), "missing phase source doc {doc}");
    }

    run_script(&root, "assert_docs_match_cargo.sh");
}
