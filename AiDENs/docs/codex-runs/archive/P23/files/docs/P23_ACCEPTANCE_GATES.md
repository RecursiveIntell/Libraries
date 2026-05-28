# P23 Acceptance Gates

## P0 gates

- Package replay is self-contained.
- `z.py` current-run logic is generic.
- Legacy `zip.py` is not a runnable alternate packager.
- Script-reference checking catches missing/excluded verifier dependencies.
- Package roles are explicit and tested.
- Stale Pxx/Pyy artifacts outside archive are classified or archived.
- Final audit package identity matches actual emitted package role.

## Capability gates

- A local fixture-backed agent/test-agent/coding-agent run path exists.
- The run emits receipt-bearing output.
- The run can be inspected by CLI or library API.
- Unsupported/deferred paths degrade explicitly.
- Tests prove the path.

## Cargo gates

- `cargo fmt --all --check`
- `cargo check --workspace --all-targets --all-features`
- `cargo test --workspace --all-targets --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` unless documented blocker.

## Final gate

`P23_REQUIRE_CARGO=1 bash scripts/p23_verify.sh` must pass or the run is not complete.
