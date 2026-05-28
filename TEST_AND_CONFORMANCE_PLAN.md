# Libraries Test and Conformance Plan

## Conformance classes

### 1. Artifact/schema conformance
- generated schemas exist for every wire-visible family
- meta-validation passes
- compatibility mode and migration owner are declared
- additive vs breaking changes are mechanically classified

### 2. Bridge conformance
- digest preservation
- backpointer preservation
- atomic import
- import failures produce artifacts, not logs

### 3. Temporal/reference conformance
- valid-time / recorded-time queries match reference interpreter
- widening / degradation semantics are explicit and reproducible
- as-of query fixtures reproduce past states exactly

### 4. Execution-evidence conformance
- retry family and attempt lineage preserved
- queue hops and replay lineage queryable
- approvals/rollback/adjudication emit receipts

### 5. Kernel/runtime conformance
- exact-on-small-slice oracle comparison
- oscillation / convergence failure artifact generation
- region boundary protocol tests
- local repair before global rebuild tests

## Acceptance gates
- no hard semantic seam policed only by prose
- no wire-visible artifact without schema and compatibility policy
- no bridge mutation without artifact proof
- no risk-bearing output without verification plan
- no control loop without stop rules, escalation rules, and auditable receipts
