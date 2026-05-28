# ARCHITECTURE.md — output_parser Module Specification

## Overview

The `output_parser` module is a production-grade LLM response parser for Rust. It extracts structured data from raw LLM text output using a multi-strategy approach that handles think blocks, markdown fences, malformed JSON, and other real-world model output quirks — all without requiring an additional LLM call.

This module lives inside `llm-pipeline` as `pub mod output_parser`.

## Module Structure

```
llm-pipeline/src/output_parser/
├── mod.rs          — Public API, re-exports, top-level parse dispatch
├── error.rs        — ParseError enum
├── extract.rs      — Shared extraction strategies (the reusable core)
├── repair.rs       — Deterministic JSON repair
├── json.rs         — parse_json<T>, parse_json_value
├── list.rs         — parse_string_list (generalized parse_tags)
├── xml.rs          — parse_xml_tag, parse_xml_tags
├── yaml.rs         — parse_yaml<T> (feature-gated behind "yaml")
├── choice.rs       — parse_choice
├── number.rs       — parse_number<T>
└── text.rs         — parse_text (clean prose extraction)
```

Every module follows the same internal pattern:
1. Preprocess via `extract::preprocess()` (strip think tags, trim)
2. Try most-structured strategy first (direct parse)
3. Try extraction strategies (code blocks, bracket matching)
4. Try repair (JSON-consuming parsers only)
5. Try format-specific fallbacks (lists, comma-separated, etc.)
6. Return `ParseError` with the cleaned text for debugging

---

## Cargo.toml Changes (llm-pipeline)

Add to `[features]`:
```toml
[features]
default = []
yaml = ["dep:serde_yaml"]
```

Add to `[dependencies]`:
```toml
serde_yaml = { version = "0.9", optional = true }
```

Verify that `serde`, `serde_json`, and `thiserror` are already present. Do NOT add any other dependencies.

Add to `lib.rs`:
```rust
pub mod output_parser;
```

---

## Module Specifications

### error.rs

```rust
use std::fmt;

/// Errors returned by output parsers.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// The LLM response was empty or whitespace-only.
    #[error("empty LLM response")]
    EmptyResponse,

    /// No parsing strategy could extract the expected format.
    #[error("could not parse {expected_format} from LLM response: {text}")]
    Unparseable {
        expected_format: &'static str,
        text: String,
    },

    /// JSON was extracted but failed to deserialize into the target type.
    #[error("JSON deserialization failed: {reason}")]
    DeserializationFailed {
        reason: String,
        raw_json: String,
    },

    /// No valid choice from the provided options was found.
    #[error("no valid choice found in response (valid: {valid:?})")]
    NoMatchingChoice {
        valid: Vec<String>,
    },

    /// No number found, or number was outside the expected range.
    #[error("no valid number found in response")]
    NoNumber,
}
```

The `ParseError` carries enough context for debugging (the cleaned text, the expected format, the raw JSON that failed deser) without exposing the full raw LLM response in error messages by default. The `text` field in `Unparseable` should be truncated to 200 chars max.

---

### extract.rs — The Reusable Core

This is the load-bearing module. Every parser calls into these shared functions.

```rust
/// Full preprocessing pipeline applied to every LLM response.
/// Strips think tags, trims whitespace.
pub fn preprocess(text: &str) -> String

/// Strip all <think>...</think> blocks from text.
/// Handles complete blocks, incomplete blocks (no closing tag),
/// and multiple sequential blocks.
pub fn strip_think_tags(text: &str) -> String

/// Extract content from the first matching markdown code block.
/// Searches for ```lang and bare ``` fences.
/// Returns (language_hint, content) where hint is None for bare fences.
pub fn extract_code_block(text: &str) -> Option<(Option<&str>, &str)>

/// Extract content from a code block matching a specific language.
/// e.g., extract_code_block_for(text, "json") looks for ```json blocks.
pub fn extract_code_block_for(text: &str, lang: &str) -> Option<&str>

/// Find a bracketed substring by matching open/close delimiters.
/// Handles nesting. Prefers later (more likely to be the actual output)
/// over earlier occurrences.
///
/// find_bracketed(text, '[', ']') — finds JSON arrays
/// find_bracketed(text, '{', '}') — finds JSON objects
pub fn find_bracketed(text: &str, open: char, close: char) -> Option<&str>
```

#### Implementation Notes

**`strip_think_tags`**: Port directly from `ollama-vision/src/parser.rs`. Exact same logic: loop finding `<think>`, if closing `</think>` found strip the block, if no closing tag strip from `<think>` to end. Also handle `<thinking>...</thinking>` (some models use the longer form).

**`extract_code_block`**: Generalized from the existing `extract_tags_from_code_block`. Instead of hardcoding `["```json", "```"]`, scan for the triple-backtick pattern, capture the optional language identifier on the same line, find the matching closing ```` ``` ````, return the inner content. The language hint lets callers filter: `extract_code_block_for(text, "yaml")` only matches ` ```yaml ` blocks.

**`find_bracketed`**: Generalized from the existing `find_json_array`. The current code hardcodes `[` and `]`. Parameterize to also support `{` and `}` for object extraction. Keep the same reverse-iteration strategy (prefer later matches). Add proper nesting depth tracking — the current implementation tries all start/end combinations which is O(n²) in bracket count but works because LLM output rarely has many brackets. Keep that approach but add a nesting-aware fast path: scan forward from each `open` tracking depth, stop at the matching `close` at depth 0.

**`preprocess`**: Calls `strip_think_tags`, then trims. Every parser module calls this as step 1.

---

### repair.rs — Deterministic JSON Fixer

```rust
/// Attempt to repair common LLM JSON mistakes without calling the model again.
///
/// Returns the repaired string if any fixes were applied and the result
/// is valid JSON. Returns None if repair was not possible.
///
/// Repairs applied (in order):
/// 1. Strip inline comments (// and /* */)
/// 2. Replace Python booleans/None: True→true, False→false, None→null
/// 3. Remove trailing commas before } or ]
/// 4. Replace single-quoted strings with double-quoted
/// 5. Quote unquoted object keys
/// 6. Append missing closing brackets/braces
/// 7. Escape raw newlines inside string values
pub fn try_repair_json(broken: &str) -> Option<String>
```

#### Implementation Details

**Comment stripping**: Remove `// ...` to end of line and `/* ... */` blocks. Be careful not to strip inside string literals — track whether you're inside a quoted string.

**Python booleans**: Replace word-boundary `True` → `true`, `False` → `false`, `None` → `null`. Only replace when NOT inside a quoted string. Simple approach: walk character by character, track in-string state, replace whole words.

**Trailing commas**: Scan for `,` followed by optional whitespace then `}` or `]`. Remove the comma. This is the single most common LLM JSON error.

**Single quotes → double quotes**: This is the tricky one. `{'key': 'value'}` → `{"key": "value"}`. BUT you must not break strings containing apostrophes like `{"text": "don't"}`. Strategy: only replace single quotes that appear at string boundaries (preceded by `{`, `:`, `,`, `[` or whitespace, and followed by content then another single quote at a boundary position). If the heuristic isn't confident, skip this repair.

**Unquoted keys**: Detect `{ key:` pattern (word chars followed by colon, not inside quotes). Wrap the key in double quotes. Common with smaller models.

**Missing closing brackets**: Count unmatched `{`/`[` and append the corresponding `}`/`]`. Simple but handles truncated output.

**Raw newlines in strings**: If there's a literal newline between opening and closing quotes of a string value, replace with `\n`.

After ALL repairs are applied, validate with `serde_json::from_str::<serde_json::Value>()`. If valid, return `Some(repaired)`. If still invalid, return `None`.

#### Test Cases for repair.rs (minimum)
```
trailing_comma_object:     {"a": 1, "b": 2,}        → {"a": 1, "b": 2}
trailing_comma_array:      [1, 2, 3,]                → [1, 2, 3]
single_quotes:             {'key': 'value'}          → {"key": "value"}
python_booleans:           {"active": True}          → {"active": true}
python_none:               {"data": None}            → {"data": null}
unquoted_keys:             {name: "Josh", age: 30}   → {"name": "Josh", "age": 30}
inline_comment:            {"a": 1} // comment       → {"a": 1}
missing_close_brace:       {"a": 1                   → {"a": 1}
missing_close_bracket:     [1, 2, 3                  → [1, 2, 3]
mixed_errors:              {'a': True, 'b': None,}   → {"a": true, "b": null}
no_repair_needed:          {"a": 1}                  → None (already valid)
unrepairable:              not json at all            → None
apostrophe_safety:         {"text": "don't stop"}    → preserved correctly
nested_trailing:           {"a": [1, 2,], "b": 3,}  → {"a": [1, 2], "b": 3}
```

---

### json.rs

```rust
use serde::de::DeserializeOwned;

/// Parse an LLM response into a typed struct.
///
/// Strategies (in order):
/// 1. Direct deserialize on preprocessed text
/// 2. Extract from markdown code block (```json)
/// 3. Extract from any code block
/// 4. Bracket-match a JSON object ({...})
/// 5. Bracket-match a JSON array ([...])
/// 6. Repair malformed JSON then retry strategies 1-5
///
/// # Examples
/// ```
/// #[derive(Deserialize)]
/// struct Analysis {
///     sentiment: String,
///     confidence: f64,
/// }
///
/// let response = r#"<think>analyzing...</think>{"sentiment": "positive", "confidence": 0.92}"#;
/// let result: Analysis = parse_json(response).unwrap();
/// ```
pub fn parse_json<T: DeserializeOwned>(response: &str) -> Result<T, ParseError>

/// Parse into a serde_json::Value when you don't know the schema.
pub fn parse_json_value(response: &str) -> Result<serde_json::Value, ParseError>
```

#### Implementation Flow

```
preprocess(response)
  → try serde_json::from_str::<T>(cleaned)
  → try extract_code_block_for(cleaned, "json") → deserialize
  → try extract_code_block(cleaned) → if content looks like JSON → deserialize
  → try find_bracketed(cleaned, '{', '}') → deserialize
  → try find_bracketed(cleaned, '[', ']') → deserialize
  --- if all above failed, enter repair path ---
  → try_repair_json on the best candidate found above
  → try_repair_json on the full cleaned text
  → ParseError::Unparseable
```

The "best candidate" is whichever extracted string got closest to valid JSON (code block content, bracketed substring, etc.). Track it through the pipeline so repair gets the most promising input.

#### Test Cases (minimum)
```
direct_json_object:         {"key": "value"}
direct_json_array:          [1, 2, 3]
think_then_json:            <think>...</think>{"key": "value"}
code_block_json:            Here's the data:\n```json\n{"key": "value"}\n```
bare_code_block:            ```\n{"key": "value"}\n```
json_in_prose:              The analysis is {"sentiment": "positive"} as shown.
nested_json:                {"outer": {"inner": [1,2,3]}}
repaired_trailing_comma:    {"key": "value",}  → succeeds after repair
repaired_single_quotes:     {'key': 'value'}   → succeeds after repair
think_and_code_block:       <think>...</think>\n```json\n{"a":1}\n```
json_with_surrounding_text: Sure! Here's your result: {"a": 1}\nHope that helps!
```

---

### list.rs

```rust
/// Parse an LLM response into a cleaned list of strings.
///
/// Cleaning: lowercase, trim, deduplicate, filter empties, filter >50 chars.
/// This is the direct successor to ollama-vision's parse_tags.
///
/// Strategies (in order):
/// 1. Direct JSON array
/// 2. JSON object with common list keys ("tags", "items", "results", "list")
/// 3. Markdown code block → JSON array/object
/// 4. Bracket-matched JSON array
/// 5. Numbered/bulleted list extraction
/// 6. Comma-separated fallback
pub fn parse_string_list(response: &str) -> Result<Vec<String>, ParseError>

/// Parse into a list without tag-specific cleaning.
/// No forced lowercase, no length filter, no dedup.
/// For general-purpose list extraction from LLM responses.
pub fn parse_string_list_raw(response: &str) -> Result<Vec<String>, ParseError>
```

#### Changes from Original parse_tags

1. Strategy 3 now uses the generalized `extract_code_block` from extract.rs instead of inlined code block logic
2. Strategy 2 expanded: check not just `"tags"` but also `"items"`, `"results"`, `"list"` keys (common LLM patterns)
3. Strategy 5 uses `find_bracketed` from extract.rs instead of the inlined `find_json_array`
4. JSON strategies now go through `try_repair_json` as a fallback before falling to list/comma strategies
5. `parse_string_list_raw` variant for non-tag use cases

Port ALL 24 tests from the existing parser.rs, then add:
```
json_object_with_items_key:   {"items": ["a", "b"]}  → ["a", "b"]
json_object_with_results_key: {"results": ["a", "b"]} → ["a", "b"]
repaired_json_list:           ['tag1', 'tag2']        → ["tag1", "tag2"] (after repair)
raw_preserves_case:           parse_string_list_raw("A, B") → ["A", "B"]
raw_preserves_length:         long items not filtered in raw mode
```

---

### xml.rs

```rust
use std::collections::HashMap;

/// Extract content from a single XML-style tag in an LLM response.
///
/// Looks for <tag>content</tag> after preprocessing.
/// Handles missing close tags (returns content to end of string).
/// Does NOT use a full XML parser — these are structured delimiters.
///
/// # Examples
/// ```
/// let response = "<answer>The capital is Paris.</answer>";
/// let answer = parse_xml_tag(response, "answer").unwrap();
/// assert_eq!(answer, "The capital is Paris.");
/// ```
pub fn parse_xml_tag(response: &str, tag: &str) -> Result<String, ParseError>

/// Extract content from multiple XML-style tags into a map.
///
/// Returns a HashMap of tag_name → content for each tag found.
/// Missing tags are simply absent from the map (not an error).
/// At least one tag must be found or returns ParseError.
///
/// # Examples
/// ```
/// let response = "<analysis>Looks good</analysis><confidence>0.95</confidence>";
/// let result = parse_xml_tags(response, &["analysis", "confidence"]).unwrap();
/// assert_eq!(result["analysis"], "Looks good");
/// assert_eq!(result["confidence"], "0.95");
/// ```
pub fn parse_xml_tags(
    response: &str,
    tags: &[&str],
) -> Result<HashMap<String, String>, ParseError>
```

#### Implementation Notes

- Preprocessing strips `<think>` tags but NOT the target tags
- Find `<tag>` (case-sensitive), then find `</tag>`. Content is everything between.
- If no `</tag>` found, take content from `<tag>` to end of string (same pattern as `strip_think_tags` for incomplete blocks)
- Handle nested occurrences: only extract the first match for each tag name
- Trim whitespace from extracted content
- `parse_xml_tags` must find at least one of the requested tags or return `ParseError::Unparseable`
- Do NOT depend on an XML parsing crate

#### Test Cases
```
simple_tag:                  <answer>Paris</answer>
think_then_tag:              <think>...</think><answer>Paris</answer>
multiple_tags:               <a>one</a><b>two</b>
nested_content:              <answer>The answer is <b>bold</b></answer>  → "The answer is <b>bold</b>"
missing_close:               <answer>Paris  → "Paris"
multiline_content:           <code>\nfn main() {}\n</code>
whitespace_trimming:         <answer>  Paris  </answer> → "Paris"
tag_not_found:               <wrong>data</wrong> looking for "answer" → error
partial_tags_found:          3 requested, 2 found → ok with 2 in map
no_tags_found:               none of requested tags present → error
case_sensitive:              <Answer> doesn't match "answer"
```

---

### yaml.rs (feature-gated)

```rust
#[cfg(feature = "yaml")]
use serde::de::DeserializeOwned;

/// Parse an LLM response containing YAML into a typed struct.
///
/// Strategies:
/// 1. Direct YAML parse on preprocessed text
/// 2. Extract from ```yaml code block
/// 3. Extract from any code block → try as YAML
///
/// Requires the `yaml` feature flag.
#[cfg(feature = "yaml")]
pub fn parse_yaml<T: DeserializeOwned>(response: &str) -> Result<T, ParseError>
```

#### Implementation
- `preprocess` → try `serde_yaml::from_str::<T>`
- Extract ```` ```yaml ```` block → try deserialize
- Extract any code block → try deserialize
- No repair step for YAML (too ambiguous with whitespace-sensitive format)

#### Test Cases
```
direct_yaml:           "name: Josh\nage: 30"
yaml_code_block:       "```yaml\nname: Josh\n```"
think_then_yaml:       "<think>...</think>\nname: Josh\nage: 30"
```

---

### choice.rs

```rust
/// Extract a single choice from a set of valid options.
///
/// Handles common LLM response patterns:
/// - Direct match: "positive"
/// - Bold: "**positive**"
/// - Quoted: "'positive'" or "\"positive\""
/// - In prose: "I would classify this as positive because..."
/// - Parenthesized: "(positive)"
///
/// Matching is case-insensitive. If multiple valid choices appear,
/// returns the first one found in the text.
pub fn parse_choice<'a>(
    response: &str,
    valid_choices: &[&'a str],
) -> Result<&'a str, ParseError>
```

#### Implementation
1. Preprocess
2. Check if cleaned text (lowercased, trimmed of punctuation) exactly matches any choice
3. Check if cleaned text starts with a choice (e.g., "positive. Because...")
4. Search for each choice as a word boundary match in the text (avoid substring false positives: "positive" shouldn't match inside "unpositive")
5. If multiple choices found, return the one appearing first in the text
6. If none found, return `ParseError::NoMatchingChoice`

Word boundary detection: check that the character before and after the match is not alphanumeric. Don't use regex — manual char checks.

#### Test Cases
```
exact_match:             "positive"                            → "positive"
with_period:             "positive."                           → "positive"
bold:                    "**positive**"                        → "positive"
quoted:                  "\"positive\""                        → "positive"
in_prose:                "I'd classify this as positive"       → "positive"
case_insensitive:        "POSITIVE"                            → "positive"
first_wins:              "positive and negative aspects"       → "positive"
with_think:              "<think>hmm</think>negative"          → "negative"
no_match:                "maybe"                               → error
no_substring:            "unpositive" with valid=["positive"]  → error
```

---

### number.rs

```rust
use std::str::FromStr;

/// Extract a numeric value from an LLM response.
///
/// Handles common patterns:
/// - Direct number: "8.5"
/// - Score format: "8.5/10", "8/10"
/// - In prose: "I'd rate it 8.5 out of 10"
/// - Labeled: "Score: 8.5", "Rating: 8"
/// - With think block: "<think>considering...</think>8.5"
pub fn parse_number<T: FromStr>(response: &str) -> Result<T, ParseError>

/// Extract a number and verify it falls within [min, max] inclusive.
pub fn parse_number_in_range<T: FromStr + PartialOrd + std::fmt::Display>(
    response: &str,
    min: T,
    max: T,
) -> Result<T, ParseError>
```

#### Implementation
1. Preprocess
2. Try parsing the entire cleaned text as T directly
3. Look for common labeled patterns: `score:`, `rating:`, `result:` followed by a number
4. Look for fraction patterns: `N/M` — extract N
5. Find all number-like substrings (digits, optional decimal point, optional leading minus)
6. Return the first valid one that parses as T
7. `parse_number_in_range` calls `parse_number` then validates bounds

Number extraction: scan for sequences matching `-?\d+(\.\d+)?`. Collect all matches, try parsing each as T, return first success.

#### Test Cases
```
integer:          "8"                     → 8
float:            "8.5"                   → 8.5
fraction:         "8/10"                  → 8
in_prose:         "I'd give it a 7.5"     → 7.5
labeled:          "Score: 9"              → 9
with_think:       "<think>...</think>8.5" → 8.5
negative:         "-3"                    → -3
range_pass:       "8" in [1, 10]          → 8
range_fail:       "15" in [1, 10]         → error
no_number:        "great work"            → error
multiple_numbers: "Page 3 of 5, score 8"  → 8 (last number, or labeled)
```

For the "multiple numbers" case: if a labeled pattern is found (`Score: N`), prefer that. Otherwise return the last number found (LLM output typically has the answer at the end, page numbers at the start).

---

### text.rs

```rust
/// Clean an LLM response for use as plain text.
///
/// Processing:
/// 1. Strip <think> blocks
/// 2. Trim whitespace
/// 3. Strip common LLM boilerplate prefixes:
///    "Sure!", "Here's...", "Of course!", "Certainly!", etc.
///
/// Returns the cleaned text or EmptyResponse if nothing remains.
pub fn parse_text(response: &str) -> Result<String, ParseError>
```

#### Implementation
- Preprocess (strip think, trim)
- Check a list of common prefixes and strip the first one found:
  - "Sure! " / "Sure, " / "Sure.\n"
  - "Of course! " / "Of course, " / "Of course.\n"
  - "Certainly! " / "Certainly, " / "Certainly.\n"
  - "Here's " / "Here is " (strip up to and including the next newline or colon)
  - "Absolutely! " / "Absolutely, "
- Trim again after stripping
- Return error if empty

#### Test Cases
```
clean_text:         "Paris is the capital."       → "Paris is the capital."
with_think:         "<think>...</think>Paris."    → "Paris."
sure_prefix:        "Sure! Paris is the capital." → "Paris is the capital."
heres_prefix:       "Here's the answer:\nParis."  → "Paris."
empty_after_strip:  "<think>just thinking</think>" → error
already_clean:      "No prefix here."             → "No prefix here."
```

---

### mod.rs — Public API

```rust
//! # LLM Output Parser
//!
//! Production-grade parser for extracting structured data from LLM responses.
//! Handles think blocks, markdown fences, malformed JSON, and other real-world
//! model output patterns without requiring an additional LLM call.
//!
//! ## Parsers Available
//!
//! | Parser | Use Case |
//! |--------|----------|
//! | [`parse_json`] | Extract typed JSON structs |
//! | [`parse_json_value`] | Extract untyped JSON |
//! | [`parse_string_list`] | Extract cleaned string lists (tags, items) |
//! | [`parse_string_list_raw`] | Extract string lists without cleaning |
//! | [`parse_xml_tag`] | Extract content from an XML tag |
//! | [`parse_xml_tags`] | Extract content from multiple XML tags |
//! | [`parse_choice`] | Extract a choice from valid options |
//! | [`parse_number`] | Extract a numeric value |
//! | [`parse_number_in_range`] | Extract a bounded numeric value |
//! | [`parse_text`] | Clean text extraction |
//! | [`parse_yaml`] | Extract typed YAML (feature: `yaml`) |
//!
//! ## Shared Utilities
//!
//! | Function | Purpose |
//! |----------|---------|
//! | [`strip_think_tags`] | Remove `<think>` blocks from text |
//! | [`try_repair_json`] | Fix common LLM JSON errors |

pub mod error;
pub mod extract;
pub mod repair;
pub mod json;
pub mod list;
pub mod xml;
pub mod choice;
pub mod number;
pub mod text;

#[cfg(feature = "yaml")]
pub mod yaml;

// Re-export all public functions at module level
pub use error::ParseError;
pub use extract::{strip_think_tags, preprocess};
pub use repair::try_repair_json;
pub use json::{parse_json, parse_json_value};
pub use list::{parse_string_list, parse_string_list_raw};
pub use xml::{parse_xml_tag, parse_xml_tags};
pub use choice::parse_choice;
pub use number::{parse_number, parse_number_in_range};
pub use text::parse_text;

#[cfg(feature = "yaml")]
pub use yaml::parse_yaml;
```

---

## ollama-vision Integration (Final Step)

### Cargo.toml Change

Add llm-pipeline as a dependency with default-features disabled:
```toml
[dependencies]
llm-pipeline = { path = "../llm-pipeline", default-features = false }
```

### src/parser.rs

Replace the entire contents with a thin re-export shim:
```rust
//! LLM response parser — delegates to llm-pipeline's output_parser module.
//!
//! This module re-exports the parsing functions from llm-pipeline
//! for backward compatibility.

pub use llm_pipeline::output_parser::ParseError;
pub use llm_pipeline::output_parser::strip_think_tags;
pub use llm_pipeline::output_parser::parse_string_list as parse_tags;
```

### src/lib.rs

Keep the same re-exports. Since `parse_tags` and `strip_think_tags` are still re-exported from `parser`, the public API doesn't change. The names `parse_tags`, `strip_think_tags`, and `ParseError` remain available at the crate root.

### Verify

- `cargo test` in ollama-vision must pass with zero test changes
- `cargo build --examples` must succeed
- All 24 original tests now live in llm-pipeline's `list.rs` tests
- ollama-vision's `parser.rs` tests are removed (they'd test re-exports of the same code)

---

## Test Count Targets

| Module | Minimum Tests |
|--------|--------------|
| extract.rs | 12 (strip_think, code_blocks, find_bracketed) |
| repair.rs | 14 (one per repair type + mixed + no-op + unrepairable) |
| json.rs | 11 (direct, think, codeblock, prose, nested, repair paths) |
| list.rs | 29 (24 ported from parser.rs + 5 new) |
| xml.rs | 11 (simple, multi, nested, missing_close, not_found) |
| yaml.rs | 3 (feature-gated, basic coverage) |
| choice.rs | 10 (exact, bold, quoted, prose, case, first_wins, no_match) |
| number.rs | 11 (int, float, fraction, prose, labeled, range, negative) |
| text.rs | 6 (clean, think, prefix strip, empty) |
| **Total** | **~107** |
