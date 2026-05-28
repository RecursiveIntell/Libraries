# P29 Phase Plan

## Phase 00 — Source basis and run identity lock

Lock P29 current-run identity and prepare directories.

## Phase 01 — P28 failure absorption

Document and matrix all P28 evidence/package failures.

## Phase 02 — Package/archive classifier repair

Ensure current-run docs/scripts/handoffs cannot be archived as stale.

## Phase 03 — Verifier and evidence manifest repair

Create verifier, manifest path validator, extracted package replay plan.

Manual injection after Phase 03.

## Phase 04 — Claude audit import and triage

Import all 200 audit BUG IDs into P29 issue matrix and classify P0/P1/P2.

## Phase 05 — HNSW integrity and concurrency repair

Address BUG-001 through BUG-010 and related HNSW issues.

## Phase 06 — SQLite, migration, pool, and schema repair

Address migration atomicity, schema mismatch, SQLite/pool integrity bugs.

## Phase 07 — Search, ranking, dedup, and classifier repair

Address recency formula, dedup keys, FTS sanitizer, merge policy bugs.

Manual injection after Phase 07.

## Phase 08 — Vector, quantization, embedding, and HNSW sync repair

Address quantization/vector path issues and embedding/HNSW sync disclosures.

## Phase 09 — MemoryStore concurrency, drop, reembed, and resource bounds

Address reembed_all OOM, drop blocking, reader/writer lock ordering.

## Phase 10 — Graph, chunker, knowledge-runtime correctness

Address graph traversal, chunker splitting, contradiction/classifier bugs.

## Phase 11 — Stack IDs, AiDENs contracts, receipts, and baseline provenance

Address TraceCtx, execution context, tool receipt, artifact lifecycle, baseline bugs.

Manual injection after Phase 11.

## Phase 12 — v11A artifact/execution/operator contract finalization

Finish v11A material-operation contract spine.

## Phase 13 — v11A boundary/proof/degradation/semantic state finalization

Finish compiler profiles, proof debt, waiver law, semantic/view disclosure.

## Phase 14 — Supported-local agent path v11A conformance

Validate AgentSpec → runner → tools → receipts → final report path.

## Phase 15 — Large-file containment and module ownership cleanup

Prevent new mega-file regressions; update ownership docs.

Manual injection after Phase 15.

## Phase 16 — Adversarial conformance and bitemporal fixtures

Add v11A and bitemporal adversarial tests.

## Phase 17 — v11B right-graph declarations and misuse tests

Seed graph-surface declarations and misuse tests.

## Phase 18 — v11B region contract and boundary message seed

Seed region contracts, boundary messages, receipts, replay slices.

## Phase 19 — v11B convergence, residual, syndrome, and subtraction seed

Seed convergence governors, residual/syndrome envelopes, lawful subtraction.

Manual injection after Phase 19.

## Phase 20 — Docs, status, support, and known limitations convergence

Converge docs/support/status/source basis and limitations.

## Phase 21 — Final hostile audit, package, and self-replay

Run full command bar, strict package, extracted package self-replay, final handoff.

Manual injection before final package generation.
