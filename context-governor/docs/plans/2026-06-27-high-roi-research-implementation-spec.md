# High-ROI Research Implementation Plan

> For Hermes: implement this plan directly in `context-governor` using strict TDD, then run a second recursive audit/implementation pass before final reporting.

Goal: turn the 2026-06-27 external research sweep into deterministic, crate-agnostic primitives that any agent host can call without adding LLM calls or host-specific behavior.

Architecture: Add a new `high_roi` module to `context-governor` and re-export it from `src/lib.rs`. The module contains pure Rust scoring/audit/evaluation helpers for governed shared memory, MCP tool-surface poisoning, compression-boundary relinking, leakage-free RAG evaluation, conflict screening, GraphRAG route gating, agent-memory module metrics, semantic KV retention planning, and provenance projection receipts. Existing compaction remains unchanged except for reusable detection functions where appropriate.

Tech Stack: Rust 2021, serde, std collections, existing context-governor crate. No network calls. No LLM calls. No new dependencies unless tests prove unavoidable.

Current state evidence:
- Repo: `/home/sikmindz/Coding/Libraries/context-governor`
- Preflight command: `cargo test --all-targets`
- Preflight result: passed, 31 tests across library integration tests/examples, 0 failures.
- Parent workspace is dirty before this work; do not touch unrelated files.
- `context-governor` is a standalone crate with its own `[workspace]`, not a member of the parent workspace, so verification must run from the crate directory.

## Acceptance gates

1. `cargo test --all-targets` passes in `/home/sikmindz/Coding/Libraries/context-governor`.
2. `cargo fmt --check` passes in `/home/sikmindz/Coding/Libraries/context-governor`.
3. New tests prove each research finding became executable behavior.
4. The crate remains deterministic and host-agnostic: no HTTP, no subprocess, no model calls.
5. The second recursive pass adds at least one gap discovered by reviewing pass-one implementation, or explicitly records no remaining code gap with evidence.

## Task 1: RED tests for high-ROI primitives

Objective: encode the research sweep as failing behavior tests before production code.

Files:
- Create: `tests/high_roi_research.rs`

Tests:
- governed shared-memory harness scores leakage, stale propagation, contradiction persistence, and provenance collapse.
- MCP tool-surface audit flags threshold-style split payloads only when fragments combine across tools.
- compression-boundary audit scans both source fragments and compressed summary.
- leakage-free RAG evaluation requires closed-book baseline failure/degradation before retrieval gain can be certified.
- conflict screening detects cheap lexical/numeric disagreement and returns `needs_expensive_review`.
- route gating keeps simple lookup on flat retrieval and escalates multi-hop/contradiction/synthesis/temporal queries.
- agent memory module metrics require representation, organization, retrieval/update, and lifecycle coverage.
- projection receipt records canonical source IDs, projection kind, derivation hash, and staleness.

Run: `cargo test --test high_roi_research -- --nocapture`
Expected RED: compile failure from missing APIs.

## Task 2: GREEN high_roi module

Objective: add minimal deterministic implementation for all RED tests.

Files:
- Create: `src/high_roi.rs`
- Modify: `src/lib.rs` to `pub mod high_roi; pub use high_roi::*;`

Implementation rules:
- Use enums/structs with serde derives for receipt surfaces.
- Use transparent reason strings suitable for audit receipts.
- Keep scoring deterministic and explainable.
- Do not mutate existing compaction behavior unless tests require it.

Run: `cargo test --test high_roi_research -- --nocapture`
Expected GREEN: all new tests pass.

## Task 3: Full crate verification

Objective: prove no regressions.

Run:
- `cargo test --all-targets`
- `cargo fmt --check`

Expected: pass.

## Task 4: Recursive pass 2 audit

Objective: review pass-one implementation against the original research sweep and identify missing executable pieces.

Checklist:
- Governed shared memory: leakage/stale/contradiction/provenance all scored.
- MCP ShareLock: combined tool-surface threshold risk, not just per-tool keyword scan.
- Relinking: source and compressed summary are checked separately.
- Agent-native memory eval: module-level metrics exist.
- SeedRG/leakage-free eval: closed-book baseline gate exists.
- ConflictRAG: cheap first-stage screen exists.
- GraphRAG: route gating exists.
- CompressKV: semantic token-retention plan exists with hosted-API boundary note.
- VISTA/projections: provenance-linked projection receipt exists.

If a gap exists, add RED test then GREEN implementation. If no gap exists, add a regression test proving the weakest boundary.

## Task 5: Final receipts

Objective: report exact files changed and commands run.

Final output must include:
- plan path
- source/test files changed
- verification commands and outputs
- explicit boundary: this implements deterministic research-derived primitives, not full paper reproduction or external benchmark reproduction.
