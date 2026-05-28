# P29 Re-run and Audit Instructions

## Source-tree verification

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
bash scripts/p29_verify.sh
```

## Package verification

```bash
python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P29 --output target/p29/package/AiDENs-p29-codex-context.zip
python3 scripts/assert_p29_package_self_replay.py --package target/p29/package/AiDENs-p29-codex-context.zip
```

## Hostile audit checklist

1. Extract the package into a temp directory.
2. Run `bash scripts/verify_current.sh`.
3. Run `bash scripts/p29_verify.sh`.
4. Check P29 docs/handoffs/scripts are present.
5. Check no P29 artifacts are archived as stale.
6. Check manifest paths resolve.
7. Check support labels do not overclaim.
8. Check v11A local release candidate evidence exists.
9. Check v11B seed labels are seed-only.
10. Check v11C is reserved-only.
