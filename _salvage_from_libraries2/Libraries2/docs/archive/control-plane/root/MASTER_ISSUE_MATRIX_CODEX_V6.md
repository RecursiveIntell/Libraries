# Master Issue Matrix — Codex V6

**Source snapshot:** `now1.zip`
**Method:** full tree inventory + deep static inspection of the core authority lane (`stack-ids`, `semantic-memory-forge`, `forge-memory-bridge`, `semantic-memory`, `knowledge-runtime`, `forge-engine`) + targeted static inspection of proof tests and control docs.
**Execution target:** Codex handoff for a **target-state completion pass**, not a redesign pass.
**Important caveat:** this is **static inspection only**. No successful full-workspace build or test run was executed in this environment because the Rust toolchain was not available here.

## What changed since the older matrices

The big migration fights are largely over:

- the root workspace exists,
- Forge owns the export envelope,
- the bridge owns transformation,
- `semantic-memory` owns durable imported projections,
- direct write-through is compatibility-only,
- and `knowledge-runtime` is more honest about degradations.

The remaining gaps are more specific and more dangerous:
**imported projections are stored but not yet the primary retrieval substrate; temporal and scope semantics are still degraded; `recorded_at` is currently authored in the wrong layer; and Forge export is still too semantically thin.**

## How to read this matrix

- **Target-state conformant now?** = whether the current code appears aligned with the canonical target-state contract.
- **Mandatory next pass?** = whether Codex should treat the row as part of the next closure pass.
- **Fossil risk** = how likely the current behavior is to become the real architecture by inertia if left alone.
- **Proof** = `confirmed-static`, `inferred-static`, or `control-doc-drift`.

## Priority summary

- **P0:** 5 rows — fix architecture-correctness gaps first
- **P1:** 4 rows — close the primary retrieval / causal / entity gaps next
- **P2:** 4 rows — burn down partial lineage / compat / durability ambiguity
- **P3:** 2 rows — scrub stale docs and misleading absolutes

## Matrix

| ID | Pri | Crate | File / Symbol | Class | Target-state conformant now? | Mandatory next pass? | Fossil risk | Proof | Issue | Required fix / acceptance |
|---|---|---|---|---|---|---|---|---|---|---|
| KR-101 | P0 | knowledge-runtime + semantic-memory | `knowledge-runtime/src/adapters/semantic_memory.rs`, `knowledge-runtime/tests/cross_crate_proof.rs::runtime_query_sees_imported_data`, `semantic-memory/src/lib.rs::query_projection_imports` | ARCH_GAP | no | yes | high | confirmed-static | Runtime retrieval still goes through `MemoryStore::search()`, and the current imported-data proof test adds a separate fact so the test does not prove imported projections are actually query-visible. | Add supported projection query APIs in `semantic-memory` and adapter methods in `knowledge-runtime`. **Accept:** a cross-crate proof imports a batch, adds no extra fact/chunk/message data, queries runtime, and gets imported claim/relation/episode results back. |
| KR-102 | P0 | knowledge-runtime + semantic-memory | `knowledge-runtime/src/runtime.rs`, `semantic-memory/src/types.rs::SearchResult` | IMPLEMENTATION_LAG | no | yes | high | confirmed-static | Temporal execution is still best-effort hybrid fallback because retrieval results do not carry enough temporal fields and runtime docs explicitly say so. | Implement projection-backed temporal retrieval or extend the result/query surface so `valid_from`, `valid_to`, and authoritative `recorded_at` are available for supported routes. **Accept:** as-of queries against imported versions return deterministically filtered results, and `strict_temporal` succeeds on supported projection routes. |
| KR-103 | P0 | knowledge-runtime | `src/runtime.rs`, `src/adapters/semantic_memory.rs` | IMPLEMENTATION_LAG | no | yes | high | confirmed-static | Scope dimensions beyond namespace are still warning/error only; they are not truly enforced during upstream retrieval. | Add full-scope filter pushdown or explicit projection-row post-filtering for projection-backed routes. **Accept:** domain/workspace/repo scoped projection queries return only matching rows; `strict_scope` passes on supported projection routes and fails only on genuinely unsupported routes. |
| BRG-101 | P0 | forge-memory-bridge + semantic-memory | `forge-memory-bridge/src/batch.rs`, `forge-memory-bridge/src/transform.rs`, `semantic-memory/src/lib.rs::import_projection_batch`, `forge-memory-bridge/tests/forge_bridge_memory_proof.rs`, `LATEST7.md` | SPEC_MISMATCH | no | yes | high | confirmed-static | `recorded_at` is currently stamped by the bridge at transform time and then persisted as if it were the importing store’s authoritative commit time. The canonical spec says imported projection rows must use memory import commit time. | Move authoritative `recorded_at` assignment into the `semantic-memory` import transaction. Keep `source_exported_at` and `transformed_at` as separate provenance fields. If per-record source timing must survive, rename it so it is not confused with authoritative store commit time. **Accept:** imported rows carry memory import commit time, bridge tests no longer assert `recorded_at == transformed_at`, and active docs stop teaching the old rule. |
| SMF-101 | P0 | semantic-memory-forge + forge-memory-bridge | `semantic-memory-forge/src/envelope.rs::ExportClaim`, `forge-memory-bridge/src/transform.rs` | ARCH_GAP | partial | yes | high | confirmed-static | Claim supersession lineage is still only claim-level (`supersedes_claim_id`), so the bridge correctly leaves `supersedes_claim_version_id` empty. Version-aware claim lineage remains incomplete end-to-end. | Extend the export schema to carry prior `claim_version_id` when known, or add an explicitly auditable importer-side resolution rule that never guesses. **Accept:** superseding claim versions round-trip with real version lineage and no synthetic IDs. |
| LIV-101 | P1 | forge-engine (`living-memory/living-memory`) | `src/export.rs::to_export_envelope_v1`, `src/lab/evidence.rs::EvidenceBundle` | ARCH_GAP | partial | yes | high | confirmed-static | Forge export rendering is canonical in shape but semantically thin: it emits one synthetic claim plus one evidence ref and leaves richer episode/relation/entity structure unused even though the envelope schema already supports it. | Enrich export rendering from `EvidenceBundle`, `ExperimentDiff`, and typed hypothesis structures into richer claim/relation/episode/evidence output where justified by the current data model. **Accept:** at least one proof covers importable rich envelopes, not just claim-text plus opaque evidence. |
| KR-104 | P1 | knowledge-runtime | `src/lib.rs`, `src/runtime.rs`, route/merge layers | IMPLEMENTATION_LAG | no | yes | medium-high | confirmed-static | Runtime docs still state that Forge causal projections are absent, and there is no projection-backed retrieval/merge path for imported episodes, relations, or evidence refs. | Add explicit projection retrieval legs and merge logic for imported causal records. Keep evidence opaque by default and only dereference through audit/explain flows. **Accept:** a causal query can be answered from imported projection rows with visible leg provenance. |
| KR-105 | P1 | knowledge-runtime | `src/entity/registry.rs` | IMPLEMENTATION_LAG | no | yes | medium | confirmed-static | Entity resolution is still exact canonical or exact alias only. The canonical contract expects fuzzy resolution or bounded candidate expansion. | Add bounded candidate expansion over imported aliases or equivalent narrow matching logic with explicit ambiguity handling. **Accept:** misspelled or variant mentions return bounded candidates without inventing new authority. |
| SM-101 | P1 | semantic-memory | `src/projection_storage.rs::query_claim_versions`, `src/lib.rs` | ARCH_GAP | no | yes | high | confirmed-static | Projection storage already has internal query helpers, but there is no supported public read API over imported claim/relation/episode/alias/evidence rows. | Promote a narrow projection query surface with scope filters, temporal filters, pagination, and stable result structs. **Accept:** public projection query methods exist and are used by runtime/adapters instead of dead-code private helpers. |
| SM-102 | P2 | semantic-memory | `src/lib.rs`, `src/projection_storage.rs::insert_derivation_edge` | PARTIAL_IMPLEMENTATION | partial | yes | medium | confirmed-static | Derivation edge coverage is currently extremely narrow (`evidence_ref -> claim/version`). That will be insufficient once richer imported projections drive bounded recomputation. | Add derivation edges for imported claim/relation/episode flows where lineage exists. **Accept:** invalidating a source artifact marks only the downstream derived artifacts that actually depend on it. |
| SM-103 | P2 | semantic-memory + stack-ids | `semantic-memory/src/lib.rs`, `semantic-memory/src/projection_storage.rs`, `stack-ids/src/trace.rs` | SPEC_ALIGNMENT_RISK | partial | yes | medium | confirmed-static | Durable imported rows and import logs currently persist `trace_id` only. That may be acceptable, but the durable trace representation is not explicitly settled while `TraceCtx` includes `parent_id` and baggage. | Choose and document a durable policy: either persist full `TraceCtx` where needed or define `trace_id` as the canonical durable trace reference. **Accept:** trace-bearing imports preserve the declared durable fields intentionally, and tests/docs stop implying more than is actually persisted. |
| SM-104 | P2 | semantic-memory | `src/lib.rs::compat`, `src/projection_import.rs`, `tests/import_boundary_tests.rs` | COMPAT_DEBT | partial | yes | medium-high | confirmed-static | Legacy import envelope APIs and JSON-compat helpers are still public and easy to mistake for canonical surfaces. | Demote harder: move active docs/examples/tests to canonical batch import only; gate or quarantine legacy modules if feasible. **Accept:** no active control doc presents `ImportEnvelope` as the normal path. |
| BRG-102 | P2 | forge-memory-bridge | `src/legacy.rs`, `src/lib.rs` | COMPAT_DEBT | partial | no | medium | confirmed-static | Legacy bridge upgrade helpers remain public and easy to fossilize. | Hide behind a compat feature/internal module if feasible, or at minimum make docs/tests clearly compat-only. **Accept:** canonical docs and examples no longer present legacy bridge transforms as normal flow. |
| DOC-101 | P3 | root control docs | `AGENTS.md`, `MASTER_ISSUE_MATRIX_CODEX_V5.md`, `LATEST7.md`, proof comments/tests | DOC_DRIFT | no | yes | high | control-doc-drift | The current root AGENTS and several V5/V7 docs still describe already-fixed migration fights and, in the case of `recorded_at`, actively encode the wrong contract. That will mis-steer Codex. | Install V6 docs as the active control plane, mark older docs historical, and scrub tests/comments that teach the wrong law. **Accept:** V6 docs are the primary entrypoint and no active doc contradicts the canonical spec on temporal/storage law. |
| DOC-102 | P3 | knowledge-runtime docs/config | `knowledge-runtime/src/config.rs`, `knowledge-runtime/src/lib.rs`, `knowledge-runtime/src/runtime.rs`, `knowledge-runtime/README.md` | DOC_DRIFT | partial | yes | medium | confirmed-static | Several docs still say projection persistence “will not be” implemented, even though the canonical spec only forbids runtime from persisting authoritative truth; future rebuildable caches/materializations are not the same thing. | Narrow the language to current-phase truth. **Accept:** docs say “not implemented in this crate today” unless the architecture has explicitly forbidden the feature forever. |

## Recommended sequencing

1. **BRG-101 + SMF-101 first** — settle temporal storage law and version-lineage law before adding new query surfaces.
2. **SM-101 + KR-101 next** — make imported projections truly queryable end-to-end.
3. **KR-102 + KR-103** — implement real temporal and full-scope semantics on the new projection routes.
4. **LIV-101 + KR-104** — enrich Forge export and causal consumption once the retrieval substrate exists.
5. **KR-105 + SM-102 + SM-103** — add entity candidate expansion and bounded recomputation hardening.
6. **SM-104 + BRG-102 + DOC-101 + DOC-102** — burn down compatibility and doc drift last.

## Secondary lane after core closure

The adjacent trace/retry lane (`agent-graph`, `job-queue`, `AI-Batch-Queue`, `LLM-Pipeline`, `Tauri-Queue`) still matters, but it should be tackled **after** the core projection path is actually queryable and temporally correct.

When the core lane is green, do one focused pass on:

- end-to-end `AttemptId` / `TrialId` / `TraceCtx` propagation,
- truthful retry ownership,
- checkpoint/replay lineage recovery,
- and parser/patch reliability hardening.

That is follow-on work, not the first blocker for V6.
