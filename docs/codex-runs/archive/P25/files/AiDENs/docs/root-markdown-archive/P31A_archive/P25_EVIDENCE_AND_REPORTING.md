# P25 Evidence and Reporting Requirements

## Every phase report must include

- phase number and name,
- changed files,
- commands run,
- command outputs or summaries,
- invariant revalidation,
- support-claim impact,
- unresolved risks,
- next-step readiness.

## Required final evidence files

```text
P25_STATUS_EVIDENCE_MANIFEST.json
docs/p25/P25_FINAL_AUDIT_REPORT.md
docs/p25/P25_KNOWN_LIMITATIONS.md
handoffs/p25/FINAL_AUDITOR_HANDOFF.md
docs/root-markdown-archive/<timestamp>/ROOT_MARKDOWN_ARCHIVE_MANIFEST.json
```

## Evidence manifest fields

```json
{
  "run_id": "P25",
  "created_utc": "...",
  "package_sha256": "...",
  "commands": [],
  "changed_files": [],
  "root_markdown_archive": {},
  "phase_gates": [],
  "support_claims": [],
  "validation_results": [],
  "known_limitations": [],
  "unresolved_risks": []
}
```

## Reporting rule

Do not say a gate passed unless its evidence file or command output exists. Absence of evidence is a failure or unresolved risk.
