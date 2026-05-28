# P17 Handoff - Attested Exchange, Federation, and External Admission

## Summary

P17 is implemented only. External artifacts now cross the boundary through typed attestation and admission artifacts, remote oracle imports remain advisory unless locally promoted through governance, remote contradictions create settlement artifacts instead of overwriting local claims, and trust-root revocation downgrades affected admissions with receipts.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-delegation-kit/Cargo.toml`
- `crates/aidens-delegation-kit/src/lib.rs`
- `crates/aidens-governance-kit/Cargo.toml`
- `crates/aidens-governance-kit/src/lib.rs`
- `crates/aidens-memory-kit/src/lib.rs`
- `crates/aidens-receipts/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `scripts/assert_no_scaffold_promoted.sh`
- `tests/fixtures/p17/*.json`
- `schemas/` regenerated, including P17 schema directories
- `README.md`
- `STATUS.md`
- `ARTIFACT_SCHEMA_REGISTRY.md`
- `handoffs/P17_ATTESTED_EXCHANGE_FEDERATION_AND_EXTERNAL_ADMISSION.md`

## Tests Added

- Contract constructor and golden-fixture tests for `AttestationEnvelopeV1`, `TrustRootV1`, `AdmissionDecisionV1`, `RemoteOracleReceiptV1`, `TreatyV1`, `SettlementCaseV1`, and `SharedDispositionV1`.
- Delegation tests for verified admission, missing/mismatched admission decisions, unsigned truth-bearing quarantine, rejected import denial receipts, remote contradiction settlement, and trust-root revocation downgrade receipts.
- Governance tests for local promotion of remote admissions and settlement shared-disposition requirements.
- Memory test proving remote oracle import is advisory and settlement does not overwrite local claims.
- Receipt-store test proving P17 artifacts append to durable receipt envelopes/outbox rows.
- CLI scaffold-status tests updated so `aidens-delegation-kit` is no longer scaffold-only.

## Commands Run

```bash
cargo check -p aidens-contracts
cargo check -p aidens-delegation-kit
cargo check -p aidens-memory-kit -p aidens-receipts
cargo check -p aidens-governance-kit
cargo check -p aidens-cli
cargo run -q -p aidens-cli -- schemas generate
cargo test -p aidens-contracts p17
cargo test -p aidens-delegation-kit
cargo test -p aidens-memory-kit remote_oracle
cargo test -p aidens-governance-kit remote_admission
cargo test -p aidens-governance-kit settlement_governance
cargo test -p aidens-receipts p17
cargo test -p aidens-cli scaffold
cargo test -p aidens-cli package
cargo run -q -p aidens-cli -- schemas check
cargo fmt --all
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
```

## Blockers

None for P17. The full required gate passed.

## Next-Pass Readiness

P18 is unblocked from the P17 substrate perspective. Mechanism search should build on the advisory external-admission model and must not treat remote artifacts as local truth without `AdmissionDecisionV1` plus local governance promotion.
