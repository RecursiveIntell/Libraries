# Acceptance Gates

## Phase gates

A phase may close only when:

- changed files are listed;
- commands run are recorded;
- tests/checks are passed or failures are explicitly bounded;
- source-of-truth boundaries are revalidated;
- no forbidden scope was added.

## Final gate

Required final files in `.codex-runs/<run-id>/`:

```text
startup_preflight.md
source_inventory.md
phase_00_report.md
phase_01_report.md
phase_02_report.md
phase_03_report.md
phase_04_report.md
phase_05_report.md
phase_06_report.md
phase_07_report.md
phase_08_report.md
phase_09_report.md
commands_run.log
changed_files.txt
validation_results.md
invariant_report.md
risk_register.md
rollback_plan.md
final_audit_report.md
remaining_delta.md
```

## Required commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
python3 scripts/validate_schemas.py
python3 scripts/check_public_claims.py
python3 scripts/validate_final_state.py
```

If a command is skipped, the final report must say why and whether it blocks release.

## Release-readiness definition

Alpha release is allowed only when:

- all normal commands pass;
- crate name availability is manually verified;
- README claim boundary is safe;
- no adapter claims are made without tests;
- no app integrations are included;
- exact fallback and receipt tests pass;
- hostile-auditor handoff exists.
