# CLAUDE SUPPORTING CONFORMANCE BRIEF

This file is a compact companion to `PERFECT_CLAUDE_CODE_PROMPT.md`.
Use it as a preserve/do-not-regress reference while you patch the docs and code-adjacent comments.

---

## 1. Source precedence snapshot

1. `CANONICAL_STACK_SPEC_V4.md`
2. `SPEC_PATCH_IMPLEMENTED.md`
3. `MIGRATION_NOTES_V4.md`
4. `NEXT_MASTER_DELTA_SPEC_FOR_CLAUDE.md`
5. `TRACE_RETRY_CONTRACT.md`
6. `LATEST4.md` only as current-state snapshot

---

## 2. Settled architecture — do not reopen

- Forge raw truth != memory queryable truth
- bridge transformation != storage ownership
- `knowledge-runtime` != authoritative database
- `stack-ids` = primitive-only
- new broad shared crate = forbidden
- bare `ImportEnvelope` in new normative text = forbidden
- bridge-owned row/batch types leaking into public memory API = forbidden

---

## 3. Already-adopted fixes that must survive

- canonical names: `ExportEnvelopeV1`, `ProjectionImportBatchV1`, `LegacyImportEnvelopeV1`
- `ClaimVersionId` first-class
- relation-version parity
- alias rows carry explicit `ScopeKey`
- durable review state on aliases/merge decisions
- canonical namespace → `ScopeKey` helper functions only
- deterministic BLAKE3 digest law
- evidence refs dereference only in explicit audit mode by default
- compatibility surfaces phase-labeled
- non-public bridge boundary pattern preserved

---

## 4. Compatibility path that must remain intact for one migration cycle

Keep all of the following alive and clearly labeled until migration notes say removal is allowed:

- `ImportEnvelope` compatibility surface in `semantic-memory`
- `import_envelope()` compatibility path
- `LegacyImportEnvelopeV1`
- `upgrade_legacy_envelope()`
- `transform_legacy_envelope()`
- `TraceCtx::from_legacy_trace_id()`
- `TraceCtx::to_legacy_trace_id()`
- phase labels saying compatibility / migration-only
- explicit removal conditions

Do **not** clean these up early just because the new architecture is prettier.

---

## 5. Remaining blockers this pass must close

### Retry ownership
- `ai-batch-queue` cannot remain “owns no retry logic”
- owner matrix must match v4

### Attempt / Trial semantics
- `AttemptId` must become logical retry family everywhere
- `TrialId` must become concrete execution inside that family everywhere
- examples, timelines, invariants, and mapping tables must all agree

### Trace serialization
- remove pad/truncate traceparent behavior
- pick one explicit canonical replacement
- preserve bounded baggage rules and link-not-parent queue/replay semantics

### `danger-sm-write`
- fully governed now **or** explicitly non-shippable this phase
- no half-support language

### Backfill / recovery proof language
- connect migration guarantees to release-blocking or clearly tracked tests
- preserve dual-path coexistence and post-migration removal conditions

---

## 6. Things that are real blockers vs things that are noise

### Real blockers
- retry owner contradiction
- wrong `AttemptId` / `TrialId` model
- unsafe trace serialization rule
- incomplete `danger-sm-write` governance
- weak backfill/recovery proof linkage

### Low-priority cleanup only
- `PhaseStatus` placement
- wording polish that does not change meaning
- non-essential prose cleanup

If real blockers remain, do not spend time on cosmetic cleanup.

---

## 7. Anti-regression reminders

Before finishing, explicitly verify that you did **not** regress:

- envelope naming law
- bridge non-public boundary
- `ClaimVersionId`
- relation-version parity
- scoped alias rows
- durable review state
- canonical namespace mapping helpers
- digest law
- audit-only evidence dereference
- compatibility phase labels
- runtime non-authority
- bridge/storage separation
- primitive-only doctrine for `stack-ids`

