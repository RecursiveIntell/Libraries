# Phase 03 Report

## Scope

Added a non-breaking builder convenience for exact fallback creation and verified adapter stubs remain unsupported.

## Changed files

- `crates/poly-kv/src/pool.rs`
- `crates/poly-kv/tests/synthetic_roundtrip.rs`

## Implementation

- Added `PoolBuilder::build_from_exact_blocks(blocks)`, which derives `ExactFallback::from_blocks(blocks.clone())` and delegates to the existing `build_from_blocks` path.
- Kept `PoolBuilder::build_from_blocks` and `PoolBuilder::exact_fallback` intact for existing callers.
- Verified `turbo_quant` and `fibquant` adapters remain explicit unsupported stubs unless their external APIs are inspected in a later pass.

## Validation

Commands and results:

- `cargo fmt --all`: pass
- `cargo test -p poly-kv synthetic`: pass for matching synthetic tests
- `cargo check --workspace --all-targets`: pass
- `cargo test -p poly-kv builder_can_derive_exact_fallback_from_input_blocks`: pass

## Compatibility

No alpha public API was removed. Existing explicit fallback examples still compile; the new convenience method is additive.
