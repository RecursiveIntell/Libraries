#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
OUT="target/aidens-final-audit"
mkdir -p "$OUT/phase-reports"

# Copy known report artifacts if present.
for f in \
  docs/p20/CONTRACT_OWNERSHIP_INVENTORY.md \
  docs/p20/CONTRACT_OWNERSHIP_INVENTORY.json \
  docs/p20/PROVIDER_CAPABILITY_MATRIX.md \
  docs/p20/PROVIDER_CAPABILITY_MATRIX.json \
  docs/p20/DOCS_CODE_TRUTH_REPORT.md \
  docs/p20/AGENCY_EVAL_REPORT.md \
  docs/p20/REFERENCE_INTERPRETER_CLOSEOUT.md \
  docs/p20/KNOWN_LIMITATIONS.md \
  docs/p20/RELEASE_READINESS.md; do
  if [[ -f "$f" ]]; then cp "$f" "$OUT/"; fi
done
if [[ -d docs/p20/reports ]]; then cp -r docs/p20/reports/. "$OUT/phase-reports/"; fi

cat > "$OUT/AGENCY_EVAL_REPORT.md" <<'EOF'
# P20 Agency Eval Report

Status: `partial/proved`

Evidence:

- `evals/p20_agency_eval_cases.jsonl`
- `crates/aidens-agency-kit/src/lib.rs`
- `crates/aidens-runner/tests/phase_08_agency_gate.rs`
- `target/aidens-final-audit/agency-eval-fixture-validation.log`

Scope:

- high-impact single-path advice;
- decorative alternatives;
- repeated semantic paraphrase nudges;
- memory personalization vulnerability use;
- tool-output urgency/scarcity;
- delegated influence aggregation;
- exit-resistance/guilt hooks;
- sycophancy overvalidation;
- user-requested manipulation;
- sensitive receipt redaction.
EOF

cat > "$OUT/REFERENCE_INTERPRETER_CLOSEOUT.md" <<'EOF'
# P20 Reference Interpreter Closeout

Status: `partial/proved`

Evidence:

- `crates/aidens-testkit/src/lib.rs`
- `crates/aidens-integration-tests/tests/phase_09_reference_hostile_tests.rs`
- `target/aidens-final-audit/test.log`

Closed supported/delegated hostile surfaces:

- temporal/as-of reference semantics;
- bridge digest/backpointer atomicity;
- provider capability truth;
- agency decision semantics;
- boundary repair treatment integrity;
- runtime widening disclosure;
- repair-record invariants.

Remaining grep hits for deferred reference language are instructional, policy, or report text rather than executable supported-feature deferrals.
EOF

cat > "$OUT/KNOWN_LIMITATIONS.md" <<'EOF'
# P20 Known Limitations

- Cloud provider HTTP execution is not claimed.
- Native provider tool loops are not claimed; all current native-tool-loop capability flags are false.
- `mock` provider support is fixture-supported, not cloud support.
- `ollama` is a partial local chat boundary and depends on a local service; it is not a native tool-loop implementation.
- `aidens-profile-daemon`, `aidens-profile-desktop`, `aidens-profile-memory`, and `aidens-profile-research` are scaffold-only/deferred product surfaces; `aidens-plan-kit` is partial and owns execution-plan assembly only.
- Memory, evidence, temporal, kernel, verification, and repair semantics remain owned by canonical sibling crates; AiDENs only wires, adapts, reports, and gates.
- Agency/influence governance is proved for the tested AiDENs boundary and runner paths; this is not a broad product safety certification.
- Full desktop daemon lifecycle, federation transport, mechanism search, and regional inference product workflows remain outside the v0.1 finish line.
- The parent Git repository has historically reported `AiDENs/` as untracked from `/home/sikmindz/Coding/Libraries`; this is a repository-boundary risk, not a build-pass claim.
EOF

blockers=()
required_files=(
  cargo-version.txt
  rustc-version.txt
  cargo-metadata.json
  cargo-tree.txt
  fmt.log
  check.log
  test.log
  clippy.log
  verify.log
  p20-scan/p20_scan.json
  p20-scan/p20_scan.md
  CONTRACT_OWNERSHIP_INVENTORY.md
  PROVIDER_CAPABILITY_MATRIX.md
  DOCS_CODE_TRUTH_REPORT.md
  AGENCY_EVAL_REPORT.md
  REFERENCE_INTERPRETER_CLOSEOUT.md
  KNOWN_LIMITATIONS.md
)
for file in "${required_files[@]}"; do
  if [[ ! -f "$OUT/$file" ]]; then
    blockers+=("missing required audit artifact: $file")
  fi
done

for phase in $(seq -w 0 10); do
  if [[ ! -f "$OUT/phase-reports/PHASE_${phase}_REPORT.md" ]]; then
    blockers+=("missing phase report in bundle: PHASE_${phase}_REPORT.md")
  fi
done

if [[ ! -f "$OUT/p20-scan.log" ]] || ! grep -q "Blocking findings: 0" "$OUT/p20-scan.log"; then
  blockers+=("P20 scan did not report zero blocking findings")
fi
if [[ ! -f "$OUT/verify.log" ]] || ! grep -q "P20 verify completed" "$OUT/verify.log"; then
  blockers+=("p20_verify transcript is missing successful completion")
fi
if [[ ! -f "$OUT/agency-eval-fixture-validation.log" ]] || ! grep -q "Agency eval validation passed" "$OUT/agency-eval-fixture-validation.log"; then
  blockers+=("agency eval fixture validation did not pass")
fi

status="P20 PASS"
if ((${#blockers[@]} > 0)); then
  status="P20 FAILED"
fi

cat > "$OUT/RELEASE_READINESS.md" <<EOF
# P20 Release Readiness

Status: \`$status\`

## Gate Summary

- cargo fmt/check/test/clippy: see \`fmt.log\`, \`check.log\`, \`test.log\`, and \`clippy.log\`.
- repository verify script: see \`repo-verify.log\`.
- P20 scanner: see \`p20-scan.log\` and \`p20-scan/p20_scan.md\`.
- phase reports: see \`phase-reports/\`.
- agency eval fixture validation: see \`agency-eval-fixture-validation.log\`.

## Blockers

EOF
if ((${#blockers[@]} == 0)); then
  echo "- None." >> "$OUT/RELEASE_READINESS.md"
else
  for blocker in "${blockers[@]}"; do
    echo "- $blocker" >> "$OUT/RELEASE_READINESS.md"
  done
fi

cat > "$OUT/FINAL_AUDITOR_HANDOFF.md" <<EOF
# P20 Final Auditor Handoff

Status: $status

## Summary

AiDENs v0.1 is evaluated as a build-certified, documentation-honest orchestration layer over the canonical sibling Rust provenance stack if and only if this handoff status is \`P20 PASS\`.

## Commands Run

- \`bash scripts/p20_verify.sh\`
- \`bash scripts/p20_generate_audit_bundle.sh\`

## Gate Outputs

- Rust toolchain: \`rustc-version.txt\`, \`cargo-version.txt\`
- Cargo metadata/tree: \`cargo-metadata.json\`, \`cargo-tree.txt\`
- Format/check/test/clippy logs: \`fmt.log\`, \`check.log\`, \`test.log\`, \`clippy.log\`
- P20 scanner output: \`p20-scan/\`, \`p20-scan.log\`
- Phase reports: \`phase-reports/\`
- Contract ownership inventory: \`CONTRACT_OWNERSHIP_INVENTORY.md\`
- Provider capability matrix: \`PROVIDER_CAPABILITY_MATRIX.md\`
- Docs/code truth report: \`DOCS_CODE_TRUTH_REPORT.md\`
- Agency eval report: \`AGENCY_EVAL_REPORT.md\`
- Reference interpreter closeout: \`REFERENCE_INTERPRETER_CLOSEOUT.md\`
- Known limitations: \`KNOWN_LIMITATIONS.md\`
- Release readiness: \`RELEASE_READINESS.md\`

## Known Limitations

See \`KNOWN_LIMITATIONS.md\`. In short: cloud provider HTTP execution and native provider tool loops are not claimed; scaffold-only profile crates remain deferred; canonical crates own memory/evidence/temporal/kernel/verification/repair semantics; AiDENs gates and reports tested orchestration paths only.

## Blockers

EOF
if ((${#blockers[@]} == 0)); then
  echo "- None." >> "$OUT/FINAL_AUDITOR_HANDOFF.md"
else
  for blocker in "${blockers[@]}"; do
    echo "- $blocker" >> "$OUT/FINAL_AUDITOR_HANDOFF.md"
  done
fi

cat >> "$OUT/FINAL_AUDITOR_HANDOFF.md" <<'EOF'

## Auditor Re-run Commands

```bash
bash scripts/p20_verify.sh
bash scripts/p20_generate_audit_bundle.sh
```
EOF

cat > "$OUT/MANIFEST.txt" <<EOF
P20 final audit bundle generated at $(date -u +%Y-%m-%dT%H:%M:%SZ)
Root: $ROOT
Status: $status
EOF
find "$OUT" -maxdepth 2 -type f | sort >> "$OUT/MANIFEST.txt"

echo "Generated $OUT"
echo "Status: $status"
if [[ "$status" == "P20 FAILED" ]]; then
  printf '%s\n' "${blockers[@]}"
  exit 1
fi
