# Re-run and Verification Instructions

## Normal re-run

```bash
bash scripts/phase_verify_contract_ownership.sh final
```

## Expanded re-run

```bash
python3 scripts/make_type_ownership_inventory.py
python3 scripts/assert_no_canonical_type_duplicates.py
bash scripts/assert_no_local_canonical_digest_law.sh
python3 scripts/assert_schema_generation_scope.py
bash scripts/assert_tool_runtime_delegation.sh
bash scripts/assert_wrapper_backpointers.sh
bash scripts/assert_no_crate_split.sh
bash scripts/assert_no_compatibility_ledgers.sh
```

## Cargo re-run

```bash
cargo check --workspace
cargo test --workspace
```

If those are too expensive, run:

```bash
cargo check -p aidens-contracts
cargo test -p aidens-contracts
cargo check -p aidens-tool-kit
cargo test -p aidens-tool-kit
```

## Regression trigger

Re-run gates after any change to:

- `crates/aidens-contracts`;
- root `Cargo.toml`;
- any schema generation path;
- tool/runtime/repair wrapper code;
- docs declaring source basis or ownership.
