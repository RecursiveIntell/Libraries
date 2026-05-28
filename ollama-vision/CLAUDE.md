# CLAUDE.md — Output Parser Extraction & Integration

## Project Context

This session extracts the LLM output parser from `ollama-vision` into `llm-pipeline` as a full-featured `output_parser` module, then updates `ollama-vision` to depend on it. This is a coordinated change across two libraries.

## Directory Layout

```
~/Coding/Libraries/
├── ollama-vision/          ← YOU ARE HERE (session root)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── parser.rs       ← SOURCE: the 7-strategy parser to extract
│   │   ├── captioner.rs    ← CONSUMER: uses strip_think_tags
│   │   ├── tagger.rs       ← CONSUMER: uses parse_tags
│   │   └── types.rs
│   ├── examples/
│   ├── Cargo.toml
│   └── CLAUDE.md           ← THIS FILE
│
├── llm-pipeline/           ← TARGET: parser module goes here
│   ├── src/
│   │   ├── lib.rs          ← ADD: pub mod output_parser;
│   │   └── ...             ← existing pipeline modules (DO NOT MODIFY)
│   ├── Cargo.toml          ← ADD: serde_yaml optional dep, feature flags
│   └── ...
```

## Critical Rules

### DO NOT
- Modify any existing llm-pipeline modules (engine, stages, streaming, ollama, prompts, etc.) — only ADD the new `output_parser` module and update `lib.rs` to expose it
- Delete `ollama-vision/src/parser.rs` until the replacement is verified working
- Break ollama-vision's existing public API — `parse_tags` and `strip_think_tags` must remain re-exported from the crate root
- Add unnecessary dependencies — the parser module must work with only `serde`, `serde_json`, and `thiserror` (which llm-pipeline already has). `serde_yaml` is the ONLY new dep and MUST be optional behind a feature flag
- Use regex crate — all parsing is manual string operations (consistent with existing parser style)

### DO
- Read `ollama-vision/src/parser.rs` first — understand every strategy before writing anything
- Read `llm-pipeline/src/lib.rs` and `llm-pipeline/Cargo.toml` first — understand the existing module structure and dependencies before adding anything
- Run `cargo build` and `cargo test` in llm-pipeline after each checkpoint
- Run `cargo build` and `cargo test` in ollama-vision after the final integration step
- Run `cargo clippy -- -D warnings` in both crates before declaring any checkpoint complete
- Write doc comments (`///`) on every public function, struct, enum, and trait
- Preserve the existing code style: `thiserror` for error types, builder patterns where appropriate, `#[cfg(test)] mod tests` inline in each module

### Code Style (match existing patterns)
- Error types use `thiserror` derive macros
- Public API functions return `Result<T, ParseError>`
- Test modules are `#[cfg(test)] mod tests { use super::*; ... }` at the bottom of each file
- Tests are descriptive: `#[test] fn parse_json_from_markdown_code_block()`
- No `unwrap()` in library code, only in tests
- Use `&str` inputs and owned return types (consistent with existing `parse_tags`)

## Checkpoint Workflow

This session has 6 checkpoints. At each checkpoint:
1. Run `cargo build` in the affected crate
2. Run `cargo test` in the affected crate  
3. Run `cargo clippy -- -D warnings` in the affected crate
4. Announce: "CHECKPOINT N COMPLETE — [summary]. Proceeding to checkpoint N+1."
5. Wait briefly — if I say "hold", stop and discuss before continuing

## File Reference

See these companion files for detailed specifications:
- `ARCHITECTURE.md` — Full module structure, function signatures, and implementation details
- `PARSER_REFERENCE.md` — Annotated copy of the existing parser.rs being extracted
