# constraint-compiler

Deterministic projection-to-inference graph compiler for the canonical lane.

## Scope

This crate compiles canonical projection import batches into bounded inference graph artifacts: nodes, hyperedges, constraints, invalidation cones, degradations, and oracle candidates.

## Non-goals

- no authority over source truth
- no runtime repair or scheduler policy
- no fabrication of semantics from thin exports

## Current maturity

Phase 0 compiler defects are closed and gate-proven. Phase 2 hardening remains open around nuisance normalization, inferential-only graph construction, and tighter oracle slice selection.
