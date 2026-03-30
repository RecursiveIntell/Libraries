# AGENTS.md — V29 Agent Coordination

## Execution Model

This pack is designed for implementation by a single agent (Claude Code or equivalent) working sequentially through phases. No multi-agent coordination is required.

## Agent capabilities required

- Rust source file editing (str_replace / create_file)
- Bash command execution (cargo check, cargo test, script execution)
- File system operations (mkdir, mv, cp)
- JSON file editing
- Markdown file creation and editing

## Session strategy

### Session 1: Phase 1 (fast — 30min)
Fix GATE-001, TRUTH-001, DOC-002. Three commits. Run cargo check after each.

### Session 2: Phase 2 core (1–2hr)
Fix TRUTH-002, TRUTH-003, GATE-002. Archive cleanup and script fixes.

### Session 3: Phase 2 wire format (1hr)
Fix WIRE-001. 56 serde annotations, crate by crate. Cargo check after each crate. Cargo test after all.

### Session 4: Phase 2 docs (2–3hr)
Fix DOC-001. Doc comment pass on supported-lane crates. This is the longest single task. Can be time-boxed to the 5 highest-priority crates if deadline pressure is severe.

### Session 5: Phase 3 (1–2hr)
Fix all Phase 3 issues. All independent.

### Session 6: Final gate (30min)
Run make gate, cargo check/test/clippy/doc. Fix any remaining failures. Generate clean archive.

## Total estimated time: 6–10 hours with AI assist

## Context window management

Each session should begin by reading:
1. `CLAUDE.md` (always)
2. `02_MASTER_ISSUE_MATRIX.md` (for current phase context)
3. `04_EXACT_FILE_TOUCH_MAP.md` (for the specific files in the current phase)

Do NOT load the full tensor JSON or both audit reports into context — they are reference material, not execution instructions.
