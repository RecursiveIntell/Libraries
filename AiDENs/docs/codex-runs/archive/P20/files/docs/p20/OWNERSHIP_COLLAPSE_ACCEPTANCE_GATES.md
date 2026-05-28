# Ownership Collapse Acceptance Gates

## Global pass/fail rule

A phase passes only if all required gates pass and evidence is saved.

## Gate table

| Gate | Script | Must prove |
|---|---|---|
| No crate split | `assert_no_crate_split.sh` | `aidens-contracts` was not split into new crates |
| Source basis current | `assert_docs_source_basis_current.sh` | docs do not refer to stale source snapshots as current |
| Duplicate canonical type block | `assert_no_canonical_type_duplicates.py` | no local public type exact-name duplicates canonical public types |
| Ownership inventory | `make_type_ownership_inventory.py` | current type map exists and is reviewable |
| No local canonical digest law | `assert_no_local_canonical_digest_law.sh` | no exported local digest/ID law remains |
| Schema scope | `assert_schema_generation_scope.py` | AiDENs does not emit canonical family schemas |
| Tool runtime delegation | `assert_tool_runtime_delegation.sh` | tool surfaces are grounded in `llm-tool-runtime` |
| Wrapper backpointers | `assert_wrapper_backpointers.sh` | report/display wrappers have canonical refs/backpointers or are quarantined |
| No compatibility ledger entries | `assert_no_compatibility_ledgers.sh` | no compatibility shims were introduced |
| Full phase verification | `phase_verify_contract_ownership.sh` | runs all relevant gates and saves output |

## Final acceptance

The final run must produce:

- `docs/contract-ownership/FINAL_OWNERSHIP_COLLAPSE_REPORT.md`
- `docs/contract-ownership/FINAL_TYPE_OWNERSHIP_INVENTORY.csv`
- `docs/contract-ownership/FINAL_QUARANTINE_LEDGER.md`
- `docs/contract-ownership/FINAL_GATE_OUTPUTS.md`
- `docs/contract-ownership/FINAL_AUDITOR_HANDOFF.md`

## Failure rule

If any gate fails, do not continue. Either repair the violation or quarantine and stop.
