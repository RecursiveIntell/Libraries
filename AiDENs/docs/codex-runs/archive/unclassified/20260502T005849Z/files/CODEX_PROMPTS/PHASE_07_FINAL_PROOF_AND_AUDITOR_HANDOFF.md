# PHASE 07 — Docs, Quarantine Ledger, Final Hostile Proof

## Objective

Finalize the ownership collapse with evidence strong enough for a hostile audit.

## Required actions

1. Run final verification:

```bash
bash scripts/phase_verify_contract_ownership.sh final
```

2. Run cargo checks if available:

```bash
cargo check --workspace
cargo test --workspace
```

If full checks are too expensive, run targeted checks and record exact skip rationale.

3. Produce final docs:

```text
docs/contract-ownership/FINAL_OWNERSHIP_COLLAPSE_REPORT.md
docs/contract-ownership/FINAL_TYPE_OWNERSHIP_INVENTORY.csv
docs/contract-ownership/FINAL_QUARANTINE_LEDGER.md
docs/contract-ownership/FINAL_GATE_OUTPUTS.md
docs/contract-ownership/FINAL_AUDITOR_HANDOFF.md
docs/contract-ownership/FINAL_UNRESOLVED_RISKS.md
docs/contract-ownership/DEPENDENCY_SOURCE_OF_TRUTH.md
```

4. Ensure `COMPATIBILITY_LEDGER.md` has no rows.
5. Ensure no crate split occurred.
6. Save final git status and diff stat.

## Acceptance

- All final gates pass.
- Final proof files exist.
- No unresolved P0 remains.
- Any unresolved P1/P2 ambiguity is quarantined and not silently worked around.
- Codex final response includes file paths and command evidence.

## Stop

Stop after this phase. Do not start a new refactor.
