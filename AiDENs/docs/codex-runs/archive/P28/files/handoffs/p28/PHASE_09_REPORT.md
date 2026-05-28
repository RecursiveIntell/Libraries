# P28 Phase 09 Report

## Scope

Implemented the v11A bitemporal reference fixture matrix for the declared local memory/query conformance seam and revalidated the existing canonical-memory production differential for combined `as_of(valid, recorded)` behavior.

## Files changed

- `crates/aidens-testkit/src/lib.rs`
- `handoffs/p28/PHASE_09_REPORT.md`

## Claims made

- Claim: valid-time-only temporal reference fixture exists and is executable.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_testkit_p28_bitemporal_matrix_phase09_closeout.log`
- Claim: recorded-time-only temporal reference fixture exists and is executable.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_testkit_p28_bitemporal_matrix_phase09_closeout.log`
- Claim: combined `as_of(valid, recorded)` reference behavior remains executable.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_testkit_p28_temporal_combined_phase09.log`
- Claim: retroactive correction and supersession fixture exists and hides superseded candidate versions.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_testkit_p28_bitemporal_matrix_phase09_closeout.log`
- Claim: stale projection fixture cannot answer as exact/current without disclosure.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_testkit_p28_bitemporal_matrix_phase09_closeout.log`
- Claim: degraded temporal fixture emits view/degradation disclosure requirements.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_testkit_p28_bitemporal_matrix_phase09_closeout.log`
- Claim: declared production memory/query path is differentially checked against the combined temporal reference case.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_integration_p28_temporal_production_diff_phase09.log`

## Evidence

- `target/p28/audit/cargo_fmt_phase09.log`
- `target/p28/audit/cargo_check_phase09.log`
- `target/p28/audit/cargo_test_aidens_testkit_p28_temporal_combined_phase09.log`
- `target/p28/audit/cargo_test_aidens_testkit_p28_bitemporal_matrix_phase09_closeout.log`
- `target/p28/audit/cargo_test_aidens_integration_p28_temporal_production_diff_phase09.log`

## Tests run

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p aidens-testkit temporal_query_reference_case_interprets_as_of_semantics
cargo test -p aidens-testkit p28_bitemporal_reference_fixture_matrix_interprets_required_cases
cargo test -p aidens-integration-tests temporal_asof_reference_matches_canonical_memory_runtime
```

## Failures / degraded checks

- `target/p28/audit/cargo_fmt_phase09_initial.log` failed before mechanical formatting was applied.

## Open risks

- Production differential coverage is direct for the combined as-of path. Valid-only, recorded-only, stale-projection, and degraded-disclosure cases are executable reference fixtures; they are not claimed as new production memory behavior.

## Next phase readiness

Ready: Phase 09 exit gate passed for the declared production combined temporal path, with additional fixture coverage established before any memory behavior expansion.
