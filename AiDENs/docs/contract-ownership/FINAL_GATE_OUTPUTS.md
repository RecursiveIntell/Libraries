# Final Gate Outputs

SOURCE BASIS: 2026-04-28

## Final Verification

Command:

```bash
bash scripts/phase_verify_contract_ownership.sh final
```

Captured output:

- `.codex_evidence/contract_ownership/07/phase_verify_final_initial.txt`
- `.codex_evidence/contract_ownership/final/`

Key final output:

```text
PASS: no aidens-contracts split crates detected.
PASS: no blocking stale source-basis docs detected.
canonical_types=633
aidens_contracts_types=194
duplicate_findings=0
PASS: no local aidens-contracts public type definitions duplicate canonical public type names.
PASS: no exported local canonical digest law detected.
PASS: schema generation scope appears AiDENs-local/non-authoritative (registered_families=58, checked_schema_files=58).
PASS: tool runtime delegation gate did not find blocking local-only tool truth.
PASS: wrapper backpointer gate did not find blocking risky wrappers.
PASS: no compatibility ledger entries or obvious compat/shim files detected.
PASS: no local substitute dependency red flags detected.
PASS: contract ownership verification passed.
```

## Cargo Verification

Commands:

```bash
cargo check --workspace
cargo test --workspace
```

Captured outputs:

- `.codex_evidence/contract_ownership/07/cargo_check_workspace.txt`
- `.codex_evidence/contract_ownership/07/cargo_test_workspace.txt`

Result:

```text
cargo check --workspace: passed
cargo test --workspace: passed
```

## Final Inventory Paths

- `docs/contract-ownership/FINAL_TYPE_OWNERSHIP_INVENTORY.csv`
- `docs/contract-ownership/CANONICAL_TYPE_INVENTORY.csv`
- `docs/contract-ownership/AIDENS_CONTRACTS_TYPE_INVENTORY.csv`
- `docs/contract-ownership/CANONICAL_DUPLICATE_FINDINGS.csv`

## Final Quarantine and Compatibility Paths

- `docs/contract-ownership/FINAL_QUARANTINE_LEDGER.md`
- `docs/contract-ownership/COMPATIBILITY_LEDGER.md`

## Final Git Evidence

- `.codex_evidence/contract_ownership/07/git_status_before.txt`
- `.codex_evidence/contract_ownership/07/git_status_after.txt`
- `.codex_evidence/contract_ownership/07/git_diff_stat.txt`
- `.codex_evidence/contract_ownership/07/git_diff.patch`
