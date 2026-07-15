use crate::bootstrap::types::{SourceFileRecord, SymbolRecord};

pub(crate) fn extract(file: &SourceFileRecord) -> Vec<SymbolRecord> {
    let mut symbols = Vec::new();
    for (index, line) in file.content.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix('#') {
            let title = name.trim_start_matches('#').trim();
            if !title.is_empty() {
                symbols.push(SymbolRecord {
                    symbol_id: format!(
                        "workspace-source-symbol-{}",
                        crate::bootstrap::manifest::digest_text(&format!(
                            "{}:{}:section:{}:{}",
                            file.relative_path,
                            file.content_digest.hex(),
                            index + 1,
                            title
                        ))
                    ),
                    name: title.to_string(),
                    kind: "section".into(),
                    language: file.language.clone(),
                    line_start: index + 1,
                    line_end: index + 1,
                    signature: None,
                    parent_chunk_id: None,
                });
            }
        }
    }
    symbols
}
