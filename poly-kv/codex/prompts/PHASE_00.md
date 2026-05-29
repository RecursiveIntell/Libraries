# PHASE 00 — Preflight and source inventory

1. Record git state, branch, commit, dirty files, and current `z.py` hash.
2. Run existing `z.py --help` and current package command in dry-run mode.
3. Inventory current `z.py` functions, CLI args, mode/profile defaults, and validators.
4. Create `.codex-runs/<run-id>/` and write:
   - `startup_preflight.md`
   - `source_inventory.md`
   - `commands_run.log`
   - `phase_00_report.md`

Do not edit files in this phase except run receipts.
