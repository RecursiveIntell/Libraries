# Codex short prompt

```text
Read the closeout bundle files first, especially:
- 00_START_HERE.md
- 04_MASTER_ISSUE_MATRIX.md
- 06_CRATE_SPLIT_PLAN.md
- 08_EXACT_FILE_TOUCH_MAP.md
- 10_ACCEPTANCE_AND_COMMANDS.md

Then implement the closeout pass in code.

Critical rules:
- root workspace crates are truth
- `libraries-source/` is a mirror, not the development authority
- preserve the already-landed fixes (`agent-graph` error cleanup, CEA confidence hardening, single canonical `PilotError`)
- do the `forge-pilot` and `knowledge-runtime` splits for real
- remove the known production panic edges
- tighten `stack-ids`
- add root toolchain/lint/deny/nextest configs and stronger scripts/CI
- make mirror drift exact-failing
- do not stop at planning
- do not count shell-game refactors as completion
```
