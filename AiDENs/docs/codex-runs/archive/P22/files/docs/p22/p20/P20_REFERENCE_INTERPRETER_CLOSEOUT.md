# P20 Reference Interpreter Closeout

## Problem

A reference interpreter that returns `deferred=true` for a feature marked complete is a semantic lie wearing a lab coat.

## Required search

```bash
grep -R "deferred.*true\|reference-deferred\|TemporalQuery.*deferred\|tempor.*deferred" -n crates tests docs
```

## Required action

For each hit:

1. If the feature is `supported`, implement reference behavior and tests.
2. If the feature is not implemented, demote all docs to `partial` or `deferred`.
3. If the feature delegates to canonical crates, add an adapter conformance test that proves delegation.

## Minimum reference domains for P20

- provider route capability truth;
- permit enforcement;
- boundary repair semantics;
- receipt lineage;
- temporal query semantics if claimed;
- runtime widening disclosure if claimed;
- bridge atomicity/digest/backpointer preservation if claimed;
- repair-record invariants if claimed;
- agency policy decision semantics.

## Final proof

Phase 09 proof is recorded in `docs/p20/reports/PHASE_09_REPORT.md` and `crates/aidens-integration-tests/tests/phase_09_reference_hostile_tests.rs`.

Phase 10 may copy this evidence into the final audit bundle.
