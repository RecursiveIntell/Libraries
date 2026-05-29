# SUPPORT_PROFILE.md

**Project**: AiDENs | **Run**: P31B | **Role**: verification-repair-candidate | **Label**: p31b-verification-repair-candidate

This profile cites `docs/codex-runs/CURRENT_RUN.json` as the single source of truth for run state.
**Last certified run**: P30.

## Claims

| Claim | Evidence | Status |
|---|---|---|
| Build passes | cargo check/test/fmt/clippy all exit 0 | proven |
| Boundary compiler | 28 tests pass, strict parse + sandbox enforcement | proven |
| Tool dispatch receipt | repo_read test emits receipt with tool_id + success | proven |
| Package validates | assert_package_validation.py PASS (1 non-fatal warning) | proven |
| Artifact classification | 659 P31A artifacts classified, active_run=P31B | proven |
| No hard p30_guard findings | 0 hard, 1842 broad (documented) | proven |

## Non-Claims

- No semantic-memory/TurboQuant certification beyond dependency declaration
- No LAN/cloud provider support
- `extracted_replay_certified=false`: environmental PermissionError in temp dir (acknowledged as not-certified, environmental blocker, not a code defect)
- a skipped post-bundle operator gate is not counted as a product defect
- Regenerated package sidecars and extracted-package self-replay are documented in CURRENT_RUN.json; the replay gate has an environmental blocker (PermissionError in temp dir)
- do not widen support labels beyond declared scope

## Support Level

P31B verification-repair-candidate. Not release-certified until full release gate passes.