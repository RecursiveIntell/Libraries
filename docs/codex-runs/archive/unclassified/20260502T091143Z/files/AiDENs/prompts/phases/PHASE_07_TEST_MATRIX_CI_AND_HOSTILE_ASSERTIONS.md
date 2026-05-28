# Phase 07 — Test Matrix, CI, and Hostile Assertions

## Goal
Turn P23 behavior into enforceable gates.

## Required assertions

Add/finish scripts:

- `scripts/assert_zpy_total_contract.py`
- `scripts/assert_codex_artifact_classification.py`
- `scripts/assert_script_refs_strict.py`
- `scripts/assert_package_self_replay.py`
- `scripts/assert_no_legacy_zip.py`
- `scripts/assert_aidens_capability_contract.py`
- `scripts/p23_verify.sh`

CI must run at least the stdlib/non-cargo gates. Full cargo can remain env-gated if repo/toolchain constraints require.

## Required tests

- `cargo fmt --all --check`
- `cargo check --workspace --all-targets --all-features`
- `cargo test --workspace --all-targets --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` unless explicitly blocked and documented.

## Acceptance gate

P23 claims must fail mechanically if any of the known P22 defects return.
