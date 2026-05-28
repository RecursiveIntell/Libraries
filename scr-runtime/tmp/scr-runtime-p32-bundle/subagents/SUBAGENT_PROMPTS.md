# Read-only subagent prompts

Use these only after preflight. Each subagent must be read-only unless explicitly promoted by the main agent.

## 1. Schema/Rust parity subagent

Inspect `crates/scr-kernel`, generated schemas, schema scripts, and CLI schema generation. Find every place schema validation is weaker than Rust validation. Produce exact file/line findings and tests needed. Do not edit.

## 2. Evaluator semantics subagent

Inspect `crates/scr-reference`. Determine whether proposed action/requested effect materially affect decisions. Find opaque-ref scanning, weak authority/evidence logic, missing rollback/owner gates, and candidate trace gaps. Do not edit.

## 3. Control-pack/gates subagent

Inspect `.codex`, `.agents`, scripts, run docs, and prompts. Find inert hooks, null gates, invalid skills, stale run IDs, missing final receipt gates, and ways false completion can pass. Do not edit.

## 4. CLI/fixture subagent

Inspect `crates/scr-cli`, `fixtures`, `policies`, and test scripts. Find conflated generation/verification, missing negative fixtures, golden drift risks, explain/validate gaps. Do not edit.

## 5. External-boundary subagent

Inspect `docs/SOURCE_BASIS.md`, `docs/EXTERNAL_CRATE_BOUNDARY_MAP.md`, Cargo manifests, and workspace references. Identify any overclaim of external integration or duplicate owner semantics. Do not edit.

## Merge rule

Main agent must preserve disagreements. Do not smooth over subagent uncertainty. Convert findings into P32 issue matrix updates.
