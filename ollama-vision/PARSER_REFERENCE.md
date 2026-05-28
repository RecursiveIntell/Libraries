# PARSER_REFERENCE.md — Existing Parser Source (Annotated)

This is the complete source of `ollama-vision/src/parser.rs` — the code being extracted and generalized. Read this before writing any code. Annotations in `// >>>` comments mark what generalizes to which new module.

```rust
// >>> ALL OF THIS FILE moves to llm-pipeline/src/output_parser/
// >>> Strategies 1-5 become the shared extraction core (extract.rs)
// >>> Strategy 6-7 stay in list.rs as list-specific fallbacks
// >>> clean_tags stays in list.rs (parse_string_list only, not raw)
// >>> ParseError moves to error.rs and gets expanded
// >>> strip_think_tags moves to extract.rs

//! Robust LLM response parser with 7-strategy tag extraction.

/// Parse an LLM response into a list of tags using 7 strategies.
pub fn parse_tags(response: &str) -> Result<Vec<String>, ParseError> {
    let trimmed = response.trim();

    if trimmed.is_empty() {
        return Err(ParseError::EmptyResponse);
    }

    // >>> STRATEGY 1: Direct JSON parse — used by json.rs AND list.rs
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(trimmed) {
        return Ok(clean_tags(arr));
    }

    // >>> STRATEGY 2: Strip think tags — becomes extract::preprocess()
    // >>> Called once at the top of every parser, not per-strategy
    let cleaned = strip_think_tags(trimmed);
    let cleaned = cleaned.trim();

    if let Ok(arr) = serde_json::from_str::<Vec<String>>(cleaned) {
        return Ok(clean_tags(arr));
    }

    // >>> STRATEGY 3: JSON object with key — list.rs expands to check
    // >>> "tags", "items", "results", "list" keys
    if let Some(tags) = try_extract_tags_from_object(cleaned) {
        return Ok(clean_tags(tags));
    }

    // >>> STRATEGY 4: Code block — becomes extract::extract_code_block()
    // >>> Generalized: not just JSON arrays, any content from any fence type
    if let Some(tags) = extract_tags_from_code_block(cleaned) {
        return Ok(clean_tags(tags));
    }

    // >>> STRATEGY 5: Bracket match — becomes extract::find_bracketed()
    // >>> Parameterized: '['/']' for arrays, '{'/'}' for objects
    if let Some(tags) = find_json_array(cleaned) {
        return Ok(clean_tags(tags));
    }

    // >>> STRATEGY 6: List extraction — stays in list.rs only
    // >>> Not useful for json.rs, xml.rs, etc.
    if let Some(tags) = extract_from_list(cleaned) {
        return Ok(clean_tags(tags));
    }

    // >>> STRATEGY 7: Comma fallback — stays in list.rs only
    let tags: Vec<String> = cleaned
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim().to_lowercase())
        .filter(|s| !s.is_empty() && s.len() < 50)
        .collect();

    if tags.is_empty() {
        return Err(ParseError::Unparseable(cleaned.to_string()));
    }

    Ok(tags)
}

// >>> Moves to extract.rs as-is. Also add <thinking>...</thinking> variant.
pub fn strip_think_tags(text: &str) -> String {
    let mut result = text.to_string();
    while let Some(start) = result.find("<think>") {
        if let Some(end) = result[start..].find("</think>") {
            result = format!("{}{}", &result[..start], &result[start + end + 8..]);
        } else {
            result = result[..start].to_string();
            break;
        }
    }
    result
}

// >>> Moves to error.rs, expanded with more variants
#[derive(Debug)]
pub enum ParseError {
    EmptyResponse,
    Unparseable(String),
}

impl std::fmt::Display for ParseError { /* ... */ }
impl std::error::Error for ParseError {}

// >>> Moves to list.rs, expanded to check multiple key names
fn try_extract_tags_from_object(text: &str) -> Option<Vec<String>> {
    let val: serde_json::Value = serde_json::from_str(text).ok()?;
    let arr = val.get("tags").and_then(|v| v.as_array())?;
    let tags = arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    Some(tags)
}

// >>> Logic moves to extract::extract_code_block() (generalized)
// >>> Specific "then parse as Vec<String>" stays in list.rs
fn extract_tags_from_code_block(text: &str) -> Option<Vec<String>> {
    for marker in ["```json", "```"] {
        let mut search_from = 0;
        while let Some(start) = text[search_from..].find(marker) {
            let abs_start = search_from + start + marker.len();
            let content_start = text[abs_start..].find('\n').map(|p| abs_start + p + 1)?;
            if let Some(end) = text[content_start..].find("```") {
                let candidate = text[content_start..content_start + end].trim();
                if let Ok(arr) = serde_json::from_str::<Vec<String>>(candidate) {
                    return Some(arr);
                }
                if let Some(tags) = try_extract_tags_from_object(candidate) {
                    return Some(tags);
                }
            }
            search_from = abs_start;
        }
    }
    None
}

// >>> Logic moves to extract::find_bracketed('[', ']')
// >>> Parameterized to also support '{'/'}' for json.rs
fn find_json_array(text: &str) -> Option<Vec<String>> {
    let starts: Vec<usize> = text.match_indices('[').map(|(i, _)| i).collect();
    let ends: Vec<usize> = text.match_indices(']').map(|(i, _)| i).collect();

    for &start in starts.iter().rev() {
        for &end in ends.iter().rev() {
            if end <= start {
                continue;
            }
            let candidate = &text[start..=end];
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(candidate) {
                return Some(arr);
            }
        }
    }
    None
}

// >>> Stays in list.rs (list-specific fallback strategy)
fn extract_from_list(text: &str) -> Option<Vec<String>> {
    let lines: Vec<&str> = text.lines().collect();
    let list_items: Vec<String> = lines
        .iter()
        .filter_map(|line| {
            let trimmed = line.trim();
            if let Some(rest) = trimmed
                .strip_prefix(|c: char| c.is_ascii_digit())
                .and_then(|s| {
                    let s = s.trim_start_matches(|c: char| c.is_ascii_digit());
                    s.strip_prefix('.')
                        .or_else(|| s.strip_prefix(')'))
                })
            {
                let tag = rest.trim().trim_matches('"').trim();
                if !tag.is_empty() {
                    return Some(tag.to_string());
                }
            }
            for prefix in ["-", "*", "•"] {
                if let Some(rest) = trimmed.strip_prefix(prefix) {
                    let tag = rest.trim().trim_matches('"').trim();
                    if !tag.is_empty() {
                        return Some(tag.to_string());
                    }
                }
            }
            None
        })
        .collect();

    if list_items.len() >= 2 {
        Some(list_items)
    } else {
        None
    }
}

// >>> Stays in list.rs — only used by parse_string_list (not raw variant)
fn clean_tags(tags: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    tags.into_iter()
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty() && t.len() < 50 && seen.insert(t.clone()))
        .collect()
}

// >>> ALL 24 TESTS port to list.rs tests (they all test parse_string_list)
#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn parse_json_array() { /* ... */ }
    #[test] fn parse_with_think_blocks() { /* ... */ }
    #[test] fn parse_with_incomplete_think_block() { /* ... */ }
    #[test] fn strip_think_tags_complete() { /* ... */ }
    #[test] fn strip_think_tags_incomplete() { /* ... */ }
    #[test] fn strip_think_tags_multiple() { /* ... */ }
    #[test] fn parse_object_with_tags_key() { /* ... */ }
    #[test] fn parse_think_then_object() { /* ... */ }
    #[test] fn parse_markdown_code_block() { /* ... */ }
    #[test] fn parse_think_then_code_block() { /* ... */ }
    #[test] fn parse_code_block_with_object() { /* ... */ }
    #[test] fn parse_with_surrounding_text() { /* ... */ }
    #[test] fn parse_mixed_text_and_json() { /* ... */ }
    #[test] fn parse_numbered_list() { /* ... */ }
    #[test] fn parse_bulleted_list() { /* ... */ }
    #[test] fn parse_star_bulleted_list() { /* ... */ }
    #[test] fn parse_comma_separated() { /* ... */ }
    #[test] fn parse_empty_fails() { /* ... */ }
    #[test] fn parse_cleans_whitespace_and_case() { /* ... */ }
    #[test] fn parse_deduplicates() { /* ... */ }
    #[test] fn parse_filters_long_tags() { /* ... */ }
    #[test] fn clean_tags_filters_empty() { /* ... */ }
    // Note: strip_think_tags tests should move to extract.rs tests
}
```

## What Consumers Use

### tagger.rs
```rust
use crate::parser::{self, ParseError};
// ...
parser::parse_tags(content).map_err(TagError::Parse)
```
After migration: `parse_tags` is re-exported as an alias for `parse_string_list`.

### captioner.rs
```rust
use crate::parser;
// ...
let caption = parser::strip_think_tags(raw).trim().to_string();
```
After migration: `strip_think_tags` is re-exported from `llm_pipeline::output_parser`.

### lib.rs (public re-exports)
```rust
pub use parser::{parse_tags, strip_think_tags, ParseError};
```
After migration: same names, different source. No API break.
