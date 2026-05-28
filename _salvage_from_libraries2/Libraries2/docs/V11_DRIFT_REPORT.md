# V11 Drift Report

Generated at: `2026-03-13T23:37:13Z`

## Schema Drift

- contract-schema-gen canonical schema check: `pass`

## Workspace Drift

```text
 forge-pilot/src/decide.rs        | 138 ++++++
 forge-pilot/src/loop_runner.rs   | 981 +++++++++++++++++++++++++++++++++++++--
 kernel-conformance/src/lib.rs    |  71 ++-
 semantic-memory-forge/src/lib.rs |  22 +-
 stack-ids/src/ids.rs             | 264 ++++++++++-
 5 files changed, 1430 insertions(+), 46 deletions(-)
```

## Pending Tracked Changes

```text
 M forge-pilot/src/decide.rs
 M forge-pilot/src/loop_runner.rs
 M kernel-conformance/src/lib.rs
 M semantic-memory-forge/src/lib.rs
 M stack-ids/src/ids.rs
?? Makefile
?? RELEASE_CHECKLIST.md
?? contract-schema-gen/src/lib.rs
?? docs/PERFORMANCE_BASELINE.md
?? forge-pilot/src/receipts.rs
?? forge-pilot/src/types.rs
?? forge-pilot/tests/verification_control_tests.rs
?? kernel-conformance/src/reference_interpreters.rs
?? kernel-conformance/tests/mixed_version_rollout.rs
?? schemas.generated/
?? schemas/
?? scripts/check_v11_release_readiness.sh
?? scripts/generate_v11_drift_report.sh
?? semantic-memory-forge/src/v11.rs
?? verification-control/src/lib.rs
```

## Notes

- This report is structural only. It does not replace `make gate`, `make test-living-memory`, `make test-ecosystem-smoke`, or `make perf-baseline`.
- Mixed-version rollout proof lives in `kernel-conformance/tests/mixed_version_rollout.rs`.
