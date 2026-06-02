# Phase 6 — Evaluation and Conformance Harness

## Goal

Produce evidence that TurboQuant helps or does not help, without pretending.

## Required changes

1. Add evaluation harness:
   - compare raw f32, SQ8, TurboQuant;
   - compute recall@k/top-k agreement;
   - score correlation if feasible;
   - byte-size accounting;
   - latency summary;
   - degradation/failure count.

2. Add test corpus:
   - small deterministic fixture using MockEmbedder;
   - enough records to exercise ranking disagreement;
   - store outputs in `target/vector-codec-evals/` or DB eval table.

3. Add scripts/checks:
   - no shadow codec;
   - no absolute path deps;
   - codec profile required;
   - approximate disclosure required;
   - existing tests.

4. Run:
   - `cargo fmt --check`;
   - `cargo test` for turbo-quant;
   - `cargo test -p semantic-memory --features hnsw`;
   - `cargo test -p semantic-memory --features hnsw,turbo-quant-codec` if feature exists;
   - clippy where feasible.

## Acceptance

Do not claim TurboQuant is superior unless evaluation proves it for at least the fixture corpus and byte-size accounting.
