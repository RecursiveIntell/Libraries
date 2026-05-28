# 08 - CI and Commands

## Current P22 Gate

The current local release gate is:

```bash
P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh
```

With `P22_REQUIRE_CARGO=1`, `scripts/p22_verify.sh` runs:

- `python3 scripts/assert_p22_zpy_archive_contract.py z.py`
- `python3 scripts/assert_p22_codex_archival_hygiene.py .`
- `python3 scripts/p22_zpy_archival_selftest.py`
- strict `codex-context` dry-run packaging;
- package-clean assertion for the normal manifest;
- deliberate `audit-full` dry-run packaging;
- `cargo fmt --all --check`
- `cargo check --workspace --all-targets --all-features`
- `cargo test --workspace --all-targets --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

The compatibility command `bash scripts/verify.sh` delegates to `scripts/p22_verify.sh`. Historical P20/P21 verifier names are archive evidence only and are not the P22 final gate.

## Release Package Replay

Use this command to produce and replay-check a normal release context package:

```bash
bash scripts/p22_verify_release_archive.sh target/p22/aidens-p22-release-context.zip
```

The release package verifier checks the zip and manifest for archived/stale Codex history, unzips the package, reruns P22 archive hygiene on the extracted tree, and writes `target/p22/archive_verifier_report.final.json`.

## CI Workflow

The active GitHub workflow at `.github/workflows/ci.yml` runs:

```bash
P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh
```

Nextest or additional matrix jobs may be added later only if they preserve the same z.py archive contract, hygiene, package-clean, cargo, and secret-scanner fixture coverage.
