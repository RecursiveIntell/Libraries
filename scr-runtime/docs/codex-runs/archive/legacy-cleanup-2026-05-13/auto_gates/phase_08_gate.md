# Auto Gate Phase 8

Before proceeding past Phase 8, Codex must:

1. Re-read AGENTS.md source-of-truth and no-invention rules.
2. Run the phase-specific acceptance commands from the phase prompt.
3. Record pass/fail/skipped status in docs/P31_COMMAND_RECEIPTS.md.
4. If a command fails, repair or stop and write the blocker in docs/P31_UNRESOLVED_RISKS.md.
5. Check for source-of-truth drift:

```bash
rg -n "ClaimLedger|PASS_CL|manual injection|paste the matching manual|no local Cargo.toml|does not yet contain a Rust package|testtmp|target_files" . || true
```

Any active non-historical hit must be fixed before continuing.

6. Check existing-crate boundary drift:

```bash
python3 scripts/assert_existing_crate_boundaries.py || true
```

If the script exists and fails, repair before continuing. If it does not exist yet and the current phase is >= 5, stop and create it.
