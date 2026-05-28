# CLAUDE OUTPUT TEMPLATE

Use this exact structure in your final response for the session.

---

## 1. What changed

Give a concise summary of the conformance corrections made.

## 2. Files patched

List every file touched.

## 3. `TRACE_RETRY_CONTRACT.md` preserve-vs-replace report

### Preserved
- single retry owner principle
- link-not-parent queue/replay semantics
- bounded baggage
- no hidden retries

### Replaced
- owner matrix row(s)
- `AttemptId` / `TrialId` model
- bad examples / timelines / invariants
- traceparent pad/truncate behavior

## 4. Regression scan report

Explicitly confirm whether any of these regressed:
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

## 5. Consistency / deferments report

### Contradictions resolved
List each contradiction resolved.

### Intentionally deferred
List anything intentionally deferred and why.

### Blocked by missing code vs missing docs
List anything not fully closable in docs alone.

## 6. Release-blocker confirmation

Explain how the docs now represent:
- retry ownership
- logical `AttemptId` / concrete `TrialId`
- trace serialization rule
- `danger-sm-write` governance
- backfill/recovery proof obligations

## 7. Mandatory yes/no answers

1. Is retry ownership now singular and consistent across all docs, including `ai-batch-queue`?
2. Does `AttemptId` now mean logical retry family everywhere?
3. Does `TrialId` now mean each concrete execution inside that logical retry family everywhere?
4. Is non-W3C trace serialization now defined without pad/truncate behavior?
5. Is `danger-sm-write` now either fully governed or explicitly non-shippable?
6. Are backfill/recovery obligations explicitly tied to concrete release-gate language?
7. Did any already-adopted correction regress?
8. Did any compatibility-only symbol or helper lose its phase label?
9. Did any new text reintroduce bare `ImportEnvelope` in new normative contexts?
10. Did any new text make `knowledge-runtime` sound authoritative or collapse bridge/storage separation?
11. Did any new wording or helper description add business logic to `stack-ids` beyond primitive/helper scope?

If any answer is “no,” explain exactly why and what remains open.

