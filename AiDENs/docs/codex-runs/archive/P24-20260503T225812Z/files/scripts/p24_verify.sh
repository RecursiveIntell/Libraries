#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

OUT_DIR="target/p24-verifier"
CHECKS_JSONL="$OUT_DIR/checks.jsonl"
RECEIPT="$OUT_DIR/p24_verifier_receipt.json"
mkdir -p "$OUT_DIR"
: >"$CHECKS_JSONL"

STARTED_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
FAILED=0

record_check() {
  local id="$1"
  local timeout_seconds="$2"
  local exit_code="$3"
  local status="$4"
  local stdout_path="$5"
  local stderr_path="$6"
  shift 6
  python3 - "$CHECKS_JSONL" "$id" "$timeout_seconds" "$exit_code" "$status" "$stdout_path" "$stderr_path" "$@" <<'PY'
import json
import sys

out, check_id, timeout_seconds, exit_code, status, stdout_path, stderr_path, *argv = sys.argv[1:]
record = {
    "id": check_id,
    "argv": argv,
    "timeout_seconds": int(timeout_seconds),
    "exit_code": int(exit_code),
    "status": status,
    "stdout": stdout_path,
    "stderr": stderr_path,
}
with open(out, "a", encoding="utf-8") as fh:
    fh.write(json.dumps(record, sort_keys=True) + "\n")
PY
}

run_check() {
  local id="$1"
  local timeout_seconds="$2"
  shift 2
  local stdout_path="$OUT_DIR/${id}.stdout"
  local stderr_path="$OUT_DIR/${id}.stderr"
  set +e
  timeout "$timeout_seconds" "$@" >"$stdout_path" 2>"$stderr_path"
  local exit_code=$?
  set -e
  local status="fail"
  if [[ "$exit_code" -eq 0 ]]; then
    status="pass"
  elif [[ "$exit_code" -eq 124 ]]; then
    status="timeout"
  else
    status="fail"
  fi
  record_check "$id" "$timeout_seconds" "$exit_code" "$status" "$stdout_path" "$stderr_path" "$@"
  if [[ "$status" != "pass" ]]; then
    FAILED=1
  fi
}

record_skipped() {
  local id="$1"
  local reason="$2"
  local stdout_path="$OUT_DIR/${id}.stdout"
  local stderr_path="$OUT_DIR/${id}.stderr"
  printf '%s\n' "$reason" >"$stdout_path"
  : >"$stderr_path"
  record_check "$id" 0 0 skipped "$stdout_path" "$stderr_path" "skipped:$reason"
}

run_check no_legacy_zip 30 python3 scripts/assert_no_legacy_zip.py .
run_check script_refs_resolve 45 python3 scripts/assert_script_refs_strict.py .
run_check codex_artifact_classification 60 python3 scripts/assert_codex_artifact_classification.py .
run_check no_local_canonical_type_substitutes 60 python3 scripts/assert_no_canonical_type_duplicates.py
run_check no_local_canonical_digest_law 30 bash scripts/assert_no_local_canonical_digest_law.sh
run_check no_fake_completion_claims 30 bash scripts/assert_no_fake_completion.sh .
run_check no_scaffold_promoted 120 bash scripts/assert_no_scaffold_promoted.sh .
run_check docs_match_cargo 30 bash scripts/assert_docs_match_cargo.sh .
run_check schema_generation_scope 30 python3 scripts/assert_schema_generation_scope.py
run_check wrapper_backpointers 30 bash scripts/assert_wrapper_backpointers.sh
run_check run_bundle_schema_generation 180 bash -lc 'cargo run -q -p aidens-cli -- schemas generate --out target/p24-verifier/schemas >/tmp/p24_schema_generate.out && test -f target/p24-verifier/schemas/aidens-run-bundle/v2.schema.json'
run_check run_bundle_fixture 180 bash -lc 'cargo run -q -p aidens-cli -- run-test-agent fixtures/test-agent/basic-agent.toml --out target/p24-verifier/run-test-agent >/tmp/p24_run_test_agent.out && cargo run -q -p aidens-cli -- inspect-run target/p24-verifier/run-test-agent/run-bundle.json >/tmp/p24_inspect_run.out'
run_check coding_agent_fixture 180 bash -lc 'cargo run -q -p aidens-cli -- run-coding-agent examples/configs/coding-agent.toml --out target/p24-verifier/coding-agent >/tmp/p24_run_coding_agent.out && cargo run -q -p aidens-cli -- inspect-run target/p24-verifier/coding-agent/run-bundle.json >/tmp/p24_inspect_coding.out'
run_check memory_runtime_fixture 240 bash -lc 'cargo run -q -p aidens-cli -- memory seam-fixture --out target/p24-verifier/memory-seam >/tmp/p24_memory_seam.out'
run_check package_dry_run 180 python3 z.py --root . --profile aidens --mode codex-context --codex-current-run P24 --strict --dry-run --check-script-refs --output target/p24-verifier/p24_codex_context.zip --manifest-out target/p24-verifier/p24_codex_context.manifest.json --report-out target/p24-verifier/p24_codex_context.report.md --excluded-out target/p24-verifier/p24_codex_context.excluded.json --findings-out target/p24-verifier/p24_codex_context.findings.json --codex-archive-report-out target/p24-verifier/p24_codex_context.codex-archive.json

if [[ -n "${P24_PACKAGE_SELF_REPLAY:-}" ]]; then
  run_check package_self_replay 300 python3 scripts/assert_package_self_replay.py "$P24_PACKAGE_SELF_REPLAY" --verifier scripts/p24_verify.sh --require-verifier
else
  record_skipped package_self_replay "set P24_PACKAGE_SELF_REPLAY=/path/to/package.zip to replay a final package"
fi

ENDED_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
OVERALL="pass"
if [[ "$FAILED" -ne 0 ]]; then
  OVERALL="fail"
fi

python3 - "$CHECKS_JSONL" "$RECEIPT" "$PWD" "$STARTED_UTC" "$ENDED_UTC" "$OVERALL" <<'PY'
import json
import sys

checks_path, receipt_path, root, started, ended, overall = sys.argv[1:]
checks = []
with open(checks_path, encoding="utf-8") as fh:
    for line in fh:
        if line.strip():
            checks.append(json.loads(line))
receipt = {
    "schema": "P24VerifierReceiptV1",
    "repo_root": root,
    "started_utc": started,
    "ended_utc": ended,
    "checks": checks,
    "overall": overall,
}
with open(receipt_path, "w", encoding="utf-8") as fh:
    json.dump(receipt, fh, indent=2, sort_keys=True)
    fh.write("\n")
print(json.dumps({"receipt": receipt_path, "overall": overall}, sort_keys=True))
PY

if [[ "$OVERALL" != "pass" ]]; then
  exit 1
fi
