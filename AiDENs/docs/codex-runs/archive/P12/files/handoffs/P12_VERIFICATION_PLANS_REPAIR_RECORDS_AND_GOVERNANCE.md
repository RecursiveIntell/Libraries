# P12 Handoff - Superseded Local Verification/Governance DTOs

This historical handoff has been superseded by the canonical-library doctrine:
AiDENs no longer owns local verification-plan, evidence-bundle, refutation,
contradiction, repair, governance-decision, or promotion-receipt schemas.
Current adapters call `verification-control`, `verification-policy`,
`verification-adjudication`, and `semantic-memory-forge` directly.

## Scope

Implemented P12 only. No P13 multi-view runtime, regional decoder, subtraction, federation, or mechanism work was started.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-governance-kit/src/lib.rs`
- `crates/aidens-repair-kit/Cargo.toml`
- `crates/aidens-repair-kit/src/lib.rs`
- `crates/aidens-arbiter-kit/Cargo.toml`
- `crates/aidens-arbiter-kit/src/lib.rs`
- `crates/aidens-memory-kit/src/lib.rs`
- `crates/aidens-receipts/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `scripts/assert_no_scaffold_promoted.sh`
- `README.md`
- `STATUS.md`
- `ARTIFACT_SCHEMA_REGISTRY.md`
- `schemas/generated_schema_manifest_v1.json`

## Tests Added

- Current tests prove the adapters build or validate canonical verification, contradiction, and repair artifacts.
- CLI doctor scaffold-hygiene test updated so governance and repair are active P12 surfaces, not scaffold-only.

## Commands Run

```bash
cargo check -p aidens-contracts -p aidens-governance-kit -p aidens-repair-kit -p aidens-arbiter-kit -p aidens-memory-kit -p aidens-receipts
cargo test -p aidens-contracts p12
cargo test -p aidens-governance-kit
cargo test -p aidens-repair-kit
cargo test -p aidens-arbiter-kit
cargo test -p aidens-memory-kit contradiction
cargo test -p aidens-receipts p12
cargo test -p aidens-cli doctor_reports_scaffold_crates
cargo run -p aidens-cli -- schemas generate
cargo run -p aidens-cli -- schemas check
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
bash scripts/verify.sh
bash scripts/assert_no_fake_completion.sh .
bash scripts/assert_no_scaffold_promoted.sh .
```

Final result: all commands passed.

One intermediate `bash scripts/verify.sh` run failed because `STATUS.md` listed a test filter whose name matched the scaffold-promotion grep pattern. The status evidence was changed to the shorter `doctor_reports_scaffold_crates` filter, and the final `bash scripts/verify.sh` run passed.

## Blockers

None for P12.

P12 intentionally does not implement P13 runtime view disclosure or later regional/federation/mechanism behavior.

## Next-Pass Readiness

P12 artifacts are typed, schema-generated, fixture-backed, durably appendable, and wired through governance, repair, arbiter, memory, receipts, status, and doctor truth surfaces. The next pass may start at P13.
