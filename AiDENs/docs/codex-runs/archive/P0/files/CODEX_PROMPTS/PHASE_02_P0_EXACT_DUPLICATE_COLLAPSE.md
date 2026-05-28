# PHASE 02 — P0 Exact Duplicate Collapse

## Objective

Delete or convert exact duplicate canonical public types in `aidens-contracts`.

## Required actions

For each P0 duplicate:

| Type | Owner |
|---|---|
| `AttestationEnvelopeV1` | `attestation-exchange` |
| `SharedDispositionV1` | `federated-settlement` |
| `SettlementCaseV1` | `federated-settlement` |
| `TheoryRefuterSuiteV1` | `mechanism-runtime` |
| `TheoryVersionV1` | `mechanism-runtime` |
| `HypothesisLibraryV1` | `mechanism-runtime` |

1. Add the canonical dependency if missing.
2. Replace local definitions with explicit canonical import/re-export if the public surface needs the name.
3. Update all local usages to canonical types.
4. Do not create aliases that preserve local semantics.
5. Do not convert canonical artifacts into untyped JSON blobs.

## If API mismatch occurs

Do not invent adapters that reinterpret meaning. Create a quarantine record and halt if semantic mapping is unclear.

## Required gates

```bash
python3 scripts/assert_no_canonical_type_duplicates.py
bash scripts/assert_no_compatibility_ledgers.sh
```

## Acceptance

- No local `pub struct/enum/type` definition remains for the six P0 types.
- Any remaining symbol is an explicit `pub use` from canonical owner crate.
- Duplicate gate passes or only non-blocking quarantined ambiguity remains.
- Cargo check for affected crates runs if available.

## Stop

Stop after this phase and wait for `GUARDRAIL_02_TO_03`.
