# CURRENT_RUN.md

**Run**: P31B (verification-repair) | **Label**: p31b-verification-repair-candidate
**Status**: candidate
**Parent**: P31A
**Last Certified**: P30

## Certification

All verification gates passed. P31B is a verification-repair candidate run.
See `CURRENT_RUN.json` for full evidence.

## Resolved Findings

| ID | Finding | Resolution |
|---|---|---|
| S0-001 | False certification state | Decertified; all gates now pass |
| S0-002 | Verifier self-poisoning | Logs moved to target/verify-current/ |
| S0-003 | P31A recovery evidence unclassified | 659 artifacts classified |
| S0-004 | DIRECT_CHILD_KILL_ONLY hard finding | Already replaced with process-group termination |
| S0-005 | Receipt-grade command evidence missing | 15 receipts recorded |
| S1-006 | Package policy says P30 | z.py fix: letter-suffix normalization |
| S1-007 | Root markdown ambiguous count | root_markdown_archive_policy PASS |
| S1-008 | Package validation binding | assert_package_validation.py PASS for P31B |
| S1-009 | Build/test unproven | All cargo gates pass with receipts |
| S1-010 | Broad p30_guard findings | 1842 broad, 0 hard |
| S2-011 | Supported-local proof lacking | Vertical slice proven |
| Q-012 | No certified claims until gates pass | Now candidate |

## Known Limits

- `extracted_replay_certified=false`: self-replay blocked by environmental PermissionError in temp directory (not a code defect)
- 1842 broad p30_guard findings (documented, 0 hard)
- 1 non-fatal package warning: `script-ref-not-archived: p30_guard.py`