# P28 Phase 13 Report

## Scope

Added an adversarial conformance suite with an explicit fixture manifest and active integration tests for boundary, tool sandbox, patch, timeout, proof/debt, degraded readiness, temporal stale projection, reserved v11B/v11C, and agency-risk semantics.

## Files changed

- `crates/aidens-integration-tests/Cargo.toml`
- `crates/aidens-integration-tests/tests/p28_adversarial_conformance.rs`
- `tests/fixtures/p28/adversarial_conformance_manifest.json`
- `handoffs/p28/PHASE_13_REPORT.md`

## Claims made

- Claim: every required Phase 13 adversarial fixture has declared expected pass/fail semantics.
  - status: pass
  - evidence: `tests/fixtures/p28/adversarial_conformance_manifest.json`, `p28_adversarial_manifest_declares_expected_semantics_for_every_fixture`
- Claim: duplicate keys, schema mismatch, and treatment-changing parser repair fail closed.
  - status: pass
  - evidence: `p28_adversarial_boundary_fixtures_fail_closed`
- Claim: symlink escape and failed patch-write dirty-directory cases are blocked.
  - status: pass
  - evidence: `p28_adversarial_tool_sandbox_blocks_symlink_and_dirty_patch_paths`
- Claim: timeout, proof waiver, degraded aggregate, and stale projection semantics remain degraded/blocked as required.
  - status: pass
  - evidence: `p28_adversarial_receipt_proof_degradation_and_temporal_fixtures_hold`
- Claim: reserved v11B/v11C and personalized-advice fixtures do not promote future truth.
  - status: pass
  - evidence: `p28_adversarial_reserved_horizon_and_agency_fixtures_do_not_promote_truth`

## Evidence

- `target/p28/audit/cargo_fmt_phase13_adversarial_after_fix.log`
- `target/p28/audit/cargo_test_integration_p28_adversarial_phase13_after_fix.log`

## Tests run

```bash
cargo fmt --all -- --check
cargo test -p aidens-integration-tests --test p28_adversarial_conformance
```

## Failures / degraded checks

- Initial compile failed because the test asserted a non-existent `failed` field on `ToolInvocationReportV1`; repaired to assert `!succeeded`.
- Initial formatting check failed before `cargo fmt --all`; repaired and rechecked.

## Open risks

- The suite validates expected local semantics. It does not activate v11B/v11C future owner behavior.

## Next phase readiness

Ready: proceed to Phase 14.
