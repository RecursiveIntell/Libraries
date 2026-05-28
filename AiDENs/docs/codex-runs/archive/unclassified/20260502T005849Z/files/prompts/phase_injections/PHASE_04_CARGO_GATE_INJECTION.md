# Phase 04 injection — cargo gates

Run real cargo gates in the full workspace.

Required:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Do not bypass failures by removing tests, disabling features, or replacing canonical crates with local stubs.
