# Plan: Libraries Hostile Audit Council (Post-AiDENs hardening)

**Created**: 2026-07-16
**Status**: Complete — audit closeout recorded in `LIBRARIES_COUNCIL_HOSTILE_AUDIT_2026-07-16.md`

## Goal
Run a hostile-audit sweep across canonical `Coding/Libraries` repositories to find high-risk runtime/surface issues now that `AiDENs` is hardened, and prepare a ranked fix list with evidence.

## Analysis
AiDENs has completed its P32 closeout and clean gates locally. Remaining risk is likely in sibling/derived repositories that still expose tool execution, dynamic parsing, unsafe process behavior, and evidence/reporting surfaces.

## Files to Read First
- `Coding/Libraries/AGENTS.md`
- `Coding/Libraries/02_MASTER_ISSUE_MATRIX.md`
- `Coding/Libraries/04_EXACT_FILE_TOUCH_MAP.md`
- `Coding/Libraries/05_ACCEPTANCE_GATES.md`

## Proposed Approach
1. Run quick preflight: git state, root issue matrix, and high-signal hostile patterns.
2. Dispatch 12 audit agents across major library crates for independent checks.
3. Re-run targeted validation commands per crate (p30_guard/verify/check/test/clippy where available).
4. Consolidate into one hostile-audit closeout with severity ordering and concrete remediation steps.

## Verification
- [ ] Collect gate output per target crate/repo
- [ ] Collect p30-like findings and hard findings
- [ ] Produce severity-ranked matrix of findings
- [ ] Propose concrete fixes and ownership + files

## Progress Log
- 2026-07-16: Plan created for hostile audit council.
- 2026-07-16: Read-only sequential audit completed. P0 release blockers and ranked remediations recorded in `LIBRARIES_COUNCIL_HOSTILE_AUDIT_2026-07-16.md`; no source or release evidence was mutated.
