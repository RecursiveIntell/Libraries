# Scripts

Copy or run these scripts from the AiDENs repo root. Set `AIDENS_REPO_ROOT` when running from elsewhere.

- `run_codex_phases.sh`: phase prompt/gate driver.
- `assert_stack_paths.sh`: rejects `Libraries2` stack-ids paths, overlays, and scaffolds while checking canonical sibling paths.
- `assert_no_shadow_truth.sh`: rejects public local canonical types outside allowed paths.
- `assert_docs_match_cargo.sh`: catches stale docs that contradict Cargo dependencies.
- `assert_adapter_delegation.sh`: heuristic check that adapters still reference canonical crates.
- `assert_compat_is_finite.sh`: checks compatibility ledger discipline.
