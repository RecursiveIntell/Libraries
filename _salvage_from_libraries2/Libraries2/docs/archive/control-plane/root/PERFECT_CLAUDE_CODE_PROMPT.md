# PERFECT CLAUDE CODE PROMPT — FINAL CONFORMANCE PASS

You are working inside the user's Rust workspace.

Your job is **not** to redesign the stack.
Your job is to execute a **final conformance pass** against the already-settled architecture and the already-adopted patch set.

This session exists because the architecture is now mostly settled, but the reference set still has a handful of dangerous contradictions and incomplete closure points that can still produce the wrong implementation while the docs look cleaner.

You must perform a **surgical conformance order**, not a brainstorming pass.

---

## 0. What kind of session this is

This is a **subordinate correction session**.

It is **not** a greenfield design pass.
It is **not** permission to reinterpret crate boundaries.
It is **not** permission to weaken migration compatibility early.
It is **not** permission to reopen already-settled architectural doctrine.

Your goal is to:
- preserve the settled architecture,
- preserve all already-adopted corrections,
- preserve the compatibility path for its full intended lifetime,
- and close the remaining contradictions **mechanically and completely**.

If a change would make the docs prettier but weaken conformance, do **not** make that change.

---

## 1. Mandatory source precedence

Read and obey the sources in this exact order.

1. `CANONICAL_STACK_SPEC_V4.md`
2. `SPEC_PATCH_IMPLEMENTED.md`
3. `MIGRATION_NOTES_V4.md`
4. `NEXT_MASTER_DELTA_SPEC_FOR_CLAUDE.md`
5. `TRACE_RETRY_CONTRACT.md`
6. `LATEST4.md` only as current-state snapshot / drift reference

### Interpretation rule

Use them this way:

- `CANONICAL_STACK_SPEC_V4.md` = target contract, ownership boundaries, package graph, authority map, non-goals, release-gate law.
- `SPEC_PATCH_IMPLEMENTED.md` = already-adopted corrections that must **not regress**.
- `MIGRATION_NOTES_V4.md` = rollout order, compatibility-path law, namespace/scope mapping, backfill guarantees, one-cycle removal conditions.
- `NEXT_MASTER_DELTA_SPEC_FOR_CLAUDE.md` = session-scoped conformance order; subordinate to the three docs above.
- `TRACE_RETRY_CONTRACT.md` = intended retry/trace reference, but it still contains known contradictions that must be corrected.
- `LATEST4.md` = current-state snapshot only. It does **not** override architecture.

If these files disagree, higher precedence wins.

---

## 2. Architecture that is fixed and must not be reopened

Do **not** redesign any of the following.

### 2.1 Core doctrine

Keep these rules intact:

- raw verification truth is not the same thing as queryable memory truth
- shared IDs do not imply shared authority
- projection transformation is not storage ownership
- `knowledge-runtime` must not become a second database
- queryable imported projections are the default knowledge surface, not implicit live joins into Forge raw truth

### 2.2 Crate authority map

These roles are fixed:

- `semantic-memory-forge` = raw verification truth
- `forge-memory-bridge` = transformation from Forge export into memory import batches
- `semantic-memory` = durable queryable knowledge state + storage-oriented import boundary
- `knowledge-runtime` = planning / routing / merge / degradation / explainability only
- `stack-ids` = primitive-only crate for shared IDs, `TraceCtx`, `ScopeKey` helpers, digest helpers, zero business logic

### 2.3 Canonical wire vocabulary

These names are fixed:

- `ExportEnvelopeV1`
- `ProjectionImportBatchV1`
- `LegacyImportEnvelopeV1`
- legacy `ImportEnvelope` only in explicitly marked compatibility-only surfaces

Bare `ImportEnvelope` remains forbidden in new code and new normative docs outside legacy-marked contexts.

### 2.4 Bridge/API boundary

Do **not** regress the adopted non-public boundary pattern:

- bridge batch/row structs remain bridge-owned
- `semantic-memory` consumes them through a **non-public integration surface**
- bridge row types do **not** become part of `semantic-memory`’s normal public API

### 2.5 Already-adopted fixes that must survive

Do **not** regress any of these:

- `ClaimVersionId` is first-class
- relation-version parity with claims
- explicit `ScopeKey` on alias/merge rows
- durable/queryable alias review state
- canonical namespace → `ScopeKey` helper law
- canonical BLAKE3 digest law
- audit-only evidence dereference by default
- phase labeling on compatibility surfaces
- bridge/public API leakage resolved via non-public boundary

### 2.6 Primitive-reuse law

Do **not** introduce:

- crate-local copies of shared ID/trace types
- new broad shared crates
- temporary wrappers that become de facto contracts
- domain/business logic inside `stack-ids`

---

## 3. What is still broken and must be fixed now

This is the real work.
Everything else is preservation, verification, and anti-regression.

### 3.1 Retry owner contradiction: `ai-batch-queue`

Current contradiction:
- v4 says `ai-batch-queue` owns **leaf-level batch-item retry only**
- `TRACE_RETRY_CONTRACT.md` still says `ai-batch-queue` owns **no retry logic**

You must normalize all docs to the v4 owner matrix.

Required retry-owner model:
- `job-queue` = durable job retry
- `agent-graph` = graph/node retry
- `llm-pipeline` = transport retry + parse/validator-correction retry for one call boundary
- `ai-batch-queue` = batch-item retry inside a model-heavy leaf execution boundary only
- envelope import transaction = **no inner retry owner; outer queue only**
- verification backend execution / check-runner = keep aligned with v4; do not silently delete this owner row

You must patch:
- owner matrix
- examples
- tables
- per-crate mappings
- any prose still implying `ai-batch-queue` owns none

### 3.2 Attempt / Trial semantics are wrong

The current retry contract still teaches the wrong model.

Wrong model:
- `AttemptId` = each concrete retry execution
- each retry produces a new `AttemptId`
- `TrialId` = subordinate experiment inside that retry attempt

Canonical model you must enforce everywhere:
- `AttemptId` = one logical retry family for a boundary with one retry owner
- `TrialId` = each concrete execution inside that retry family

Consequences you must encode everywhere:
- retries inside one owner boundary create new `TrialId`s, not new `AttemptId`s
- exactly one `AttemptId` exists for one logical retry family
- one logical attempt may not have multiple retry owners
- replay follows the v4 rule: same `AttemptId` only when policy says it is the same logical attempt; otherwise new `AttemptId` with explicit linkage
- queue retry storms must not collapse multiple concrete executions into one fake trial

You must patch all of the following, not just one section:
- definitions
- examples
- timelines
- cardinality rules
- mapping tables
- invariant summaries
- replay wording
- any event/log/checkpoint language that still assumes “new `AttemptId` per retry”

### 3.3 Non-W3C trace serialization is unsafe

The current retry doc still permits pad/truncate behavior when serializing non-compliant trace IDs into W3C `traceparent` form.
That must be removed.

You must choose and document **one explicit canonical replacement**.

#### Preferred replacement
- deterministic hash-based conversion from legacy/non-W3C IDs into W3C-compliant wire IDs
- preserve original legacy identifier in a bounded structured metadata field or tightly bounded baggage key
- name the helper location explicitly
- name the preserved-original field name explicitly
- state round-trip expectations explicitly

#### Acceptable alternative
- reject non-W3C serialization at the boundary
- require conversion before serialization
- keep migration-only compatibility helpers explicitly phase-labeled

#### Forbidden
- padding
- truncation
- silent lossy conversion
- “deterministic conversion” without algorithm / location / preserved-original rule

You must also preserve:
- baggage restrictions
- no large/sensitive payload rule
- queue-hop span-link semantics
- replay-link semantics
- migration-only status of legacy interop helpers if they still exist

### 3.4 `danger-sm-write` governance is incomplete

You must force a binary outcome.

#### Option A — fully governed now
Document and, where needed, wire up:
- full audit contract
- release-build default disablement
- explicit non-canonical marker on resulting artifacts
- explicit rule that artifacts written through this path do not inherit normal comparability/import-lineage assumptions
- explicit rule that this is never the default ingest path
- explicit CI / release-note callout requirement if enabled in release builds

#### Option B — explicitly non-shippable this phase
- mark as unavailable for release use
- keep allowed only for tests/dev/migration/forensics under explicit opt-in
- remove ambiguous half-support wording

Mandatory audit fields if supported:
- operator/tool identity
- timestamp
- reason code / justification
- trace context
- explicit bypass marker
- explicit non-canonical marker

### 3.5 Backfill / recovery law needs sharper proof linkage

Migration notes already state the right guarantees.
That is not enough by itself.

You must explicitly tie backfill/recovery claims to release-gate or clearly tracked test obligations.

At minimum, docs after this pass must preserve or restate coverage for:

#### Import and consistency
- out-of-order envelope arrival
- duplicate-but-not-identical envelopes
- rollback on mid-import failure
- late-arriving older valid-time envelope
- import-lag warning propagation

#### Storage and recovery
- WAL crash during projection import
- HNSW rebuild after projection-storage migration
- restart during bridge dual-path migration window

#### Identity and entity
- alias unmerge after downstream projections exist
- human-confirmed merge reversal via explicit migration/repair flow
- competing canonical IDs under new evidence

#### Retry and trace
- nested retry misconfiguration rejected
- same logical attempt with multiple retry owners rejected
- replay trace linked but not parented
- queue retry storm isolation

#### Ranking
- deterministic ordering under identical semantic base with overlay changes
- unsupported causal leg cannot silently outrank supported evidence without warning

You must also preserve broader migration law:
- `import_log` preserved
- coexistence with `projection_import_log`
- legacy `Fact | Episode` rows remain queryable
- one-cycle compatibility path lifetime
- read-only retention of `import_log` after migration
- explicit removal conditions for compatibility-only surfaces

### 3.6 Python sidecar ambiguity must be closed cleanly

Do not leave the sidecar in vague half-deferred language.
Choose one explicit outcome:

#### Deferred / out of scope now
- clearly mark as phase-gated / out of scope for this pass
- state that no implementation work is required now
- preserve future hard requirements: request/response versioning, estimator metadata, timeout/backpressure ownership, replay determinism

#### Active soon / concretely tracked
- name the owning crate/client boundary
- name schema versioning rule
- name timeout/backpressure owner
- add replay/schema compatibility tests to release-blocking or explicitly tracked test surface

One fluffy paragraph is not enough.

### 3.7 `PhaseStatus` is non-blocking cleanup only

Treat `PhaseStatus` placement as cosmetic cleanup.
Do not spend meaningful effort there while real blockers remain.
You may move it if trivial and clearly useful, or explicitly leave it where it is for now.

---

## 4. Preserve vs replace map for `TRACE_RETRY_CONTRACT.md`

This section exists to prevent both under-correction and over-correction.

### 4.1 Preserve unless a canonical contradiction forces wording change

Preserve these ideas:
- single retry owner as governing principle
- queue-hop semantics use span links, not fake parent-child
- replay hops also use links, not fake parenting
- baggage is bounded
- no large blobs or sensitive data in baggage
- retries must be observable; hidden retries are forbidden

### 4.2 Replace decisively wherever they appear

Replace all of these:
- any owner matrix row claiming `ai-batch-queue` owns no retry logic
- any definition of `AttemptId` as each concrete retry execution
- any definition of `TrialId` subordinate to a per-retry `AttemptId` model
- any example timeline showing new `AttemptId` per retry
- any invariant summary encoding the wrong attempt/trial model
- any `traceparent` rule that pads or truncates trace IDs

### 4.3 No mixed models allowed

After patching, there must not be one section teaching the new model while an example, table, or checklist still teaches the old one.
Treat the file as a whole-file semantic alignment pass.

---

## 5. Scope of work

### 5.1 Allowed work

You may and should do all of the following where directly required:
- patch markdown docs
- patch module docs and type docs
- patch helper semantics docs where compatibility helpers are described
- patch checklists and release-gate language
- patch test-plan text
- patch code-adjacent comments if they still encode the wrong contract
- add small helper/validation logic only if directly required to remove ambiguity already claimed by docs

### 5.2 Forbidden work

Do not:
- redesign the stack
- reopen crate boundaries
- move transformation back into `semantic-memory`
- make `knowledge-runtime` authoritative storage
- introduce crate-local copies of shared ID/trace types
- invent a third broad shared crate
- weaken evidence dereference rules
- weaken compatibility guarantees early
- remove migration-only helpers before migration notes allow it
- “clean up” legacy compatibility surfaces as if migration is already complete

---

## 6. Mandatory patch order

### Step 1 — lock inherited constraints before editing anything
Internally restate:
- v4 target contract
- patch fixes already adopted
- migration compatibility obligations still in force
- exact retry sections to preserve vs replace

### Step 2 — patch `TRACE_RETRY_CONTRACT.md`
This is the highest-risk file.
Correct:
- retry owner matrix
- `AttemptId` / `TrialId` semantics
- replay wording if inconsistent
- trace serialization behavior
- examples, tables, invariants, mapping sections

Preserve:
- link-not-parent queue/replay semantics
- baggage bounds
- no-hidden-retry principle

### Step 3 — patch `SPEC_PATCH_IMPLEMENTED.md`
- update unresolved/deferred sections to reflect the corrected retry/trace model
- close `danger-sm-write` into governed or non-shippable
- explicitly preserve already-adopted fixes
- mark cosmetic cleanup as non-blocking if retained

### Step 4 — patch `MIGRATION_NOTES_V4.md`
- preserve compatibility timeline and removal conditions
- preserve namespace/scope mapping law
- connect backfill/recovery guarantees to explicit test / release-gate obligations
- preserve migration-only helper lifetime and phase labeling

### Step 5 — patch code-adjacent docs / helper docs where behavior is claimed
If helper behavior, compatibility behavior, or release-gating behavior is documented in module docs or comments, align them too.
Do not leave markdown and helper docs teaching different rules.

### Step 6 — optional low-risk cleanup only after blockers are closed
Only then do cosmetic cleanup such as clarifying `PhaseStatus` placement.

### Step 7 — touch `CANONICAL_STACK_SPEC_V4.md` only if unavoidable
Prefer fixing derived docs before editing the canonical source.
Only touch v4 if a direct wording inconsistency truly requires it.

---

## 7. Mechanical conformance checks you must run

### 7.1 Grep-style closure checks
Run checks equivalent to:

```bash
grep -RIn --include='*.md' --include='*.rs' '\bImportEnvelope\b' .
grep -RIn --include='*.md' --include='*.rs' 'crate-local TraceId\|\bTraceId\b' .
grep -RIn --include='*.md' --include='*.rs' 'ScopeKey \{ namespace:' .
grep -RIn --include='*.md' --include='*.rs' 'new AttemptId per retry\|Each retry produces a new AttemptId\|retry produces a new `AttemptId`' .
grep -RIn --include='*.md' --include='*.rs' 'ai-batch-queue.*none\|owns no retry' .
grep -RIn --include='*.md' --include='*.rs' 'pad\|truncate.*traceparent\|traceparent.*pad\|traceparent.*truncate' .
grep -RIn --include='*.rs' 'Phase status: compatibility' .
```

You must interpret results, not just dump them.
False positives in explicitly marked legacy contexts are acceptable only if clearly phase-labeled and compatible with migration notes.

### 7.2 Regression scan against already-adopted fixes
Explicitly verify this pass did **not** regress:
- envelope/batch naming law
- bridge non-public boundary law
- `ClaimVersionId`
- relation-version parity
- scoped alias rows
- durable review state
- canonical namespace mapping helpers
- digest law
- audit-only evidence dereference rule
- phase labeling on compatibility surfaces

### 7.3 Boundary sanity checks
Explicitly verify after editing:
- `stack-ids` still reads as primitive-only
- `knowledge-runtime` is still non-authoritative
- bridge/storage separation remains intact
- compatibility path still exists for one migration cycle
- migration helpers still have explicit removal conditions

---

## 8. Deliverables you must produce

Your output must include all of the following:

1. concise summary of what changed
2. exact files patched
3. preserve-vs-replace report for `TRACE_RETRY_CONTRACT.md`
4. regression scan report against already-adopted fixes
5. consistency report listing:
   - contradictions resolved
   - anything intentionally deferred
   - anything blocked by missing code rather than missing documentation
6. release-blocker confirmation showing how backfill/recovery/retry issues are now represented

---

## 9. Mandatory final yes/no questions

Answer these explicitly at the end.

### Remaining-blocker closure
1. Is retry ownership now singular and consistent across all docs, including `ai-batch-queue`?
2. Does `AttemptId` now mean logical retry family everywhere?
3. Does `TrialId` now mean each concrete execution inside that logical retry family everywhere?
4. Is non-W3C trace serialization now defined without pad/truncate behavior?
5. Is `danger-sm-write` now either fully governed or explicitly non-shippable?
6. Are backfill/recovery obligations explicitly tied to concrete release-gate language?

### Anti-regression closure
7. Did any already-adopted correction regress?
8. Did any compatibility-only symbol or helper lose its phase label?
9. Did any new text reintroduce bare `ImportEnvelope` in new normative contexts?
10. Did any new text make `knowledge-runtime` sound authoritative or collapse bridge/storage separation?
11. Did any new wording or helper description add business logic to `stack-ids` beyond primitive/helper scope?

If any answer is “no,” state exactly why and what remains open.

---

## 10. Success condition

This session is complete only if **all** of the following are true:

- no remaining doc claims `ai-batch-queue` owns no retry logic while v4 assigns leaf batch retry ownership
- no remaining doc teaches “new `AttemptId` per retry”
- no remaining doc teaches or permits trace-ID pad/truncate during `traceparent` serialization
- `danger-sm-write` is either fully governed or explicitly non-shippable
- backfill/recovery guarantees are tied to explicit release-blocking or clearly tracked test obligations
- already-adopted fixes remain intact and are explicitly regression-checked
- compatibility-path lifetime and removal conditions remain intact
- migration-only helpers remain phase-labeled
- no cosmetic cleanup displaced real blocker work

---

## 11. One-paragraph operating instruction

Use the stack docs as a conformance set, not a brainstorming prompt. Preserve the settled architecture, preserve the already-implemented patch resolutions, preserve migration-only compatibility surfaces for their full intended lifetime, and make a narrow but mechanically complete correction pass that fixes retry ownership, `AttemptId`/`TrialId` semantics, trace wire conversion, `danger-sm-write` governance, and backfill/recovery proof language. Patch not just markdown headlines but also examples, tables, invariants, module/type docs, helper semantics docs, and release/checklist text wherever they still encode the wrong model. Do not redesign the stack. Do not weaken the canonical boundary rules. Do not leave mixed models behind.
