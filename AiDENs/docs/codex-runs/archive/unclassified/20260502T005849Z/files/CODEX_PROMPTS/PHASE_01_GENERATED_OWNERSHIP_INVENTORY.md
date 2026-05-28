# PHASE 01 — Generated Ownership Inventory and Duplicate Gate

## Objective

Install a generated ownership inventory/gate before deleting anything. The gate must detect the known P0 duplicates.

## Required actions

1. Run:

```bash
python3 scripts/make_type_ownership_inventory.py
python3 scripts/assert_no_canonical_type_duplicates.py
```

2. Confirm the gate finds the known P0 duplicates before fixes:

- `AttestationEnvelopeV1`
- `SharedDispositionV1`
- `SettlementCaseV1`
- `TheoryRefuterSuiteV1`
- `TheoryVersionV1`
- `HypothesisLibraryV1`

3. Save outputs to:

```text
.codex_evidence/contract_ownership/01/
docs/contract-ownership/TYPE_OWNERSHIP_INVENTORY.csv
docs/contract-ownership/CANONICAL_DUPLICATE_FINDINGS.csv
```

4. If the generated gate does not catch known P0 duplicates, fix the gate, not the code.
5. Do not use a hand-maintained denylist as the only enforcement.

## Acceptance

- Generated inventory exists.
- Duplicate findings exist and include the six P0 duplicates.
- The script distinguishes local definitions from explicit `pub use` re-exports.
- No code deletion yet unless needed to repair script integration.

## Stop

Stop after this phase and wait for `GUARDRAIL_01_TO_02`.
