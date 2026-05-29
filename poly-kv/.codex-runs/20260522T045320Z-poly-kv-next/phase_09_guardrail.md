# Phase 09 Boundary Guardrail

Guardrail sources:

- `codex/manual-injections/PHASE_BOUNDARY_GUARDRAIL.md`
- `codex/manual_injections/AFTER_PHASE_09.md`

## Revalidation

1. Source-of-truth ownership: final handoff preserves crate ownership decisions in `invariant_report.md`.
2. Duplicate abstraction or shadow implementation: no duplicate TurboQuant/FibQuant math, controller, runtime authority, or app store introduced.
3. Silent schema widening, coercion, hidden fallback, fake compatibility: final reports record explicit fallback receipts, shape fail-closed behavior, and Python native skips.
4. Material operations and receipts: build/decode/reader receipts and harness JSON artifacts are listed.
5. Exact fallback: preserved and tested.
6. Optional adapters: unsupported stubs only; deferred until API inspection.
7. Tests/fixtures/assertions: validation results enumerate Rust, Python, and script gates.
8. Failed/skipped validation: maturin, native Python tests, and cargo-semver-checks skips are recorded.
9. Scope exclusions: no governor, runtime authority, daemon, semantic-memory, Gloss, Recall, AiDENs, or ClaimLedger integration.
10. Rollback path: `rollback_plan.md` exists.

Decision: final response may report completion with recorded skips/risks.
