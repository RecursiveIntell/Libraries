# Acceptance Checklist

Use this checklist to decide whether the phase is actually complete.

## Boundary enforcement

- [ ] `semantic-memory` remains storage/query authority, not Forge-policy authority.
- [ ] Forge remains raw verification authority.
- [ ] `knowledge-runtime` remains planning/merge/projection-interpretation only.
- [ ] Projection transformation/import mediation has an explicit boundary.
- [ ] There is no obvious convenience API that bypasses the intended import seam.

## Identity and lineage

- [ ] Core IDs/provenance carriers have explicit canonical ownership.
- [ ] Duplicate/conflicting core type definitions have been reconciled or intentionally wrapped.
- [ ] Projection version/lineage metadata is explicit.
- [ ] Invalid/ambiguous lineage states are rejected or strongly guarded.

## Import behavior

- [ ] Import is atomic per envelope/import unit.
- [ ] Repeated ingest is idempotent.
- [ ] Partial failure does not expose partial visibility.
- [ ] Dedupe semantics are explicit and not brittle.
- [ ] Import failure or lag surfaces in projection state/warnings.

## Runtime semantics

- [ ] Degraded behavior emits warnings instead of silently downgrading.
- [ ] Scope enforcement limitations are surfaced clearly.
- [ ] Duplicate fusion is deterministic.
- [ ] Ranking/tie-breaking is deterministic enough to debug.
- [ ] Merge preserves provenance/source-leg information.

## Trace and explainability

- [ ] Trace/provenance survives import, query, and merge paths.
- [ ] Results can be traced to source path/version/import context.
- [ ] Projection freshness/state is inspectable.
- [ ] The system does not imply stronger freshness or consistency than it has.

## Testing

- [ ] Boundary/invariant tests exist.
- [ ] Invalid import/input tests exist.
- [ ] Partial rollback tests exist.
- [ ] Idempotent repeated-ingest tests exist.
- [ ] Degraded-warning tests exist.
- [ ] Deterministic merge/order tests exist.
- [ ] Trace/provenance preservation tests exist.

## Documentation alignment

- [ ] Code-level docs/comments reflect the actual enforced design.
- [ ] No major behavior is “documented only” without corresponding enforcement.

## Final truthfulness check

- [ ] The implementation does not merely look aligned; it is actually aligned.
- [ ] No major unfinished behavior is hidden behind silent fallback or vague semantics.

