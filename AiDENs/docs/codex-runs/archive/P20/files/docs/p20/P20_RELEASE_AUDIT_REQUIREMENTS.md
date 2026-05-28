# P20 Release Audit Requirements

Final audit output directory:

```text
target/aidens-final-audit/
```

Required contents:

```text
cargo-version.txt
rustc-version.txt
cargo-metadata.json
cargo-tree.txt
fmt.log
check.log
test.log
clippy.log
verify.log
p20-scan/
phase-reports/
CONTRACT_OWNERSHIP_INVENTORY.md
PROVIDER_CAPABILITY_MATRIX.md
DOCS_CODE_TRUTH_REPORT.md
AGENCY_EVAL_REPORT.md
REFERENCE_INTERPRETER_CLOSEOUT.md
KNOWN_LIMITATIONS.md
RELEASE_READINESS.md
FINAL_AUDITOR_HANDOFF.md
```

The handoff must say either:

- `P20 PASS`, or
- `P20 FAILED` with blockers.
