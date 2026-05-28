# Evidence Reporting Requirements

## Evidence root

Every phase must write evidence to:

```text
.codex_evidence/contract_ownership/<phase>/
```

## Required files per phase

| File | Required content |
|---|---|
| `git_status_before.txt` | `git status --short` before edits |
| `git_status_after.txt` | `git status --short` after edits |
| `git_diff_stat.txt` | `git diff --stat` after edits |
| `git_diff.patch` | `git diff --binary` after edits |
| `commands_run.txt` | Commands run, in order |
| `gate_outputs.txt` | Exact stdout/stderr for gates |
| `phase_report.md` | Human-readable phase report |
| `quarantine_delta.md` | New/updated quarantine records |
| `skipped_checks.md` | Any check skipped and exact reason |

## Command capture rule

Every command must be captured with enough detail for replay:

```text
COMMAND:
WORKING_DIRECTORY:
START_TIME:
EXIT_STATUS:
STDOUT:
STDERR:
```

## Final evidence pack

At final phase, produce:

```text
docs/contract-ownership/FINAL_GATE_OUTPUTS.md
docs/contract-ownership/FINAL_AUDITOR_HANDOFF.md
docs/contract-ownership/FINAL_UNRESOLVED_RISKS.md
```

## Cargo evidence

If full workspace cargo checks are too expensive, Codex must still run targeted checks and explicitly report:

- why full check was skipped;
- what targeted checks were run;
- what remains unverified.

Skipping without a reason is failure.
