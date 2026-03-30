# PROMPT.md — V29 Execution Prompt

## Session initialization

You are implementing the V29 Remediation Pack for the RecursiveIntell library stack. Read CLAUDE.md first, then the numbered orientation documents in order.

## Start command

```
Read CLAUDE.md, then 00_START_HERE.md, then 02_MASTER_ISSUE_MATRIX.md, then 03_IMPLEMENTATION_PLAYBOOK.md.
Begin with Phase 1. Work issue-by-issue in the order specified by the playbook.
After each issue: run cargo check --workspace, commit with fix(ISSUE-ID): description.
After each phase: run cargo test --workspace.
Report status after each phase completion.
```

## Phase-specific prompts

### Phase 1 session
```
Implement Phase 1 of the V29 pack: TRUTH-001, GATE-001, DOC-002.
Start with GATE-001 (independent).
Then TRUTH-001 + DOC-002 (combined README rewrite).
See 04_EXACT_FILE_TOUCH_MAP.md for every file to touch.
See 05_TEST_AND_CONFORMANCE_PLAN.md for acceptance tests.
```

### Phase 2 session
```
Implement Phase 2 of the V29 pack: TRUTH-002, TRUTH-003, GATE-002, WIRE-001, DOC-001.
Start with TRUTH-002 (archive cleanup) and TRUTH-003 (manifest) — these are fast.
Then GATE-002 (budget script fix).
Then WIRE-001 (56 serde annotations — work crate by crate, cargo check after each).
Then DOC-001 (doc comments — longest task, can be time-boxed).
```

### Phase 3 session
```
Implement Phase 3 of the V29 pack: TRUTH-004, GATE-003, WIRE-002, CONV-001, GOV-001, PERF-001, SAFE-001.
All issues are independent. Work in order listed.
```

### Final gate session
```
Run the full gate verification:
make gate
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
Report all results. If any fail, diagnose and fix.
Then regenerate the clean archive excluding target-* directories.
```
