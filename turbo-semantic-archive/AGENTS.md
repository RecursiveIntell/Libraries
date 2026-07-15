# AGENTS.md — TurboQuant × Semantic-Memory Super-Pass Doctrine

This run is not a feature sprint. It is a bounded, evidence-bearing integration pass.

## Non-negotiable constraints

1. **No shadow codec.**
   - `semantic-memory` must not reimplement TurboQuant math locally.
   - `turbo-quant` remains the canonical owner of TurboQuant, PolarQuant, QJL, rotation, encoding, decoding, and compressed scoring.
   - `semantic-memory` may define a generic `VectorCodec` trait and an adapter wrapper, but the adapter must call the real `turbo-quant` crate.

2. **No default promotion.**
   - TurboQuant must not become the default authoritative search/storage path in this pass.
   - It must start as optional, feature-gated, shadow-mode, evaluation-backed capability.

3. **Raw/f32 remains authoritative.**
   - Existing raw embeddings and existing SQ8 behavior must remain intact.
   - TurboQuant results are approximate unless reranked or verified against raw f32.

4. **Approximation must be visible.**
   - Any result scored by TurboQuant must carry codec profile, approximation class, degradation markers, and whether f32 rerank occurred.

5. **Receipts or it did not happen.**
   - Encoding, scoring, evaluation, corruption detection, fallback, and degraded paths must emit structured receipts or persisted evaluation artifacts.

6. **No silent schema widening.**
   - Codec profiles and encoded vector artifacts must be versioned.
   - Profile mismatch, dimension mismatch, unsupported codec, or corrupt payload must fail explicitly.

7. **No brittle absolute paths.**
   - Do not add absolute path dependencies.
   - Preferred: move/add `turbo-quant` as a sibling workspace crate under `/home/sikmindz/Coding/Libraries/turbo-quant`.
   - If not available, stop and report. Do not paste-copy TurboQuant code into semantic-memory.

8. **Preserve existing semantic-memory behavior.**
   - Existing tests must pass.
   - Existing HNSW/vector-only/hybrid search APIs must not regress.
   - Existing quantization tests must remain meaningful.

9. **Artifact-law fit.**
   - Codec output is not truth; it is a derived artifact.
   - `semantic-memory` owns projection/query/evaluation records.
   - `turbo-quant` owns mathematical codec semantics.

10. **Stop on ambiguity.**
    - If ownership, path layout, schema semantics, or search authority is unclear, halt that subtask and emit a precondition/ambiguity report instead of guessing.

## Required reporting at each phase

Each phase report must include:

- changed files;
- new public APIs;
- migrations/schema changes;
- tests added/changed;
- commands run and results;
- invariant checklist;
- unresolved risks;
- whether any manual injection constraints were violated.

## Forbidden shortcuts

- Do not invent a `turbo_quant` module inside `semantic-memory`.
- Do not store encoded vectors without a profile digest.
- Do not score approximate vectors without exposing approximation.
- Do not remove existing SQ8 tests.
- Do not make TurboQuant default before benchmark/evaluation evidence.
- Do not claim compression improvement until `encoded_bytes()` proves it.
- Do not use `unwrap`, `expect`, `todo!`, `unimplemented!`, or `dbg!` in production code.
- Do not “fix” compile errors by deleting tests or weakening assertions.
