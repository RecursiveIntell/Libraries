# context-governor guidance

## Verification

- Run `cargo fmt --check` after Rust edits.
- Run `cargo clippy --all-targets -- -D warnings` for behavior changes.
- Run `cargo test --all-targets` before reporting completion.
- Run `python scripts/codex_roi_eval.py` when changes affect compaction quality, receipts, or Codex integration.

## Invariants

- The latest user message must remain active after any compacted summary.
- Receipt hashes and compacted token counts must describe the final emitted `compacted_messages`.
- Exact fallback must remain available for summarized, omitted, quarantined, receipt-only, and archived items.
- Hard budget modes may warn or fail, but they must not silently imply unrecoverable context is recoverable.

## Scope

- Keep the core crate deterministic and host-agnostic.
- Put host-specific behavior in plugins, hooks, scripts, or adapters.
- Treat local benchmark numbers as local evidence only, not universal quality claims.
