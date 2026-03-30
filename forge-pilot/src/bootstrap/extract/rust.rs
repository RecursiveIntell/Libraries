use crate::bootstrap::types::{SourceFileRecord, SymbolRecord};

use super::SymbolExtractionResult;

pub(crate) fn extract(file: &SourceFileRecord) -> SymbolExtractionResult {
    let mut symbols = Vec::new();
    for (index, line) in file.content.lines().enumerate() {
        let trimmed = strip_comment(line).trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(name) = capture_name(
            trimmed,
            &["pub async fn ", "async fn ", "pub fn ", "fn "],
            '(',
        ) {
            symbols.push(symbol(file, index + 1, name, "function", Some(trimmed)));
            continue;
        }
        if let Some(name) = capture_name(trimmed, &["pub struct ", "struct "], '{') {
            symbols.push(symbol(file, index + 1, name, "struct", Some(trimmed)));
            continue;
        }
        if let Some(name) = capture_name(trimmed, &["pub enum ", "enum "], '{') {
            symbols.push(symbol(file, index + 1, name, "enum", Some(trimmed)));
            continue;
        }
        if let Some(name) = capture_name(trimmed, &["pub trait ", "trait "], '{') {
            symbols.push(symbol(file, index + 1, name, "trait", Some(trimmed)));
            continue;
        }
        if let Some(name) = capture_name(trimmed, &["pub mod ", "mod "], ';') {
            symbols.push(symbol(file, index + 1, name, "module", Some(trimmed)));
            continue;
        }
        if let Some(name) = capture_name(trimmed, &["pub type ", "type "], '=') {
            symbols.push(symbol(file, index + 1, name, "type_alias", Some(trimmed)));
            continue;
        }
        if let Some(name) = capture_name(trimmed, &["pub const ", "const "], ':') {
            symbols.push(symbol(file, index + 1, name, "const", Some(trimmed)));
            continue;
        }
        if let Some(name) = capture_name(trimmed, &["pub static ", "static "], ':') {
            symbols.push(symbol(file, index + 1, name, "static", Some(trimmed)));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("impl ") {
            let target = rest
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .trim_matches('{');
            if !target.is_empty() {
                symbols.push(symbol(file, index + 1, target, "impl", Some(trimmed)));
            }
        }
    }
    SymbolExtractionResult {
        symbols,
        degraded_reason: unsupported_surface_reason(&file.content),
    }
}

fn strip_comment(line: &str) -> &str {
    line.split("//").next().unwrap_or(line)
}

fn unsupported_surface_reason(content: &str) -> Option<String> {
    let mut markers = Vec::new();
    let mut previous_was_attribute = false;

    for line in content.lines() {
        let trimmed = strip_comment(line).trim();
        if trimmed.is_empty() {
            previous_was_attribute = false;
            continue;
        }

        let is_attribute = trimmed.starts_with("#[");
        if is_attribute {
            if trimmed.starts_with("#[cfg") || trimmed.starts_with("#[cfg_attr") {
                push_marker(&mut markers, "cfg_gated_item");
            } else {
                push_marker(&mut markers, "attribute_annotated_item");
            }
            previous_was_attribute = true;
            continue;
        }

        if previous_was_attribute && starts_rust_item(trimmed) {
            push_marker(&mut markers, "attribute_annotated_item");
        }
        previous_was_attribute = false;

        if starts_rust_item(trimmed) && trimmed.contains('<') {
            push_marker(&mut markers, "generic_header");
        }
        if starts_multiline_header(trimmed) {
            push_marker(&mut markers, "multiline_signature");
        }
    }

    if markers.is_empty() {
        None
    } else {
        Some(format!(
            "rust_line_scanner_v1 degraded on unsupported surface: {}",
            markers.join(", ")
        ))
    }
}

fn starts_rust_item(line: &str) -> bool {
    [
        "pub async fn ",
        "async fn ",
        "pub fn ",
        "fn ",
        "pub struct ",
        "struct ",
        "pub enum ",
        "enum ",
        "pub trait ",
        "trait ",
        "pub type ",
        "type ",
        "pub const ",
        "const ",
        "pub static ",
        "static ",
        "impl ",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
}

fn starts_multiline_header(line: &str) -> bool {
    if line.starts_with("pub async fn ")
        || line.starts_with("async fn ")
        || line.starts_with("pub fn ")
        || line.starts_with("fn ")
    {
        return !line.contains('(');
    }
    if line.starts_with("pub struct ")
        || line.starts_with("struct ")
        || line.starts_with("pub enum ")
        || line.starts_with("enum ")
        || line.starts_with("pub trait ")
        || line.starts_with("trait ")
        || line.starts_with("impl ")
    {
        return !(line.contains('{') || line.ends_with(';'));
    }
    false
}

fn push_marker(markers: &mut Vec<&'static str>, marker: &'static str) {
    if !markers.contains(&marker) {
        markers.push(marker);
    }
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
            "workspace-source-symbol:{}",
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
