# Production non-goals and guardrails — 2026-03-18

## Do not do these things

- Do **not** invent a v27 or another constitutional plane while closing v25.
- Do **not** let consumer crates inspect raw profile fields like `allowed_run_modes`, `max_delegation_depth`, or `required_assurance_sections`.
- Do **not** add a dependency from `verification-policy` onto `profile-runtime`; use `stack-ids` citations only there.
- Do **not** keep `effect-runtime` on raw `String` IDs for effect-owned artifacts once typed IDs already exist in `stack-ids`.
- Do **not** ship new source fields without backfilling schema JSON, example JSON, and tests.
- Do **not** wire CI to claim closure unless the end-state production gate is actually passing.
- Do **not** silently widen authority, disclosure, residency, or emergency semantics while adding citation fields.

## Positive rules

- `profile-runtime` remains the sole owner of composite constitutional computation.
- Consumers may consume compiled outputs and cite them; they may not privately recompute them.
- Where cycle risk exists, prefer `stack-ids` references over importing upstream runtime types.
- Every externally visible artifact touched by this pass must have a schema, an example, and at least one test path.
- The `libraries-source/` mirror must be resynced after the repo changes land.
