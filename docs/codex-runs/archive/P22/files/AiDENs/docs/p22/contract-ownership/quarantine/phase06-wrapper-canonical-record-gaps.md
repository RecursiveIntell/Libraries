# Phase 06 Quarantine - Wrapper Canonical Record Gaps

SOURCE BASIS: 2026-04-28

## Scope

Phase 06 inspected AiDENs-local tool, repair, runtime-view, kernel/region, and subtraction DTOs that overlap canonical stack concepts.

## Quarantined Gaps

1. `BoundaryRepairReportV1` and `JsonRepairReportV2` now carry `canonical_repair_record_ids: Vec<StackBoundaryRepairRecordId>` and `canonical_backpointers`, but the current `aidens-boundary-kit` display repair helpers do not persist or return concrete `verification-control::BoundaryRepairRecord` artifacts.
2. `SchemaValidationReportV1` now carries `canonical_control_receipt_ids: Vec<StackControlReceiptId>` and `canonical_backpointers`, but local schema validation display reports do not mint canonical `verification-control::ControlReceipt` records.
3. Region/subtraction DTOs now carry typed canonical IDs and canonical backpointers, but they remain report/display wrappers until owner-approved wiring supplies concrete `constraint-compiler`, `recursive-kernel-core`, `kernel-execution`, and `semantic-memory-forge` artifacts.

## Owner Decision Required

- Boundary repairs must either be produced through `verification-control::BoundaryRepairRecord` and record the returned `BoundaryRepairRecordId`, or remain display-only and blocked from being used as repair truth.
- Schema validation must either be attached to a canonical `verification-control::ControlReceipt`, or remain a local boundary/display report only.
- Subtraction/frontier/support reports need an owner decision for whether the canonical concrete artifact is `semantic-memory-forge::SupportSetV1`, `semantic-memory-forge::RetractionRecordV1`, a constitutional-memory compaction receipt, or another owner-approved family.

## Safe Behavior Until Resolved

- Do not treat empty canonical ID vectors as canonical truth.
- Do not invent AiDENs-local repair, validation, region, or subtraction semantics.
- Use the new backpointer fields only for display/report reconciliation with owner crates.
