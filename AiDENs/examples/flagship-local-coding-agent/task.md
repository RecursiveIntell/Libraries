# Phase 06 Flagship Coding-Agent Task

Task:

- Load the fixture repo at `fixtures/p25/coding-agent-repo`.
- Read `README.md`.
- List workspace files and search by the app id token.
- Return a single bounded patch proposal for `src/lib.rs` to make `add_one` overflow-safe.
- Apply patch only if an explicit scoped permit is supplied.

Fixture-backed success criteria:

- The run emits `AiDENsRunBundleV2`.
- Patch proposal is emitted.
- Patch apply is blocked without permit and succeeds when permit is supplied.
- `inspect-run` validates the replay digest.
