# Architecture Closure Execution Map — V4

Generated on 2026-03-08 from the latest full snapshot.

## Objective
Finish as much of the architecture as possible **while remaining safe**. This means preferring strict validation, explicit compatibility labeling, and stronger proof coverage over risky broad rewrites.

## Non-goals for this pass
- Do not reopen the authority map.
- Do not introduce a forbidden dependency direction (for example, forcing `semantic-memory` to depend directly on bridge-owned Rust structs if the logical wire contract can remain serialized and validated).
- Do not remove compatibility APIs blindly if downstream callers are likely still relying on them.
- Do not fake completion for runtime temporal/scope semantics or Forge adapter wiring.

## Phase 0 — safety rails first
1. Read `MASTER_ISSUE_MATRIX_ARCH_CLOSURE_V4.md`, `FILE_AUDIT_INVENTORY_ARCH_CLOSURE_V4.md`, and this execution map.
2. Preserve dependency direction and crate authority boundaries.
3. Make changes in small, reviewable commits grouped by issue ID.

## Phase 1 — importer and bridge hardening

- **SM-003**: Review each non-core default individually. Keep only defaults that are explicitly canonical for V1; hard-fail fields that should be mandatory or provenance-bearing.
- **SM-004**: Add CHECK constraints (or equivalent validated write path with migration) for review_state and merge_decision values, with migration-safe handling for existing rows.
- **BRG-001**: Rename or document fields so import-side version law is explicit. Keep compatibility, but make it obvious which field is checked by bridge compatibility and which is persisted for provenance.
- **BRG-002**: Safely extend the export/bridge contract to carry real superseded claim_version identifiers, or explicitly freeze the limitation and add TODO-gated tests proving None is intentional until schema upgrade lands.
- **BRG-003**: Clarify in docs/tests that these are bridge-time import defaults for V1, not exporter-owned truth, and add upgrade notes for future schema enrichment.
- **SM-006**: Add one architecture-closure test module proving canonical import semantics end-to-end from bridge payload through memory query surfaces and derivation tracking.

## Phase 2 — runtime closure

- **KR-001**: Implement a real temporal execution path over imported claim/relation temporal fields, or tighten non-support behavior so temporal mode cannot pretend to be real temporal search.
- **KR-002**: Strengthen scope enforcement semantics. Either add reliable runtime-side filtering for all returned results plus proof tests, or keep strict_scope as the only allowed mode for non-namespace dimensions in canonical flows.
- **KR-003**: Either wire a minimal rebuild execution interface or make the separation explicit through a dedicated external rebuild driver trait and proof tests.
- **KR-005**: Expand proof tests to include trace continuity, strict-scope behavior, temporal degradation behavior, and imported evidence visibility rules.

## Phase 3 — compat burn-down with low blast radius

- **AG-001**: Keep compat fields only at explicit compatibility boundaries. Make canonical trace_ctx/attempt_id/trial_id the dominant internal/event path and reduce legacy trace emission sites.
- **AG-002**: Preserve legacy attempt numbers only for consumers that still need them, but add proof tests showing AttemptId is stable across the retry family and TrialId changes per concrete retry.
- **JQ-001**: Keep compatibility, but route new examples/tests/docs through TraceCtx-first APIs only. Plan hard deprecation once downstream consumers migrate.
- **JQ-002**: Do not remove legacy columns unsafely. Add migration plan/tests showing canonical fields are authoritative when present and old rows are upgraded predictably.
- **LLM-001**: Do not break callers casually, but continue deprecation and migrate examples/tests/events to TraceCtx-first usage.
- **TQ-001**: Keep the bridge helpers, but move examples/default guidance to include_trace_ctx and canonical TraceCtx consumption only.
- **SID-001**: Do not remove in this pass unless callers are already migrated. Instead, tighten docs/tests so new work uses TraceCtx first and legacy helpers stay compatibility-only.

## Phase 4 — end-to-end proof and docs

- **FMF-001**: Add minimal end-to-end proof that Forge bundle/envelope metadata survives the export/bridge/import path and is auditable from memory/runtime without raw-receipt leakage.
- **ABQ-001**: Add one integration proof tying traced enqueue, executor trial stamping, and retry_failed semantics to the wider stack’s retry-owner law.
- **E2E-001**: Add one end-to-end proof suite that exercises the canonical path and asserts evidence opacity, lineage continuity, scope/temporal truthfulness, idempotent import, and replay-safe semantics.
- **DOC-001**: Update root and crate docs after code changes so compat/deferred features and canonical normal paths are described accurately.

## Exit condition
This pass is successful when the P0/P1 issues are closed or explicitly documented as deferred with proof-backed rationale, and when the end-to-end proof suite is strong enough that the remaining open items are concentrated mainly in clearly labeled deferred features rather than core architecture seams.