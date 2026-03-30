use std::fs;
use std::path::PathBuf;

fn production_source(path: &PathBuf) -> String {
    let text = fs::read_to_string(path).expect("source file should be readable");
    if path.ends_with("src/lib.rs") {
        let marker = "\n#[cfg(test)]";
        if let Some((production, _)) = text.split_once(marker) {
            return production.to_string();
        }
    }
    text
}

#[test]
fn production_source_has_no_panic_shortcuts() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let targets = [
        manifest_dir.join("src/lib.rs"),
        manifest_dir.join("src/main.rs"),
    ];
    let patterns = [
        (".unwrap(", ".unwrap("),
        (".expect(", ".expect("),
        ("panic!(", "panic!("),
        ("todo!(", "todo!("),
        ("unimplemented!(", "unimplemented!("),
    ];

    let mut violations = Vec::new();
    for path in targets {
        for (line_number, line) in production_source(&path).lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for (label, needle) in patterns {
                if code.contains(needle) {
                    violations.push(format!("{}:{}:{}", path.display(), line_number + 1, label));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "production panic shortcuts found:\n{}",
        violations.join("\n")
    );
}
