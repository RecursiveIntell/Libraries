# P31A Package Replay Receipt

## Replay Commands

```bash
# Verify workspace compiles
cd /home/sikmindz/Coding/Libraries/AiDENs && cargo check --workspace

# Verify formatting
cargo fmt --all -- --check

# Verify clippy
cargo clippy --all-targets

# Verify all tests
cargo test --workspace

# Verify Python assertion gates
python3 scripts/assert_release_ledger_schema.py
python3 scripts/assert_release_truth_consistency.py
python3 scripts/assert_support_claims_have_evidence.py
bash scripts/assert_adapter_delegation.sh
python3 scripts/assert_root_markdown_archive_policy.py
python3 scripts/assert_codex_artifact_classification.py
```

## Expected Results

- cargo check: 0 errors
- cargo fmt: clean
- cargo clippy: 0 warnings, 0 errors
- cargo test: 429 passed, 0 failed
- All assertion scripts: exit 0
