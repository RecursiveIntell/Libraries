# Output Parser Extraction — Full Session Prompt

Read `CLAUDE.md`, `ARCHITECTURE.md`, and `PARSER_REFERENCE.md` in this directory before doing anything else. Those three files are your complete specification. Do not start writing code until you have read all three.

## What We're Doing

Extracting the 7-strategy LLM output parser from `ollama-vision/src/parser.rs` into `llm-pipeline/src/output_parser/` as a full-featured module, adding 5 new parser types (JSON, XML, YAML, choice, number, text) plus a deterministic JSON repair engine, then updating `ollama-vision` to depend on `llm-pipeline` for its parsing. The existing `parse_tags` and `strip_think_tags` public API must remain backward-compatible in ollama-vision.

## Session Structure — 6 Checkpoints

Work through these in order. At each checkpoint: build, test, clippy, then announce completion and pause briefly for my go/no-go.

### Checkpoint 1: Reconnaissance & Scaffolding
1. Read `ollama-vision/src/parser.rs` (the source being extracted)
2. Read `llm-pipeline/src/lib.rs` and `llm-pipeline/Cargo.toml` (the target)
3. Identify llm-pipeline's existing module structure — list what's there so I can verify
4. Create the `llm-pipeline/src/output_parser/` directory and `mod.rs` with the module declarations (all submodules)
5. Create `error.rs` with the `ParseError` enum
6. Add `pub mod output_parser;` to `llm-pipeline/src/lib.rs`
7. Add `serde_yaml` as optional dependency behind `yaml` feature in `llm-pipeline/Cargo.toml`
8. Verify: `cargo build` in llm-pipeline (empty modules are fine, just need structure to compile)

### Checkpoint 2: Core Extraction (extract.rs + list.rs)
1. Build `extract.rs`: port `strip_think_tags` (add `<thinking>` variant), build `preprocess`, `extract_code_block`, `extract_code_block_for`, `find_bracketed`
2. Build `list.rs`: port `parse_tags` → `parse_string_list` using extract.rs functions, port `clean_tags`, add `parse_string_list_raw`, expand object key checking to "tags"/"items"/"results"/"list"
3. Port ALL 24 tests from ollama-vision's parser.rs into the appropriate modules (strip_think tests → extract.rs, everything else → list.rs)
4. Add new tests for generalized extraction functions
5. Verify: `cargo test` in llm-pipeline — all ported tests pass, new tests pass

### Checkpoint 3: JSON Repair + JSON Parser (repair.rs + json.rs)
1. Build `repair.rs`: all 7 repair strategies, validate output with serde_json before returning
2. Build `json.rs`: `parse_json<T>` and `parse_json_value` using extract.rs + repair.rs
3. Write tests: 14+ for repair, 11+ for json
4. Verify: `cargo test` in llm-pipeline

### Checkpoint 4: Remaining Parsers (xml.rs, choice.rs, number.rs, text.rs, yaml.rs)
1. Build `xml.rs`: `parse_xml_tag`, `parse_xml_tags`
2. Build `choice.rs`: `parse_choice` with word-boundary matching
3. Build `number.rs`: `parse_number<T>`, `parse_number_in_range`
4. Build `text.rs`: `parse_text` with boilerplate stripping
5. Build `yaml.rs`: `parse_yaml<T>` behind feature flag
6. Write all tests per ARCHITECTURE.md specs
7. Wire up all re-exports in `mod.rs`
8. Verify: `cargo test` and `cargo test --features yaml` in llm-pipeline

### Checkpoint 5: Documentation & Polish
1. Ensure every public item has `///` doc comments
2. Add module-level `//!` docs to each file
3. Run `cargo clippy -- -D warnings` — fix all warnings
4. Run `cargo doc --no-deps` — verify docs build cleanly
5. Verify: clippy clean, docs build, all tests still pass

### Checkpoint 6: ollama-vision Integration
1. Add `llm-pipeline = { path = "../llm-pipeline", default-features = false }` to ollama-vision's Cargo.toml
2. Replace `ollama-vision/src/parser.rs` contents with re-export shim (see ARCHITECTURE.md)
3. Verify `tagger.rs` and `captioner.rs` compile without changes (they use `crate::parser::parse_tags` and `crate::parser::strip_think_tags` which are now re-exports)
4. Verify: `cargo build` and `cargo test` in ollama-vision — everything passes
5. Run `cargo clippy -- -D warnings` in ollama-vision

## Key Constraints Reminder

- The parser module uses ONLY `serde`, `serde_json`, `thiserror`, and optionally `serde_yaml`. No regex. No other crates.
- `find_bracketed` returns `&str` (borrows from input) — zero allocation for the extraction step
- `repair.rs` is pure string manipulation — no serde until final validation
- `parse_string_list` must produce IDENTICAL output to the old `parse_tags` for all existing test inputs
- ollama-vision's public API (`parse_tags`, `strip_think_tags`, `ParseError`) must not change

Begin by reading the three reference files, then start Checkpoint 1.
