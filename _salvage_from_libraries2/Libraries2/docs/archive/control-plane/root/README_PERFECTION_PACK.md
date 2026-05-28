# Forge Workbench Perfection Pack

This pack is the updated completion bundle for the **current** Forge Workbench repository state.

It assumes the repository already has:

- a working Tauri/React shell
- concrete run routing
- cancel/retry controls
- durable settings and provider config persistence
- keyring-first secret storage
- provider test and model discovery commands
- a first-run setup page
- baseline capture through the queued worker

It also treats the latest screenshot and code review as first-class evidence. The current app **looks much better**, but the actual repair lane is still incomplete and the current failure is still happening in the **repo/baseline lane**, not in the LLM lane.

## What is in this pack

- `AGENTS.md` — canonical repo instructions for Codex or any other implementation agent
- `agents.md` — lowercase compatibility pointer
- `master_issue_matrix.csv` — the full issue matrix in tracker-friendly CSV form
- `plans/forge-workbench-v1-perfection.execplan.md` — the execution plan from the current repo state
- `docs/12_provider_setup_and_model_selection.md` — hardened provider/setup/model-routing spec
- `docs/13_settings_persistence_and_secret_handling.md` — settings and secret-handling contract
- `docs/14_master_issue_matrix.md` — human-readable matrix and sequencing guide
- `docs/15_completion_acceptance_test_plan.md` — end-to-end acceptance and release test contract
- `docs/16_repo_state_audit.md` — current-state audit grounded in the repo and screenshots
- `docs/17_run_failure_diagnostics_and_repo_preflight.md` — immediate corrective spec for the current failure class
- `docs/18_truthful_phase_model.md` — corrected run phase/status model
- `prompts/codex_master_prompt.md` — full Codex handoff prompt
- `prompts/codex_short_prompt.md` — compact Codex prompt

## Matrix totals

- Total issues: 47
- Done: 12
- Open: 35
- Open P0: 15
- Open P1: 19
- Open P2: 1
- Total estimate points: 206
- Open estimate points: 162

## Immediate execution order

1. **Truth-repair the current run lane** — `FW-013`, `FW-015`, `FW-017`, `FW-019`, `FW-020`
2. **Harden setup/provider routing** — `FW-014`, `FW-047`
3. **Land the real candidate lane** — `FW-023` through `FW-029`
4. **Land paired verification and gating** — `FW-030` through `FW-034`
5. **Land memory, audit, and apply** — `FW-035` through `FW-041`
6. **Harden the release surface** — `FW-042` through `FW-046`

## Recommended use

1. Replace the repository `AGENTS.md` with the version in this pack.
2. Read `docs/16_repo_state_audit.md` before making more UI polish decisions.
3. Import `master_issue_matrix.csv` into the tracker.
4. Use `plans/forge-workbench-v1-perfection.execplan.md` as the live execution plan.
5. Start Codex with `prompts/codex_master_prompt.md`.
