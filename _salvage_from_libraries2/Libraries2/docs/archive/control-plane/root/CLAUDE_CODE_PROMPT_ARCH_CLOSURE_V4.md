# Claude Code Prompt — Architecture Closure V4

You are working on the latest full snapshot of the Rust stack. Your job is to finish as much of the remaining architecture-closure work as possible **while remaining safe**.

## Read first
1. `MASTER_ISSUE_MATRIX_ARCH_CLOSURE_V4.md`
2. `FILE_AUDIT_INVENTORY_ARCH_CLOSURE_V4.md`
3. `ARCHITECTURE_CLOSURE_EXECUTION_MAP_V4.md`
4. `ARCHITECTURE_CLOSURE_ACCEPTANCE_CHECKLIST_V4.md`
5. Existing repo docs such as root `CLAUDE.md` and affected crate READMEs

## Core instructions
- Work issue-by-issue using the matrix IDs.
- Prefer strict validation, explicit compatibility labeling, and stronger tests over broad speculative redesign.
- Preserve the authority map and dependency direction.
- Do not force `semantic-memory` to depend directly on bridge-owned Rust types if the logical serialized boundary can remain strict and safe.
- Do not silently break compatibility callers unless the matrix explicitly pushes for removal and you can update all affected consumers in this repo.
- If you defer an issue, leave a crisp comment and/or doc note explaining exactly why.

## Highest-priority work
- `SM-003`, `SM-004`, `BRG-001`, `BRG-002`, `KR-001`, `KR-002`, `KR-003`, `KR-005`, `AG-001`, `E2E-001`, `DOC-001`

## Required output from your run
- Code changes grouped cleanly by issue ID
- Updated tests
- Updated docs/comments where semantics changed
- A short summary of what was completed, what was deferred, and why
