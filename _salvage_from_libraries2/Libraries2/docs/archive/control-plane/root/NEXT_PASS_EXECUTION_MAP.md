# Next Pass Execution Map

This is the shortest possible “what Claude should do next” document derived from the master issue matrix.

## First cut: do these five things before anything cosmetic

1. **Kill `agent-graph` primitive drift.**
   - Replace local `AttemptId = String` with canonical identity handling.
   - Reduce legacy trace down-conversion in core graph execution.

2. **Harden `semantic-memory` importer law.**
   - Replace ad hoc batch field scraping with a stronger decode/validation path.
   - Resolve version-law handling deliberately.
   - Confirm or fix the evidence-ref derivation target identity smell.

3. **Land runtime truth, not just runtime honesty.**
   - True temporal retrieval.
   - Stronger scope pushdown.
   - Tighter non-authoritative identity/cache fencing.

4. **Make `job-queue` retry lineage durable.**
   - Stop reconstructing canonical attempt identity purely from legacy counters where avoidable.
   - Prove replay/re-enqueue lineage with tests.

5. **Add the cross-crate proof suite.**
   - Forge export → bridge → memory import → runtime query.
   - Explicit evidence dereference behavior.
   - Retry/replay lineage preservation.
   - Bounded invalidation/recompute.

## Second cut: remove the debt most likely to fossilize

1. `semantic-memory` public compat `TraceId` and old import path.
2. `LLM-Pipeline` public `TraceId` and dual `ExecCtx` trace surface.
3. `agent-graph` legacy-first event surface.
4. `Tauri-Queue` legacy emitted trace defaults.

## Third cut: tighten semantics so the docs stop lying by accident

1. Clarify bridge import/export version naming.
2. Clarify `recorded_at` ownership semantics.
3. Add executor-path lineage proof in `AI-Batch-Queue`.

## What Claude should not do

- Do **not** invent new wrapper types to dodge migration work.
- Do **not** break dependency direction to make the importer “more typed.”
- Do **not** destabilize `forge-memory-bridge` unnecessarily.
- Do **not** let runtime become identity or persistence authority.
- Do **not** replace honest degradation with silent fallback.

