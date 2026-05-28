# Digest Migration Runbook

## Purpose

This runbook covers digest/version changes that affect replay identity, evidence linking, or import compatibility.

## Current explicit digest versions seen in the snapshot

From `Primitives/cea-core/src/attribution.rs`:

- `cea-core:run-hash:v2`
- `cea-core:cause:v2`
- `cea-core:effect:v2`

## Required migration steps

1. Introduce the new digest version as an explicit constant.
2. Document why the digest changed (semantic change vs bug fix vs normalization change).
3. Provide a replay-compatibility decision:
   - dual-read / dual-write,
   - read-old write-new,
   - or one-shot rehash migration.
4. Record how historical receipts, bundles, and projections continue to point back to the old digest if needed.
5. Add downgrade safety notes: can an older binary safely read the new digest namespace?
6. Add regression tests that round-trip old and new digest material where compatibility is promised.

## Forbidden shortcuts

- Do **not** silently replace one digest namespace with another.
- Do **not** rewrite historical evidence-bearing rows without append-plus-supersession semantics.
- Do **not** claim replay continuity without tests that prove it.
