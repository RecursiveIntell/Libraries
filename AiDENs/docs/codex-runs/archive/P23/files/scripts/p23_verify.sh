#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-.}"
cd "$ROOT"
mkdir -p target/p23/audit

python3 scripts/assert_no_legacy_zip.py . | tee target/p23/audit/assert_no_legacy_zip.log
python3 scripts/assert_script_refs_strict.py . | tee target/p23/audit/assert_script_refs_strict.log
python3 scripts/assert_codex_artifact_classification.py . | tee target/p23/audit/assert_codex_artifact_classification.log
python3 scripts/assert_zpy_total_contract.py z.py | tee target/p23/audit/assert_zpy_total_contract.log
python3 scripts/assert_aidens_capability_contract.py . | tee target/p23/audit/assert_aidens_capability_contract.log

python3 z.py --root . --profile aidens --mode codex-context --codex-current-run P23 --strict --dry-run \
  --check-script-refs \
  --output target/p23/audit/p23_codex_context.zip \
  --manifest-out target/p23/audit/p23_codex_context.manifest.json \
  --report-out target/p23/audit/p23_codex_context.report.md \
  --excluded-out target/p23/audit/p23_codex_context.excluded.json \
  --findings-out target/p23/audit/p23_codex_context.findings.json \
  --codex-archive-report-out target/p23/audit/p23_codex_context.codex-archive.json \
  | tee target/p23/audit/zpy_codex_context_dry_run.log

if python3 z.py --help | grep -q "release-context"; then
  python3 z.py --root . --profile aidens --mode release-context --codex-current-run P23 --strict --dry-run \
    --check-script-refs \
    --output target/p23/audit/p23_release_context.zip \
    --manifest-out target/p23/audit/p23_release_context.manifest.json \
    --report-out target/p23/audit/p23_release_context.report.md \
    --excluded-out target/p23/audit/p23_release_context.excluded.json \
    --findings-out target/p23/audit/p23_release_context.findings.json \
    --codex-archive-report-out target/p23/audit/p23_release_context.codex-archive.json \
    | tee target/p23/audit/zpy_release_context_dry_run.log
fi

if [[ "${P23_REQUIRE_CARGO:-0}" == "1" ]]; then
  cargo fmt --all --check | tee target/p23/audit/cargo_fmt_check.log
  cargo check --workspace --all-targets --all-features | tee target/p23/audit/cargo_check_workspace_all_targets_all_features.log
  cargo test --workspace --all-targets --all-features | tee target/p23/audit/cargo_test_workspace_all_targets_all_features.log
  cargo clippy --workspace --all-targets --all-features -- -D warnings | tee target/p23/audit/cargo_clippy_workspace_all_targets_all_features.log
fi

echo "P23 verify completed" | tee target/p23/audit/p23_verify.done
