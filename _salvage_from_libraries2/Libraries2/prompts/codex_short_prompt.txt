Read AGENTS.md, docs/16_repo_state_audit.md, plans/forge-workbench-v1-perfection.execplan.md, docs/14_master_issue_matrix.md, docs/17_run_failure_diagnostics_and_repo_preflight.md, docs/18_truthful_phase_model.md, docs/12_provider_setup_and_model_selection.md, docs/13_settings_persistence_and_secret_handling.md, docs/15_completion_acceptance_test_plan.md, and master_issue_matrix.csv.

Then:
1. audit repo vs pack,
2. start with FW-013, FW-015, FW-017, FW-019, FW-020,
3. keep Tauri thin and core logic in forge-workbench-core,
4. do not leak secrets,
5. do not let baseline-only success say completed,
6. do not mark a milestone done without tests,
7. update the ExecPlan when crossing a major seam,
8. implement, verify, and report crisply.
