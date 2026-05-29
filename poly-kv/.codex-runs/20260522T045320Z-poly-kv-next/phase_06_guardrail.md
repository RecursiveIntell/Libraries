# Phase 06 Boundary Guardrail

Guardrail sources:

- `codex/manual-injections/PHASE_BOUNDARY_GUARDRAIL.md`
- `codex/manual_injections/AFTER_PHASE_06.md`

## Revalidation

1. Source-of-truth ownership: harness scripts consume crate/test behavior and write run artifacts; they do not own codec, shape, pool, or receipt semantics.
2. Duplicate abstraction or shadow implementation: no codec math or pool implementation added to scripts.
3. Silent schema widening, coercion, hidden fallback, fake compatibility: harness outputs explicit `status` and `skip_reason`; no fake Python compatibility.
4. Material operations and receipts: Rust synthetic harness records command result; Python boundary harness records native availability; parity report records pass/skip explicitly.
5. Exact fallback: unchanged in Rust pool build paths.
6. Optional adapters: unchanged; no compatibility claim added.
7. Tests/fixtures/assertions: JSON artifacts validated with `python -m json.tool`; parity script checks required inputs.
8. Failed/skipped validation: Python boundary status skip is recorded with exact reason.
9. Scope exclusions: no governor, runtime authority, semantic-memory, Gloss, Recall, AiDENs, or ClaimLedger integration introduced.
10. Rollback path: remove the three harness scripts and generated JSON artifacts.

Decision: proceed to Phase 07.
