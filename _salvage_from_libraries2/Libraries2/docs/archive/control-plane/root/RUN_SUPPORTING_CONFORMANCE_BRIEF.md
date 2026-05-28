# RUN_SUPPORTING_CONFORMANCE_BRIEF.md

This brief exists to prevent three common failure modes in the next Claude pass:

1. **paper compliance** — polishing markdown while the code model stays old
2. **migration breakage** — removing compatibility surfaces early
3. **authority drift** — treating the subordinate delta as a replacement for the canon

---

## A. Read the workspace as three migration states

### 1. Core migrated / partly visible
Expected focus:
- complete what is already underway
- verify missing visibility vs actual missing code
- do not redesign

### 2. Compatibility-contained / legacy-facing
Expected focus:
- keep compat surfaces alive only where allowed
- label them visibly
- stop them from being presented as normal-path guidance

### 3. Supporting pre-migration / propagation lag
Expected focus:
- replace local/stringly retry-trace identity with shared semantics where due now
- leave only clearly marked compat shims where unavoidable this cycle

---

## B. Preserve vs replace in `TRACE_RETRY_CONTRACT.md`

### Preserve
- canonical retry ownership principle
- `ai-batch-queue` leaf-level retry ownership
- `AttemptId` = logical retry family
- `TrialId` = concrete execution inside that family
- queue-hop = link semantics, not fake parent/child
- deterministic legacy trace conversion behavior
- bounded baggage law

### Replace if still present anywhere
- any stale “new AttemptId per retry” language
- any stale “ai-batch-queue owns no retry logic” language
- any pad/truncate trace serialization behavior
- stale examples or tables teaching the old model

---

## C. Compat-only is allowed; normal-path leakage is not

Allowed this cycle if phase-labeled:
- old `ImportEnvelope`
- `import_envelope()`
- legacy trace helpers
- namespace-only migration assumptions
- old Forge -> memory seam that survives only for migration

Not allowed:
- presenting any of those as the preferred normal path
- using them in newly added code by default
- deleting them early without satisfying migration law

---

## D. When `LATEST5.md` is insufficient

If `LATEST5.md` does not visibly prove a private/internal surface:
1. inspect source
2. decide whether the code actually has it
3. if present, improve reporting/snapshot generation
4. if absent, implement/fix it

Do not rewrite architecture just because a summary omitted something.

---

## E. This pass should leave fewer lies in the workspace

Success is not:
- more polished prose,
- higher test count,
- or a prettier summary.

Success is:
- less divergence between canon, code, and current-state reporting.
