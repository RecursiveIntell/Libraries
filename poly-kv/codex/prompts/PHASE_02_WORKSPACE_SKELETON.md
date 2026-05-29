# Phase 02 — Workspace and Crate Skeleton

## Objective

Create or update Rust workspace structure safely.

## Required actions

1. Add workspace `Cargo.toml` if missing.
2. Add `crates/quant-codec-core` and `crates/poly-kv`.
3. Add crate manifests with minimal dependencies.
4. Add lib/module skeletons.
5. Run `cargo metadata --no-deps` and `cargo check --workspace`.

## Acceptance gate

Workspace resolves and skeleton compiles.

## Phase-boundary report must include

- files inspected;
- files changed;
- commands run;
- tests/checks passed/failed/skipped;
- source-of-truth boundary status;
- unresolved blockers;
- rollback notes.
