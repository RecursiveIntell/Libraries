# Rollback Plan

To roll back this implementation pass:

1. Remove the added workspace files: `Cargo.toml`, `Cargo.lock`, and `crates/`.
2. Restore `README.md` and `docs/README_DRAFT.md` to their prior tracked contents.
3. Remove `.codex-runs/20260520T174516Z-alpha1/`.

No generated runtime state, external service changes, publish actions, or app integrations were created.
