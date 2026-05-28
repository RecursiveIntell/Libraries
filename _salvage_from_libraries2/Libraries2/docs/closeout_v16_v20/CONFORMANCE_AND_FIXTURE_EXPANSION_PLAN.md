# Conformance and Fixture Expansion Plan

## Principles

1. Every new artifact family in this pass gets:
   - schema registration
   - at least one fixture
   - at least one test
2. Every evaluator change must prove:
   - happy path
   - degraded/advisory path
   - non-silent blocking path where relevant
3. `kernel-conformance` should stay the cross-crate reference harness for the hard semantic seams.

## Required new test coverage

### v16
- replay slice admitted
- replay slice downgraded
- divergence report emitted
- treaty suspension emitted
- local dissent preserved under downgrade/suspension

### v17
- fit run blocked when refuter suite missing
- fit run advisory when stability report is missing or adverse
- fit run eligible only when both gating surfaces are present

### v18
- value-aware ordering beats naive source order
- budget exhaustion pauses rather than silently dropping campaigns
- hypothesis linkage remains queryable in the decision path

### v19
- amendment blocked when rollback missing
- amendment approved when rollback and obligations satisfied
- archive/compaction preserves historical-query guarantee with explicit degradation text

### v20
- generation emits all companion bundles with backpointers
- unsatisfied proof obligations remain visible
- human veto changes generated-artifact state without pretending prior admission
- rollback path restores prior advisory constitutional surface

## Fixture discipline

New fixtures should live beside the existing versioned fixture folders.
Do not create an ad hoc fixture directory.

Each new fixture should state, as applicable:
- ids
- owned backpointers
- advisory/degraded/admission state
- generated timestamps
- why a path is blocked or downgraded

## Manifest discipline

Every new schema file added in this pass must also be listed in the matching:
- `contracts/schemas/v16/manifest.json`
- `contracts/schemas/v17/manifest.json`
- `contracts/schemas/v18/manifest.json`
- `contracts/schemas/v19/manifest.json`
- `contracts/schemas/v20/manifest.json`
