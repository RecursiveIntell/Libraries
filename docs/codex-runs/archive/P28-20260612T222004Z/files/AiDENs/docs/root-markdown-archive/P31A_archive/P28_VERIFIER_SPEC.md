# P28 Verifier Spec

## Goal

Extend `scripts/verify_current.sh` so P28 can verify v11A declared-core conformance without pretending to prove full v11+ compliance.

## Required verifier sections

1. **Source basis check**
   - `P28_SOURCE_BASIS.md` exists.
   - `P28_BUG_ABSORPTION_MATRIX.csv` exists and includes 72 Claude findings.
   - `P28_STATUS_EVIDENCE_MANIFEST.json` exists for final pass.

2. **P0 bug closure check**
   - scan P28 manifest for every C05/C07/C11/C24/C25/C32/C53/C54/C55/C59/C66/C72 status.
   - every P0 must be `fixed` or `quarantined_release_blocking`; final success requires `fixed` unless explicit scope downgrade is recorded.

3. **v11A artifact family check**
   - code contains/admits artifact envelope, manifest, lifecycle, transition receipt, execution context, operator contract, invocation receipt, tool receipt, boundary compiler profile, proof debt/waiver, view disclosure, degradation record.
   - if names differ, manifest must provide admitted aliases.

4. **material operation registry check**
   - required operator IDs are present.
   - each has effects, forbidden effects, pre/postconditions, replay requirements, failure taxonomy.

5. **receipt gate check**
   - tests include no-done-without-receipts.
   - run bundle store overwrite test exists.
   - event log digest-chain/tamper test exists if hash chain is implemented.

6. **boundary compiler check**
   - duplicate-key test exists.
   - unknown-field/schema mismatch test exists.
   - parser repair/treatment-integrity test exists.

7. **proof economy check**
   - waiver-is-not-proof test exists.
   - proof debt restricts use/promotion test exists.
   - degraded release readiness test exists.

8. **tool/patch/sandbox hostile check**
   - symlink read/write escape tests exist.
   - patch dirty-dir rollback test exists.
   - command allowlist test exists.
   - timeout partial-output test exists.

9. **status honesty check**
   - if any subcheck has `degraded_exact_check`, aggregate semantic status cannot be `exact_check`.
   - package hash is labeled zip-byte hash.
   - final package path in manifest matches final sidecar names.

## Final verifier status values

- `pass_exact`: all required checks exact.
- `pass_degraded`: all safety checks pass, but at least one admitted degraded check exists; release label must reflect degraded status.
- `fail`: any required safety/v11A gate fails.

## Non-goal

The verifier does not prove full v11B or v11C active compliance. It verifies v11A declared-core and v11C reserved non-shadow behavior only.
