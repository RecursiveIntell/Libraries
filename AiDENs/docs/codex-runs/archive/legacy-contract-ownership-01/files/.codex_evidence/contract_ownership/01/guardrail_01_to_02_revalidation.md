# GUARDRAIL_01_TO_02 Revalidation

Date: 2026-04-29

1. Generated canonical type inventory path:
   - `docs/contract-ownership/CANONICAL_TYPE_INVENTORY.csv`
   - Observed rows: 696.

2. Generated AiDENs contracts type inventory path:
   - `docs/contract-ownership/AIDENS_CONTRACTS_TYPE_INVENTORY.csv`
   - Observed rows: 204.

3. Exact duplicate findings path:
   - `docs/contract-ownership/CANONICAL_DUPLICATE_FINDINGS.csv`

   Exact findings:

   ```csv
   type_name,aidens_file,aidens_line,canonical_owner,canonical_file,canonical_line,severity
   AttestationEnvelopeV1,crates/aidens-contracts/src/lib.rs,2481,attestation-exchange,attestation-exchange/src/lib.rs,117,P0
   SharedDispositionV1,crates/aidens-contracts/src/lib.rs,2843,federated-settlement,federated-settlement/src/lib.rs,95,P0
   SettlementCaseV1,crates/aidens-contracts/src/lib.rs,2891,federated-settlement,federated-settlement/src/lib.rs,144,P0
   TheoryRefuterSuiteV1,crates/aidens-contracts/src/lib.rs,3508,mechanism-runtime,mechanism-runtime/src/lib.rs,131,P0
   TheoryVersionV1,crates/aidens-contracts/src/lib.rs,3603,mechanism-runtime,mechanism-runtime/src/lib.rs,61,P0
   HypothesisLibraryV1,crates/aidens-contracts/src/lib.rs,3815,mechanism-runtime,mechanism-runtime/src/lib.rs,81,P0
   ```

4. Six P0 exact duplicates detected:
   - `AttestationEnvelopeV1`
   - `SharedDispositionV1`
   - `SettlementCaseV1`
   - `TheoryRefuterSuiteV1`
   - `TheoryVersionV1`
   - `HypothesisLibraryV1`

   Revalidation found no missing or extra P0 duplicate names.

5. Same-name symbols are allowed only if they are explicit `pub use` re-exports:
   - `scripts/make_type_ownership_inventory.py` scans `pub struct`, `pub enum`, and `pub type` as `definition_kind=local_def`.
   - It separately scans `pub use ...;` as `definition_kind=pub_use`.
   - Duplicate findings are generated only from AiDENs rows where `definition_kind == "local_def"`.
   - Therefore same-name local public definitions are blocking findings; explicit `pub use` rows are inventoried separately and are not treated as local duplicate definitions.

6. No hand-maintained denylist is the only enforcement mechanism:
   - The gate scans canonical owner crates listed in `CANONICAL_CRATES`.
   - It scans `crates/aidens-contracts/src/lib.rs`.
   - It builds `canon_by_name` from scanned canonical `local_def` rows.
   - It emits findings by matching scanned AiDENs `local_def` names against scanned canonical names.
   - The six-name set is used only to classify severity as `P0`; all other scanned duplicates are still emitted as `P1_REVIEW`.

Result: the generated gate detects the known P0 duplicates. Phase 02 has not been started by this revalidation.
