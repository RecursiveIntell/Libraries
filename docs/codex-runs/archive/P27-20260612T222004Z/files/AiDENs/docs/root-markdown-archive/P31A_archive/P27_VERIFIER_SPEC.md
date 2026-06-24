# P27 Verifier Spec

## Required entrypoints

- `scripts/p27_verify.sh` is the current verifier.
- `scripts/verify_current.sh` delegates to `p27_verify.sh`.
- `scripts/verify.sh` delegates to `verify_current.sh` for compatibility.
- CI calls `scripts/verify_current.sh` or `scripts/p27_verify.sh`, not historical Pxx verifiers.

## Verifier modes

Environment variables:

- `P27_REQUIRE_CARGO=1`: fail if cargo is unavailable or cargo checks fail.
- `P27_SKIP_CARGO=1`: skip cargo checks explicitly and record skip.
- `P27_REQUIRE_PACKAGE_REPLAY=1`: require package self-replay proof.
- `P27_ALLOW_OPTIONAL_OLLAMA=1`: run local Ollama smoke if available, otherwise skip.

## Required static assertions

1. verifier entrypoints exist;
2. active-run docs agree on P27;
3. AGENTS.md is P27 current;
4. source basis does not say P22/P23/P24/P25/P26 current;
5. support profile does not claim cloud/autonomy/V10 support;
6. ownership scanner fail-closed behavior is present;
7. scaffold-only profiles are fenced;
8. root Markdown archive policy is current;
9. package sidecars/final docs link to current run.

## Cargo checks

When cargo exists, run at minimum:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
```

Clippy/doc may be optional during early phase gates, but final gate should run them if feasible.

## Failure behavior

The verifier must fail honestly. It must not downgrade a hard failure to a pass because an old gate passed. If an environment prerequisite is missing, emit a specific classification:

- `cargo_unavailable`
- `sibling_workspace_missing`
- `optional_provider_unavailable`
- `package_replay_not_attempted`
- `package_replay_failed`
- `script_target_missing`
