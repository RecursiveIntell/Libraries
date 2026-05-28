# PHASE 03 — Canonical Dependency and Re-export Wiring

## Objective

Ensure missing canonical owner crates are real dependencies, not implicit local substitutes.

## Required actions

1. Check root `Cargo.toml` and affected crate `Cargo.toml` files.
2. Add dependencies where the code surfaces those canonical concepts:
   - `attestation-exchange`
   - `federated-settlement`
   - `mechanism-runtime`
   - `remote-oracle-admission` if remote oracle admission surfaces remain
   - `contract-schema-gen` where schema generation is invoked/owned
   - `verification-calibration` if calibration surfaces are referenced
   - `forge-pilot` if pilot/control-loop receipts are referenced
3. Confirm dependencies point to `~/Coding/Libraries`, not `Libraries2`.
4. Update `docs/contract-ownership/DEPENDENCY_SOURCE_OF_TRUTH.md`.
5. Run dependency and duplicate gates.

## Acceptance

- Real canonical dependencies are wired where required.
- No `Libraries2` dependency is introduced when `Libraries` owner exists.
- No local substitute module exists for a missing canonical crate.
- Build metadata/check is run if available; otherwise skip rationale is saved.

## Stop

Stop after this phase and wait for `GUARDRAIL_03_TO_04`.
