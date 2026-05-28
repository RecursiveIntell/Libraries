use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if dir.file_name().and_then(|s| s.to_str()) == Some("target") {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_runtime_placeholder_completion_strings_remain() {
    let root = workspace_root();
    let mut files = Vec::new();
    collect_rs_files(&root.join("crates"), &mut files);
    let forbidden = vec![
        format!("AiDENs {} response", "placeholder"),
        format!("{} runner output", "placeholder"),
        format!("wire provider {} next", "implementation"),
        format!("fake {}", "success"),
        format!("TODO {}", "runtime"),
    ];
    let mut hits = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).unwrap_or_default();
        for needle in &forbidden {
            if text.contains(needle) {
                hits.push(format!("{} contains {needle}", file.display()));
            }
        }
    }
    assert!(hits.is_empty(), "{}", hits.join("\n"));
}
