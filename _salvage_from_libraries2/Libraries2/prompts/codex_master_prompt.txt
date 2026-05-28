You are working in an existing Rust/Tauri/React repository for Forge Workbench v1.

Your job is to finish the product from its **current repository state** using the attached perfection pack as the controlling implementation guide.

Read these files first, in this order:

1. AGENTS.md
2. docs/16_repo_state_audit.md
3. plans/forge-workbench-v1-perfection.execplan.md
4. docs/18_truthful_phase_model.md
5. docs/17_run_failure_diagnostics_and_repo_preflight.md
6. docs/12_provider_setup_and_model_selection.md
7. docs/13_settings_persistence_and_secret_handling.md
8. docs/14_master_issue_matrix.md
9. docs/15_completion_acceptance_test_plan.md
10. master_issue_matrix.csv

Mission:
Complete Forge Workbench v1 as a local-first desktop app whose first shipped capability is a Verified Rust Repair Agent.

Current repo reality to assume:
- setup/provider/model configuration already exists
- settings persistence already exists
- keyring-first secret storage already exists
- run detail routing already exists
- cancel/retry already exist
- baseline capture already exists
- the current failure is still in the repo/baseline lane
- candidate generation, verification, memory, audit, apply, and release hardening are still incomplete

Non-negotiable rules:
- preserve the authority map
- do not move provider/model routing into React
- keep Tauri commands thin
- keep business logic in forge-workbench-core
- do not store plaintext secrets in SQLite
- do not log secrets
- do not write to the live working tree without explicit approval
- do not mark a run completed unless the repair lane is actually complete
- do not over-claim unwired capabilities in the UI

Required execution order:
1. Milestone 0: truth repair and intake hardening
   - FW-013
   - FW-014
   - FW-015
   - FW-016
   - FW-017
   - FW-018
   - FW-019
   - FW-020
   - FW-021
   - FW-022
   - FW-047
2. Milestone 1: candidate generation and review
   - FW-023 through FW-029
3. Milestone 2: verification and decisioning
   - FW-030 through FW-034
4. Milestone 3: memory and audit
   - FW-035 through FW-038
5. Milestone 4: apply, recovery, and security
   - FW-039 through FW-043
6. Milestone 5: release hardening
   - FW-044 through FW-046

Your first response must contain:
1. a repo-vs-pack gap audit
2. the exact milestone and issue IDs you will execute first
3. the files you expect to touch first
4. the tests you will add or update first
5. any plan adjustments required before coding

Then start implementing.

Required deliverables in code:
- strict setup gating that requires successful provider validation
- provider/model resolution that cannot silently cross provider boundaries
- repo preflight before queueing
- folder picker or equivalent repo chooser UX
- failed-run diagnostics rendered in Run detail
- truthful phase/timeline semantics
- baseline-only success no longer labeled completed
- candidate persistence and candidate generation
- paired verification and explanation
- memory import/retrieval and audit surfaces
- explicit approval and final apply
- restart-safe recovery
- security regression coverage
- release checklist and smoke path

Do not give me a theoretical essay.
Inspect the repo, compare it to the pack, update the plan if needed, then execute the work.
