#!/usr/bin/env bash
set -euo pipefail

out="docs/V11_DRIFT_REPORT.md"
timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

schema_status="pass"
if ! cargo run -p contract-schema-gen -- --check schemas >/tmp/v11-schema-check.log 2>&1; then
  schema_status="fail"
fi

cat > "$out" <<EOF
# V11 Drift Report

Generated at: \`$timestamp\`

## Schema Drift

- contract-schema-gen canonical schema check: \`$schema_status\`

## Workspace Drift

\`\`\`text
$(git diff --stat -- stack-ids/src/ids.rs semantic-memory-forge/src/lib.rs semantic-memory-forge/src/v11.rs verification-control/src/lib.rs forge-pilot/src/types.rs forge-pilot/src/decide.rs forge-pilot/src/receipts.rs forge-pilot/src/loop_runner.rs forge-pilot/tests/verification_control_tests.rs kernel-conformance/src/lib.rs kernel-conformance/src/reference_interpreters.rs kernel-conformance/tests/mixed_version_rollout.rs contract-schema-gen/src/lib.rs Makefile RELEASE_CHECKLIST.md scripts/check_v11_release_readiness.sh scripts/generate_v11_drift_report.sh schemas schemas.generated docs/PERFORMANCE_BASELINE.md 2>/dev/null || true)
\`\`\`

## Pending Tracked Changes

\`\`\`text
$(git status --short -- stack-ids/src/ids.rs semantic-memory-forge/src/lib.rs semantic-memory-forge/src/v11.rs verification-control/src/lib.rs forge-pilot/src/types.rs forge-pilot/src/decide.rs forge-pilot/src/receipts.rs forge-pilot/src/loop_runner.rs forge-pilot/tests/verification_control_tests.rs kernel-conformance/src/lib.rs kernel-conformance/src/reference_interpreters.rs kernel-conformance/tests/mixed_version_rollout.rs contract-schema-gen/src/lib.rs Makefile RELEASE_CHECKLIST.md scripts/check_v11_release_readiness.sh scripts/generate_v11_drift_report.sh schemas schemas.generated docs/PERFORMANCE_BASELINE.md 2>/dev/null || true)
\`\`\`

## Notes

- This report is structural only. It does not replace \`make gate\`, \`make test-living-memory\`, \`make test-ecosystem-smoke\`, or \`make perf-baseline\`.
- Mixed-version rollout proof lives in \`kernel-conformance/tests/mixed_version_rollout.rs\`.
EOF

echo "wrote $out"
