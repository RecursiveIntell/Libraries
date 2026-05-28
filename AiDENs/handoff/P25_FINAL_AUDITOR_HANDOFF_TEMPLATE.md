# P25 Final Auditor Handoff Template

## Run identity

- Run: P25
- Date:
- Operator:
- Package SHA-256:
- Git revision:
- Workspace dirty state:

## Claims

State only supported claims.

## Changed files

List all changed files.

## Commands run

List commands and results.

## Phase gate compliance

| Gate | Injection pasted? | Evidence | Result |
|---|---:|---|---|
| After Phase 01 |  |  |  |
| After Phase 03 |  |  |  |
| After Phase 05 |  |  |  |
| After Phase 07 |  |  |  |
| After Phase 09 |  |  |  |

## Root Markdown archive

- Manifest:
- Moved files:
- Protected files:
- Ambiguous files:

## Flagship demo evidence

- Demo path:
- Run bundle:
- Replay result:
- Receipts:

## Support profile

- Supported-local:
- Fixture-backed:
- Experimental:
- Deferred:

## Known limitations

List honestly.

## Unresolved risks

List honestly.

## Auditor instructions

A hostile auditor should first run:
1. `bash scripts/p25_verify.sh`
2. package validation
3. phase-gate integrity check
4. root Markdown archive dry-run
5. flagship demo replay
