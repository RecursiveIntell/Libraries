# CLAUDE MECHANICAL CHECKS

Run checks equivalent to these after patching.
Interpret results; do not just paste raw grep output.

---

## 1. Forbidden / stale wording checks

```bash
grep -RIn --include='*.md' --include='*.rs' '\bImportEnvelope\b' .
grep -RIn --include='*.md' --include='*.rs' '\bTraceId\b' .
grep -RIn --include='*.md' --include='*.rs' 'ScopeKey \{ namespace:' .
grep -RIn --include='*.md' --include='*.rs' 'Each retry produces a new `AttemptId`\|retry produces a new AttemptId\|new AttemptId per retry' .
grep -RIn --include='*.md' --include='*.rs' 'ai-batch-queue.*none\|owns no retry' .
grep -RIn --include='*.md' --include='*.rs' 'traceparent.*pad\|traceparent.*truncate\|pad.*traceparent\|truncate.*traceparent' .
```

### How to interpret

- `ImportEnvelope` hits are only acceptable in explicitly legacy / compatibility-only contexts.
- `TraceId` hits are only acceptable where migration notes still explicitly allow legacy interop references.
- `ScopeKey { namespace:` hits are suspect unless they are in a very clearly justified primitive/helper implementation. New ad-hoc conversion call sites are forbidden.
- Any hit implying “new `AttemptId` per retry” is a failure.
- Any hit implying `ai-batch-queue` owns no retry logic is a failure.
- Any hit permitting pad/truncate traceparent behavior is a failure.

---

## 2. Compatibility-phase labeling checks

```bash
grep -RIn --include='*.md' --include='*.rs' 'Phase status: compatibility' .
grep -RIn --include='*.md' --include='*.rs' 'migration-only' .
```

### How to interpret

Compatibility helpers and surfaces should still be easy to find and obviously phase-labeled.
Missing labels on migration-only helpers are a failure.

---

## 3. Regression checks for already-adopted fixes

Confirm all of the following remain true:

- canonical names still intact: `ExportEnvelopeV1`, `ProjectionImportBatchV1`, `LegacyImportEnvelopeV1`
- bridge non-public boundary still explicit
- `ClaimVersionId` still first-class
- relation-version parity still documented
- alias rows still scoped
- durable review state still explicit
- canonical namespace helper law still enforced
- digest law still BLAKE3 + deterministic serialization constraints
- evidence dereference still audit-only by default
- compatibility surfaces still phase-labeled

---

## 4. Boundary sanity checks

Confirm all of the following after patching:

- `stack-ids` still reads as primitive-only
- `knowledge-runtime` still reads as non-authoritative
- bridge/storage separation is still intact
- compatibility path still exists for one migration cycle
- migration helpers still have explicit removal conditions
- no doc now sounds like migration is already complete when it is not

---

## 5. Release / blocker confirmation

Before finishing, explicitly confirm:

- retry ownership is singular and aligned across docs
- `AttemptId` means logical retry family everywhere
- `TrialId` means concrete execution inside that family everywhere
- non-W3C trace serialization is now defined without pad/truncate
- `danger-sm-write` is governed or explicitly non-shippable
- backfill/recovery language now points to explicit release-blocking or clearly tracked tests

