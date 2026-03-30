use std::path::Path;

pub(crate) fn classify_language(relative_path: &str) -> Option<&'static str> {
    let ext = Path::new(relative_path)
        .extension()
        .and_then(|ext| ext.to_str())?;
    Some(match ext {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "md" => "markdown",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "c" | "cc" | "cfg" | "cpp" | "css" | "go" | "graphql" | "h" | "hpp" | "html" | "java"
        | "kt" | "kts" | "proto" | "rb" | "ron" | "scala" | "sh" | "sql" | "swift" | "txt" => {
            "generic"
        }
        _ => return None,
    })
}

pub(crate) fn is_supported_text_file(relative_path: &str) -> bool {
    classify_language(relative_path).is_some()
}

pub(crate) fn is_supported_source_file_path(path: &Path) -> bool {
    path.to_str().is_some_and(is_supported_text_file)
}
