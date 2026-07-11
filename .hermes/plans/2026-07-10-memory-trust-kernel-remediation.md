# Memory Trust Kernel Remediation and Research Plan

> Governing specification saved from Josh Stevenson's 2026-07-10 audit/remediation directive. Execute with strict TDD, receipts, append-plus-supersession, no shadow truth, and no false completion.

Goal: Make RecursiveIntell's existing local-first, provenance-led, typed, bitemporal architecture inescapable in ordinary executable paths, then promotion-gate retrieval, graph, routing, injection, cache, and compression research.

Architecture: five one-way planes: (1) SQLite authority journal, canonical MemoryEnvelope, claim-ledger references, bitemporal lineage, permits and receipts; (2) rebuildable FTS/vector/quant/cache/community projections; (3) witnessed retrieval execution; (4) shared governance/admission; (5) thin host adapters. One canonical mutation API owns append, supersede, and redact.

Hard rules:
- Every retrieval API has explicit StateView; default Current; fail closed on inconsistent lineage.
- Autonomous-agent retrieval is witnessed by default. Unwitnessed read is explicitly named and cannot authorize actions.
- Authority mutations are atomic, idempotent, append/supersede based, and receipt bearing.
- Host hooks contain no independent admission semantics and never fall back from admitted hits to raw hits.
- Derived artifacts are generation-bound, deletable, rebuildable, and never canonical truth.
- Empty, zero, no-path, skipped, degraded, failed, and budget-exhausted are distinct typed outcomes.
- No optimization becomes default without fixed holdout evidence, safety gates, rollback pointer, and receipt.

## Phase 0 — Reproducible baseline

Deliver commit/status manifest, cargo metadata, schema/migration digest, fixed hostile corpus, golden MCP tools/list, feature/build matrix, current tests and public/private divergence. Preserve pre-existing dirty work and record blockers.

## Phase 1 — Critical containment

1. Fix bitemporal as-of future leakage and stale-head leakage. All retrieval APIs default to Current and reject inconsistent lineage.
2. Disable or generation-bind search caching; cache keys include StateView, exactness, authority snapshot, retrieval epoch, model/config digests. Witness cache hits.
3. Persist all typed metadata promised by adapters; unsupported metadata fails closed.
4. Remove destructive tools from autonomous profiles. Separate read, append, operator, admin capabilities.
5. Disable non-atomic consolidation and in-place correction. Replace with propose_merge and atomic commit_merge producing one new active head.
6. Expose only witnessed search to autonomous agents.
7. Remove raw-hit/fail-open hook injection fallback. Disable automatic injection for action-capable sessions without complete provenance.

Exit: no stale cache truth, future-state leakage, discarded security metadata, destructive autonomous mutation, non-atomic lineage, unwitnessed autonomous search, or fail-open injection.

## Phase 2 — Canonical contracts

Implement/version/schema:
- MemoryEnvelopeV1
- StateView { Current, HistoricalAt, RecordedAsOf, IncludeSuperseded }
- CapabilityManifestV1
- RetrievalResponseV1 and RetrievalWitnessV1
- StageOutcomeV1: NotPlanned, Skipped, AnalysisOnly, Applied, Degraded, Failed, BudgetExceeded
- InjectionDecisionV1 and InjectionDisposition
- SupersessionReceiptV1
- bounded Probability, Confidence, CosineSimilarity, NonNegativeWeight types
- transactional retrieval_epoch and AuthoritySnapshotId

Retrieval witnesses bind request ID, evaluation time, authority snapshot, epoch, query/filter/model/config digests, candidate backend/generation, exactness, ordered IDs/digests, stage outcomes, degradation list, cached witness parent, and source spans.

Exit: MCP, Recall, AiDENs, importers, Hermes, Claude, and Codex round-trip identical contracts without widening.

## Phase 3 — Atomic authority transitions

Create MemoryAuthority append/supersede/redact operations with permits, caller idempotency keys, operation journal, atomic lineage/head/epoch/receipt persistence. Separate operation identity, content equivalence, and assertion identity. Replace consolidation with propose_merge and commit_merge; never rewrite assertions.

Fault gates: before/after append, lineage edge, active-head update, journal append, epoch increment, receipt persistence, lost response, duplicate retry. Every fault yields no commit or one complete discoverable transition.

## Phase 4 — Shared governance and host adapters

Move admission into one shared library/MCP operation. InjectionDecision binds retrieval receipt, principal, host, task class, policy digest, token/influence budget, risk, admitted/rejected items and disposition. Evaluate authorization, sensitivity, trust, StateView, source authority, hubness, duplicates, contradictions, injection indicators, semantic relevance, task/tool risk, and cumulative influence.

Rigid injected frame includes memory_id, namespace, source, trust, state, valid_at and receipt reference, labeled DATA ONLY / NOT AN INSTRUCTION. All hosts must produce identical decisions for normalized requests. Sensitive/action-capable failure is fail-closed. Calibrate thresholds per embedding model/corpus/task, not globally.

## Phase 5 — P1 correctness hardening

- Decoder/factor graph analysis changes response order only when StageOutcome=Applied with before/after order digests and changed positions.
- Graph path returns Found, NoPathWithinCompleteSearch, BudgetExceeded, InvalidEndpoint.
- Statistics return component health/error, never fabricated zero.
- Routing feedback stores immutable observations; policy writes marked mutating; persistence errors surfaced; proxy labels distinct; offline holdout promotion only.
- Bounded numeric types reject NaN/infinity/out-of-range at authoritative constructors/deserializers with property/fuzz tests.
- CI compiles semantic-memory and MCP no-default/search/all-feature matrices or removes false profile docs.
- Caller idempotency journal replaces content-only identity policy.

## Phase 6 — Result-oriented research portfolio

A. StateValidityBench: current/historical/transition/what-known-then queries. Gate: >=80% lower superseded leakage without historical-accuracy loss.
B. Pipeline fault localization across ingestion through testimony. Gate: >=95% stage-specific receipt; no injected failure becomes ordinary empty output.
C. Poisoning/governance corpus. Gate: >=90% lower attack success while preserving >=95% benign utility on same model/corpus.
D. Reasoning drift: no-memory, gold, retrieved, labeled, contradictory, admitted/redacted. Gate: aggregate quality improves with no unsafe-action regression.
E. Fine-grained relation/evidence completeness. Promote graph/factor stages only if they beat exact hybrid retrieval with evidence-complete witnesses.
F. Compression recoverability against exact f32 for usearch/turbo-quant/FibQuant/multistage plus corrupt/mixed generations. Require explicit degradation, full rebuild, and pointer rollback.

## Blocking CI suites

freshness: mutation visibility; StateView cache separation; exact/approx separation; generation invalidation.
authority: metadata round-trip; unsupported metadata fail-closed; zero autonomous destructive tools.
lineage: atomic fault injection; exactly one active head; Current excludes superseded; Historical reconstructs; retries idempotent.
receipts: every agent search witnessed; snapshot/epoch/digests/order bound; cache witness; degradation explicit.
graph: stored-edge traversal; direction semantics; no-path vs budget; errors not zeros.
injection: instruction isolation; fail-closed sensitive sessions; namespace rejection; hub limiting; complete identity/provenance; cross-host conformance.
routing: mutating annotation; persistence surfaced; proxy labels typed; holdout promotion.
recovery: delete/rebuild projections; corrupt fallback; reject mixed generations; pointer rollback.

## Immediate execution order

Workstream A: repair search_as_of and StateView defaults/fail-closed lineage.
Workstream B: add sm_search_witnessed and restrict autonomous profile exposure; receipts mandatory and cache-aware.
Workstream C: remove raw-hit fallback, repair namespace matching, add identity/trust/state/source/receipt framing, fail closed for action-capable sessions.
Then run full affected test matrices and hostile benchmark. Only after all three are green proceed to atomic authority transactions and canonical contracts.

## Claim boundary

Until all gates pass, safe wording is: RecursiveIntell has strong evidence-governed memory machinery, but ordinary paths do not yet uniformly enforce it. Do not claim universal provenance, correct bitemporality, best overall memory, or end-to-end trust-kernel completion.

## Do not prioritize

More graph algorithms, autonomous consolidation, routing classes/RL, compression codecs, host-specific heuristics, natural-language temporal inference, larger tool surfaces, or universal claims before the trust kernel closes.
