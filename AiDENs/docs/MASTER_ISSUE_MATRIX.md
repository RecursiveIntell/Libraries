# Documentation Issue Matrix - Active P20 View

This file mirrors the active P20 issue matrix at the docs level. Historical P00-P19 matrices and handoffs are evidence packets, not current support claims.

| Area | Current label | Evidence | Next owner |
|---|---|---|---|
| Build truth | `supported` | Phase 01 report and logs | none for Phase 02 |
| Root README truth | `supported` after Phase 02 patch | `README.md`; `DOCS_CODE_TRUTH_REPORT.md` | Phase 02 |
| STATUS truth | `supported` after Phase 02 patch | `STATUS.md`; crate table | Phase 02 |
| Source basis truth | `supported` after Phase 02 patch | `SOURCE_BASIS.md`; Phase 00/01 logs | Phase 02 |
| Contract ownership inventory | `supported` after Phase 03 inventory | `docs/p20/CONTRACT_OWNERSHIP_INVENTORY.md`; Phase 03 report | none |
| Scanner integration | `supported` after Phase 04 guardrail pass | `scripts/p20_scan_aidens.py`; `scripts/p20_verify.sh`; Phase 04 report | none |
| Provider capability matrix | `supported` after Phase 05 provider truth pass | `docs/p20/PROVIDER_CAPABILITY_MATRIX.md`; Phase 05 report | none |
| Runner vertical slice | `partial/proved` | `tests/fixtures/p06/runner_vertical_slice_aidens.toml`; `crates/aidens-app-kit/tests/phase_06_runner_vertical_slice.rs`; Phase 06 report | none |
| Canonical adapter proofs | `partial/proved` | `crates/aidens-integration-tests/tests/phase_07_canonical_adapter_proof.rs`; Phase 07 report | none |
| Agency governance | `partial/proved` | `crates/aidens-agency-kit/src/lib.rs`; `crates/aidens-runner/tests/phase_08_agency_gate.rs`; Phase 08 report | none |
| Reference interpreter closeout | `partial/proved` | `crates/aidens-testkit/src/lib.rs`; `crates/aidens-integration-tests/tests/phase_09_reference_hostile_tests.rs`; Phase 09 report | none |
| Final audit bundle | `deferred` | P20 Phase 10 pending | Phase 10 |
