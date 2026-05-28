# P20 Verification Commands

Run from the AiDENs repo root.

```bash
rustc --version
cargo --version
cargo metadata --format-version=1 > target/p20-cargo-metadata.json
cargo tree --workspace > target/p20-cargo-tree.txt
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
python3 scripts/p20_scan_aidens.py --root . --out target/p20-scan
python3 scripts/p20_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl
bash scripts/p20_generate_audit_bundle.sh
```

If a command fails, fix it. Do not delete the command from the release gate.
