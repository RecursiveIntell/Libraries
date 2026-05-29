# Phase 07 Boundary Guardrail

Guardrail sources:

- `codex/manual-injections/PHASE_BOUNDARY_GUARDRAIL.md`
- `codex/manual_injections/AFTER_PHASE_07.md`

## Revalidation

1. Source-of-truth ownership: docs preserve `quant-codec-core` ownership of shapes/codecs and `poly-kv` ownership of pool/receipts/fallback.
2. Duplicate abstraction or shadow implementation: docs add no implementation.
3. Silent schema widening, coercion, hidden fallback, fake compatibility: docs state unsupported adapters and native sidecar availability honestly.
4. Material operations and receipts: docs describe receipt fields and harness artifacts without claiming unrecorded behavior.
5. Exact fallback: docs continue to describe exact fallback as required and explicit.
6. Optional adapters: docs state TurboQuant/FibQuant remain unsupported stubs until API inspection.
7. Tests/fixtures/assertions: docs point to validation commands and run artifacts.
8. Failed/skipped validation: docs record maturin/native sidecar validation as pending/unavailable.
9. Scope exclusions: no governor, runtime authority, semantic-memory, Gloss, Recall, AiDENs, or ClaimLedger integration claimed.
10. Rollback path: revert the Phase 07 docs listed in `phase_07_report.md`.

Decision: proceed to Phase 08.
