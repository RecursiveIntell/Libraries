# P22 Release Truth Requirements

A P22 release claim must cite executable evidence, not just prose.

## Required final artifacts

```text
handoffs/p22/FINAL_AUDIT_REPORT.md
handoffs/p22/KNOWN_LIMITATIONS.md
target/p22/audit/COMMAND_LOG_SUMMARY.md
target/p22/audit/CHANGED_FILE_SUMMARY.md
target/p22/audit/UNRESOLVED_RISKS.md
target/p22/archive_verifier_report.final.json
<normal package>.manifest.json
<normal package>.report.md
<normal package>.findings.json
<normal package>.excluded.json
```

## Required release facts

- Exact package path and SHA-256.
- Exact z.py archive summary.
- Whether archive history was excluded or deliberately included.
- Cargo command results.
- Assertion script results.
- Known limitations by support tier.
- Any skipped checks and reason.
- Any unclassified archived artifacts.
