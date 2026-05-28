# Rollback and Quarantine Plan

## Purpose

This run is allowed to delete, replace, and rewire ownership. It is not allowed to hide ambiguity or failed attempts.

## Pre-phase snapshot

Before each phase:

```bash
mkdir -p .codex_evidence/contract_ownership/PHASE_ID
git status --short > .codex_evidence/contract_ownership/PHASE_ID/git_status_before.txt
git diff --binary > .codex_evidence/contract_ownership/PHASE_ID/git_diff_before.patch
```

If the repo is not a git checkout, create a tar snapshot of touched files before editing and record that git is unavailable.

## Rollback trigger

Rollback is required if:

- a phase gate fails and the failure is not locally repairable;
- a canonical owner cannot be found;
- an attempted conversion requires semantic invention;
- a cargo check failure is caused by the phase and the correct canonical adapter is unclear.

## Rollback procedure

1. Save current failure diff:

```bash
git diff --binary > .codex_evidence/contract_ownership/PHASE_ID/failing_diff_before_rollback.patch
git status --short > .codex_evidence/contract_ownership/PHASE_ID/status_before_rollback.txt
```

2. Write a rollback record:

```text
.codex_evidence/contract_ownership/PHASE_ID/rollback_record.md
```

Required fields:

```text
PHASE:
WHY ROLLBACK WAS NEEDED:
FILES REVERTED:
GATE THAT FAILED:
UNRESOLVED OWNER QUESTION:
SAFE NEXT ACTION:
```

3. Revert only the affected files:

```bash
git restore -- path/to/file
```

4. Save post-rollback status and gate output.

## Quarantine trigger

Create a quarantine record instead of inventing local semantics when:

- no canonical owner exists but the concept clearly belongs to stack law;
- multiple canonical crates appear to own a concept;
- a local report/display type is too close to truth semantics;
- preserving compatibility would require local reinterpretation;
- a replacement would require lossy conversion.

## Quarantine location

```text
docs/contract-ownership/quarantine/<TYPE_OR_CONCEPT>.md
```

## Quarantine record template

```text
# Quarantine: <TYPE_OR_CONCEPT>

STATUS: blocked | deferred | needs human owner decision
DISCOVERED_IN_PHASE:
LOCAL FILE/LINE:
LOCAL SYMBOLS:
SUSPECTED CANONICAL OWNER(S):
SEARCHES PERFORMED:
WHY AUTOMATIC COLLAPSE IS UNSAFE:
TEMPORARY ACTION TAKEN:
FORBIDDEN ACTIONS:
REQUIRED HUMAN DECISION:
RECOMMENDED NEXT RUN:
```

## Important

Quarantine is not a compatibility shim. It is a stop surface.
