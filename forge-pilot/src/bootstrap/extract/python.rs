use crate::bootstrap::types::{SourceFileRecord, SymbolRecord};

pub(crate) fn extract(file: &SourceFileRecord) -> Vec<SymbolRecord> {
    let mut symbols = Vec::new();
    for (index, line) in file.content.lines().enumerate() {
        let trimmed = strip_comment(line).trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(name) = capture_name(trimmed, &["async def ", "def "], '(') {
            symbols.push(symbol(file, index + 1, name, "function", Some(trimmed)));
            continue;
        }
        if let Some(name) = capture_name(trimmed, &["class "], '(')
            .or_else(|| capture_name(trimmed, &["class "], ':'))
        {
            symbols.push(symbol(file, index + 1, name, "class", Some(trimmed)));
        }
    }
    symbols
}

fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or(line)
}

fn capture_name<'a>(line: &'a str, prefixes: &[&str], terminator: char) -> Option<&'a str> {
    let rest = prefixes
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix))?
        .trim_start();
    let name = rest
        .split([terminator, '<', ' ', ':'])
        .next()
        .unwrap_or_default()
        .trim();
    (!name.is_empty()).then_some(name)
}

fn symbol(
    file: &SourceFileRecord,
    line: usize,
    name: &str,
    kind: &str,
    signature: Option<&str>,
) -> SymbolRecord {
    SymbolRecord {
        symbol_id: format!(
            "workspace-source-symbol-{}",
            crate::bootstrap::manifest::digest_text(&format!(
                "{}:{}:{}:{}:{}",
                file.relative_path,
                file.content_digest.hex(),
                line,
                kind,
                name
            ))
        ),
        name: name.to_string(),
        kind: kind.to_string(),
        language: file.language.clone(),
        line_start: line,
        line_end: line,
        signature: signature.map(ToString::to_string),
        parent_chunk_id: None,
    }
}
