#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-.}"
cd "$ROOT"
mkdir -p target/p22/audit

python3 scripts/assert_p22_zpy_archive_contract.py z.py | tee target/p22/audit/assert_zpy_archive_contract.log
python3 scripts/assert_p22_codex_archival_hygiene.py . | tee target/p22/audit/assert_codex_archival_hygiene.log
python3 scripts/p22_zpy_archival_selftest.py | tee target/p22/audit/zpy_archival_selftest.log
python3 scripts/p22_secret_scan_fixture_test.py | tee target/p22/audit/p22_secret_scan_fixture_test.log

# Dry-run normal and audit-full packaging. Sidecars must still emit under dry-run,
# and verifier-generated sidecars should stay under target/p22/audit.
python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run \
  --output target/p22/audit/p22_verify_codex_context.zip \
  --manifest-out target/p22/audit/p22_verify_codex_context.manifest.json \
  --report-out target/p22/audit/p22_verify_codex_context.report.md \
  --excluded-out target/p22/audit/p22_verify_codex_context.excluded.json \
  --findings-out target/p22/audit/p22_verify_codex_context.findings.json \
  --codex-archive-report-out target/p22/audit/p22_verify_codex_context.codex-archive.json \
  | tee target/p22/audit/zpy_codex_context_dry_run.log
python3 scripts/assert_p22_release_package_clean.py --manifest target/p22/audit/p22_verify_codex_context.manifest.json \
  | tee target/p22/audit/assert_p22_verify_codex_context_clean.log

python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run \
  --output target/p22/audit/p22_verify_audit_full.zip \
  --manifest-out target/p22/audit/p22_verify_audit_full.manifest.json \
  --report-out target/p22/audit/p22_verify_audit_full.report.md \
  --excluded-out target/p22/audit/p22_verify_audit_full.excluded.json \
  --findings-out target/p22/audit/p22_verify_audit_full.findings.json \
  --codex-archive-report-out target/p22/audit/p22_verify_audit_full.codex-archive.json \
  | tee target/p22/audit/zpy_audit_full_dry_run.log

if [[ "${P22_REQUIRE_CARGO:-0}" == "1" ]]; then
  cargo fmt --all --check | tee target/p22/audit/cargo_fmt_check.log
  cargo check --workspace --all-targets --all-features | tee target/p22/audit/cargo_check_workspace_all_targets_all_features.log
  cargo test --workspace --all-targets --all-features | tee target/p22/audit/cargo_test_workspace_all_targets_all_features.log
  cargo clippy --workspace --all-targets --all-features -- -D warnings | tee target/p22/audit/cargo_clippy_workspace_all_targets_all_features.log
fi

echo "P22 verify completed" | tee target/p22/audit/p22_verify.done
