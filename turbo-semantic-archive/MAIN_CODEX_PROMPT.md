# MAIN CODEX PROMPT — TurboQuant × Semantic-Memory Super-Pass

You are operating on the user's Rust projects:

- `turbo-quant`: canonical vector codec crate.
- `semantic-memory`: canonical memory/search/projection crate inside the Libraries workspace.

Your task is to implement a safe, measurable, non-authoritative TurboQuant integration into semantic-memory.

This is a **super-pass**, not a small patch. Execute in phases. At each phase boundary, stop, report, and wait for the user's manual phase-injection prompt before continuing.

## Source basis

Observed source-package facts:

- `turbo-quant` package: small crate, 16 included files, 9 Rust files, no validation findings.
- `semantic-memory` package: larger workspace package, 173 included files, 85 Rust files, includes `semantic-memory`, `stack-ids`, `forge-memory-bridge`, and `semantic-memory-forge`, no validation findings.
- Current `semantic-memory` has quantization/search/HNSW tests and episode/projection tests.
- Current `turbo-quant` has `polar`, `qjl`, `rotation`, `turbo`, and `kv` surfaces, plus determinism and inner-product tests.
- Current turbo-quant storage is not yet a clear storage win because QJL signs are stored as `i8`, polar angle indices are stored as `u16`, radii are `f32`, and dense rotation is expensive.

## End goal

Implement a staged integration:

1. Harden turbo-quant as a real codec crate:
   - compact wire artifacts;
   - profile/version/digest;
   - query-prepared scoring;
   - cosine estimate;
   - corruption/profile mismatch handling;
   - storage accounting;
   - deterministic tests and golden fixtures.

2. Add a `semantic-memory` vector codec abstraction:
   - `RawF32Codec`;
   - current SQ8 codec preserved;
   - optional `TurboQuantCodec` adapter behind a feature gate;
   - no local reimplementation of TurboQuant math.

3. Add shadow-mode storage/evaluation:
   - raw and existing behavior remain authoritative;
   - TurboQuant encoded artifacts stored separately when enabled;
   - encode/search/evaluation receipts or JSON artifacts emitted;
   - top-k agreement, recall@k, score correlation, latency, and byte-size recorded.

4. Add approximate result disclosure:
   - codec family/profile digest;
   - approximation class;
   - rerank status;
   - degradation/fallback flags;
   - no silent use of approximate scores.

5. Add tests and conformance checks:
   - no shadow codec;
   - optional feature compilation;
   - existing tests still pass;
   - turbo-quant corruption/profile mismatch tests;
   - semantic-memory shadow-mode/evaluation tests;
   - search result disclosure tests.

## Hard boundaries

- Do not make TurboQuant default.
- Do not remove existing SQ8.
- Do not invent local TurboQuant math in semantic-memory.
- Do not add absolute Cargo path dependencies.
- Do not proceed if `turbo-quant` cannot be resolved as the canonical crate.
- Do not silently widen semantics or hide approximate scoring.

## Phase list

Execute the phase prompts in `PHASE_PROMPTS/`:

0. Preflight and source-basis freeze.
1. TurboQuant codec hardening.
2. semantic-memory VectorCodec abstraction.
3. Optional TurboQuant adapter and Cargo integration.
4. Shadow-mode encode/persist/evaluate path.
5. Search disclosure and explained result path.
6. Benchmark/evaluation/conformance harness.
7. Documentation, cleanup, final audit.

Stop after each phase and emit the required phase report.

## Required final deliverables

At the end, provide:

- changed file list;
- new API list;
- new feature flags;
- migrations/schema changes;
- exact commands run;
- test results;
- compression/evaluation results if runnable;
- known limitations;
- explicit statement that TurboQuant is not default unless gates prove otherwise;
- instructions for enabling the feature;
- rollback instructions;
- next pass recommendations.
