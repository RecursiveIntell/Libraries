# Phase 03 — Core API compatibility and migration

Make API changes intentionally:

- Preserve existing public API where reasonable.
- Add V2 types without breaking alpha examples unless necessary.
- If breaking, document migration in `docs/NEXT_RELEASE_PLAN.md`.
- Add builder convenience to create fallback from exact blocks without requiring duplicate user input.
- Keep adapters as explicit unsupported stubs unless external APIs are inspected.

Gate: old synthetic examples compile or migration docs exist; no hidden compatibility shim.
