# Phase 01 — Build Certification and Raw Failure Repair

## Objective

Make build/test/clippy/verify truth visible before feature work.

## Required commands

```bash
rustc --version | tee target/p20-rustc-version.txt
cargo --version | tee target/p20-cargo-version.txt
cargo metadata --format-version=1 > target/p20-cargo-metadata.json
cargo tree --workspace > target/p20-cargo-tree.txt
cargo fmt --all --check 2>&1 | tee target/p20-fmt.log
cargo check --workspace --all-targets --all-features 2>&1 | tee target/p20-check.log
cargo test --workspace --all-targets --all-features 2>&1 | tee target/p20-test.log
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tee target/p20-clippy.log
bash scripts/verify.sh 2>&1 | tee target/p20-existing-verify.log
```

Fix errors. Do not add features. Do not bypass canonical crates.

## Acceptance gate

Commands pass or failures are explicitly fixed/quarantined with docs corrected.
