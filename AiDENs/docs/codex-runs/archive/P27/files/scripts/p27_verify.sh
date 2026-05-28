#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-.}"
cd "$ROOT"
mkdir -p target/p27/audit

log() { printf '[p27-verify] %s
' "$*"; }

log "static verifier surface"
python3 scripts/assert_p27_verifier_surface.py . 2>&1 | tee target/p27/audit/assert_p27_verifier_surface.log

log "strict script references"
python3 scripts/assert_script_refs_strict.py . 2>&1 | tee target/p27/audit/assert_script_refs_strict.log

log "current run truth"
python3 scripts/assert_p27_current_run_truth.py . 2>&1 | tee target/p27/audit/assert_p27_current_run_truth.log

log "AGENTS.md current"
python3 scripts/assert_p27_agents_md_current.py . 2>&1 | tee target/p27/audit/assert_p27_agents_md_current.log

if [[ -f scripts/assert_p27_ownership_scanner_fail_closed.py ]]; then
  log "ownership scanner fail-closed"
  python3 scripts/assert_p27_ownership_scanner_fail_closed.py . 2>&1 | tee target/p27/audit/assert_p27_ownership_scanner_fail_closed.log
fi

if [[ -f scripts/assert_p27_no_scaffold_profile_inflation.py ]]; then
  log "scaffold profile claim fence"
  python3 scripts/assert_p27_no_scaffold_profile_inflation.py . 2>&1 | tee target/p27/audit/assert_p27_no_scaffold_profile_inflation.log
fi

if [[ -f scripts/assert_p27_memory_no_local_truth.py ]]; then
  log "memory no-local-truth guard"
  python3 scripts/assert_p27_memory_no_local_truth.py . 2>&1 | tee target/p27/audit/assert_p27_memory_no_local_truth.log
fi

if [[ -f scripts/assert_p27_strict_structured_inputs.py ]]; then
  log "strict structured-input guard"
  python3 scripts/assert_p27_strict_structured_inputs.py . 2>&1 | tee target/p27/audit/assert_p27_strict_structured_inputs.log
fi

if [[ -f scripts/assert_p27_contracts_megafile_containment.py ]]; then
  log "contracts megafile containment"
  python3 scripts/assert_p27_contracts_megafile_containment.py . 2>&1 | tee target/p27/audit/assert_p27_contracts_megafile_containment.log
fi

if [[ -f scripts/assert_p27_cli_megafile_containment.py ]]; then
  log "CLI megafile containment"
  python3 scripts/assert_p27_cli_megafile_containment.py . 2>&1 | tee target/p27/audit/assert_p27_cli_megafile_containment.log
fi

if [[ -f scripts/assert_p27_agency_eval_harness.py ]]; then
  log "agency eval harness"
  python3 scripts/assert_p27_agency_eval_harness.py . 2>&1 | tee target/p27/audit/assert_p27_agency_eval_harness.log
fi

if [[ -f scripts/assert_p27_semantic_disclosure.py ]]; then
  log "11A semantic disclosure"
  python3 scripts/assert_p27_semantic_disclosure.py . 2>&1 | tee target/p27/audit/assert_p27_semantic_disclosure.log
fi

if [[ -f scripts/assert_p27_support_docs_traceable.py ]]; then
  log "support docs traceability"
  python3 scripts/assert_p27_support_docs_traceable.py . 2>&1 | tee target/p27/audit/assert_p27_support_docs_traceable.log
fi

if [[ -f scripts/assert_sibling_workspace_layout.py ]]; then
  log "sibling workspace layout"
  python3 scripts/assert_sibling_workspace_layout.py --root . --receipt-out target/p27/audit/assert_sibling_workspace_layout_receipt.json 2>&1 | tee target/p27/audit/assert_sibling_workspace_layout.log
fi

if [[ "${P27_SKIP_CARGO:-0}" == "1" ]]; then
  log "cargo checks skipped by P27_SKIP_CARGO=1"
elif command -v cargo >/dev/null 2>&1; then
  log "cargo fmt"
  cargo fmt --all -- --check 2>&1 | tee target/p27/audit/cargo_fmt.log
  log "cargo check"
  cargo check --workspace --all-targets 2>&1 | tee target/p27/audit/cargo_check.log
  log "cargo test"
  cargo test --workspace --all-targets 2>&1 | tee target/p27/audit/cargo_test.log
  if [[ -f scripts/p27_provider_path_smoke.py ]]; then
    log "provider path smoke"
    optional_ollama_args=()
    if [[ "${P27_ALLOW_OPTIONAL_OLLAMA:-0}" == "1" ]]; then
      optional_ollama_args+=(--allow-optional-ollama)
    fi
    python3 scripts/p27_provider_path_smoke.py . \
      --receipt-out target/p27/audit/assert_p27_provider_path_smoke_receipt.json \
      "${optional_ollama_args[@]}" \
      2>&1 | tee target/p27/audit/assert_p27_provider_path_smoke.log
  fi
  if [[ "${P27_FINAL_STRICT:-0}" == "1" ]]; then
    log "cargo clippy"
    cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee target/p27/audit/cargo_clippy.log
    log "cargo doc"
    cargo doc --workspace --no-deps 2>&1 | tee target/p27/audit/cargo_doc.log
  fi
else
  log "cargo unavailable"
  if [[ "${P27_REQUIRE_CARGO:-0}" == "1" ]]; then
    echo "cargo unavailable but P27_REQUIRE_CARGO=1" >&2
    exit 20
  fi
fi

log "p27 verifier complete"
