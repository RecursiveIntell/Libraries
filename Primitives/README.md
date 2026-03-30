# Primitives

Internal support crates for the Forge execution and evaluation lane.

These crates provide focused building blocks used mainly by `forge-engine`:

- `check-runner`: execution backend and check normalization
- `effect-signature`: stable effect payloads and hashing
- `forge-policy`: path, cap, and database safety policy
- `sandbox-workspace`: workspace sandbox and patch filesystem helpers
- `typed-patch`: structured patch schema plus validation/apply helpers
- `mindstate-core`: serializable mindstate payload types
- `stabilizer-core`: bounded attempt-phase and delta policy primitives
- `cea-core`: causal edit attribution and prediction primitives
- `cea-store`: storage contract for CEA graphs
- `cea-sqlite`: SQLite backend for `cea-store`

## Status

These are package-scoped support crates, not part of the root Tier 0 workspace. They are still active public API surfaces inside the repo, so their crate roots should explain purpose and authority even when they are not yet promoted as standalone top-tier packages.
