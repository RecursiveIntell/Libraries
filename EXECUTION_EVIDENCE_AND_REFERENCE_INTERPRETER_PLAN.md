# Execution Evidence and Reference Interpreter Plan

## Core thesis
The stack must be able to **testify**:
- what it did,
- under which execution conditions,
- with which retries, deadlines, routes, and widening decisions,
- and how that work linked back to episodes, claims, and repairs.

## Execution-evidence family
Freeze a canonical family covering at minimum:
- tool / dispatch receipt
- retry family + attempt lineage
- queue-hop lineage
- replay linkage
- deadline / budget lineage
- provider/tool route
- degradation/widening markers
- approval / rollback / adjudication backpointers

## Reference interpreters
Implement executable reference behavior for:
1. bitemporal `as_of(valid_t, recorded_t)`
2. bridge import atomicity and digest preservation
3. runtime widening / multi-view provenance semantics
4. repair-record invariants
5. exact-on-small-slice oracle comparison for kernel paths

## Deliverables
- reference interpreter modules
- golden artifact fixtures
- differential tests against production paths
- failure artifacts when production diverges from reference
