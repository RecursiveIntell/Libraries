# CLAUDE.md — Agent Instructions for V29 Remediation Pack

## Identity

You are implementing the V29 Unified Hostile Audit Remediation Pack for the RecursiveIntell library stack — a 30-crate Rust workspace (~116K lines) targeting DARPA CLARA submission by April 10, 2026.

This pack was produced by merging two independent hostile audits (Claude 10-angle and GPT 10-lens) against `libraries-source-clean-20260330.zip`. Every issue has exact file and line evidence.

## Orientation Documents

Read these before writing any code:

1. `00_START_HERE.md` — Context and conformance rules
2. `01_MASTER_ISSUE_TENSOR.json` — All 16 issues with evidence and acceptance criteria
3. `02_MASTER_ISSUE_MATRIX.md` — Priority and phase assignments
4. `03_IMPLEMENTATION_PLAYBOOK.md` — Phase order, dependency graph, execution rules
5. `04_EXACT_FILE_TOUCH_MAP.md` — Every file to create or modify, by issue
6. `05_TEST_AND_CONFORMANCE_PLAN.md` — Required tests per issue
7. `06_RISK_REGISTER.md` — What can go wrong, mitigations, forbidden shortcuts

## Execution Rules

### General

- Work in phase order: Phase 1 → Phase 2 → Phase 3 → Phase 4.
- Within Phase 1, resolve TRUTH-001 before DOC-002 (README rewrite depends on knowing canonical snapshot).
- GATE-001 is independent and can be done first.
- Commit after each issue with message format: `fix(ISSUE-ID): brief description`
- Run `cargo check --workspace` after every issue.
- Run `cargo test --workspace` after every phase.
- Never batch untested changes across phases.

### Rust Conventions (project-specific)

- `BTreeMap` over `HashMap` everywhere (exception: HNSW index, documented)
- Typed ID newtypes from `stack-ids` — never raw strings for identity
- `thiserror` for all error types
- No `unwrap()` in production code (test code is fine)
- `tracing` over `println!` in library crates (`println!` OK in forge-pilot CLI output)
- `#[serde(rename_all = "snake_case")]` on all enums with Serialize derive
- `schemars::JsonSchema` derive on all public types
- `#[serde(default, skip_serializing_if = "Option::is_none")]` on optional fields
- `#[serde(default, skip_serializing_if = "Vec::is_empty")]` on vec fields

### What NOT to Do

1. **Do not weaken gates to get green.** Make the underlying surface pass.
2. **Do not add rename_all without running cargo test.** Serde changes can break fixture deserialization.
3. **Do not move archived docs without checking for references.** `grep -rn 'filename' .` first.
4. **Do not combine doc-only changes with code changes.** Separate commits.
5. **Do not create new schema versions or artifact families.** This pack closes gaps only.
6. **Do not rename crates.** Compatibility names are documented and intentional.
7. **Do not change wire format of existing serialized types** unless the issue explicitly requires it.
8. **Do not edit CANONICAL_STACK_SPEC documents.** Constitutional documents are out of scope.

## Issue-Specific Guidance

### TRUTH-001 + DOC-002: README and Truth Unification

These are best done together. The README rewrite resolves both issues.

The new README.md should contain:
- Project name and one-line description
- What the OODA governance orchestrator does
- 3-tier crate architecture diagram (text)
- Build instructions (cargo build/test/clippy + make gate)
- Link to canonical stack spec
- No references to remediation packs, audits, or pack versions

### GATE-001: Permit Type Fix

The check_commit_permit_paths.py script has one primary failing pattern:

```python
# CURRENT (broken):
r"permit:\s*Option<&ExecutionPermit>"
# SHOULD BE:
r"permit:\s*Option<&ToolExecutionPermit>"
```

Note: The `execute_plan` signature in `forge-pilot/src/act.rs` correctly uses `ExecutionPermit` (from `verification-policy`). This is a DIFFERENT type from `ToolExecutionPermit` (from `llm-tool-runtime`). The script needs to check for the RIGHT type in each location.

### WIRE-001: Serde rename_all

This is a bulk mechanical change. For each identified enum:

```rust
// BEFORE:
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ActionFamily {

// AFTER:
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionFamily {
```

Work crate-by-crate and run `cargo check` after each crate.

If any test fails after adding `rename_all`, it means test fixtures contain PascalCase values. Fix the test fixtures to use snake_case.

### DOC-001: Doc Coverage

For struct/enum/trait docs, use this template:

```rust
/// One-line purpose description.
///
/// Additional context if the type crosses crate boundaries or has
/// non-obvious semantics.
pub struct GovernanceObservation {
```

For module-level docs:

```rust
//! Governance surface checks for the forge-pilot OODA loop.
//!
//! This module evaluates governance artifact state during observation
//! and gates execution during the act phase.
```

Do NOT generate implementation details. Focus on purpose and contract.

## Crate Authority Map (reference)

| Tier | Crates | Role |
|------|--------|------|
| Tier 1 | constraint-compiler, kernel-oracles | Hardest to replicate |
| Tier 2 | semantic-memory, forge-engine (living-memory), knowledge-runtime, forge-pilot | Core orchestration |
| Tier 3 | stack-ids, llm-tool-runtime, profile-runtime, forge-memory-bridge, semantic-memory-forge | Support and bridge |
| Governance | assurance-runtime, attestation-exchange, authority-delegation, constitutional-memory, continuity-runtime, effect-runtime, mechanism-runtime | Governance artifact types |
| Verification | verification-control, verification-policy, verification-calibration, verification-adjudication | Verification pipeline |
| Other | contract-schema-gen, discovery-portfolio, federated-settlement, spec-execution, remote-oracle-admission, recursive-kernel-core, kernel-execution, kernel-conformance | Specialized surfaces |
