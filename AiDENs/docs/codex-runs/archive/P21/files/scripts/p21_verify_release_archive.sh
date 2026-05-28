#!/usr/bin/env bash
set -euo pipefail
ZIP_PATH="${1:-target/p21/aidens-v0.1-candidate.zip}"
CALLER_ROOT="$(pwd)"
REPORT_OUT="${P21_ARCHIVE_REPORT_OUT:-target/p21/archive_verifier_report.json}"
if [[ "$REPORT_OUT" != /* ]]; then
  REPORT_OUT="$CALLER_ROOT/$REPORT_OUT"
fi
mkdir -p target/p21
if [[ ! -f "$ZIP_PATH" ]]; then
  echo "creating release archive at $ZIP_PATH"
  mkdir -p "$(dirname "$ZIP_PATH")"
  if [[ -f zip.py ]]; then
    python3 zip.py --output "$ZIP_PATH" --root .
  else
    zip -qr "$ZIP_PATH" . -x 'target/*' '.git/*' '*.zip' '__pycache__/*'
  fi
fi
ZIP_ABS="$(python3 - "$ZIP_PATH" <<'PY'
import pathlib
import sys
print(pathlib.Path(sys.argv[1]).resolve())
PY
)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
unzip -q "$ZIP_ABS" -d "$TMP"
# Find archive root. If zip has files at top-level, use temp directly.
ARCHIVE_ROOT="$TMP"
if [[ ! -f "$ARCHIVE_ROOT/Cargo.toml" ]]; then
  candidate="$(find "$TMP" -maxdepth 2 -name Cargo.toml -print -quit || true)"
  if [[ -n "$candidate" ]]; then
    ARCHIVE_ROOT="$(dirname "$candidate")"
  fi
fi
cd "$ARCHIVE_ROOT"

required_paths=(
  Cargo.toml
  Cargo.lock
  rust-toolchain.toml
  scripts
  scripts/p21_verify.sh
  scripts/p21_scan_package_integrity.py
  scripts/p21_scan_source_cross_refs.py
  scripts/p21_verify_release_archive.sh
  evals
  evals/p20_agency_eval_cases.jsonl
  evals/p21_agency_eval_cases.jsonl
  fixtures
  fixtures/test-agent/basic-agent.toml
  fixtures/runner/expected_test_agent_event_log.ndjson
  tests/fixtures
  crates/aidens-cli
  crates/aidens-integration-tests
  crates/aidens-integration-tests/tests
  docs/p21
  docs/p21/P21_SCOPE.md
  docs/p21/P21_ACCEPTANCE_GATES.md
  docs/p21/RECALL_CODING_EXTRACTION_REPORT.md
  docs/p21/RECALL_DAEMON_EXTRACTION_REPORT.md
  examples
  examples/configs/coding-agent.toml
  examples/configs/daemon-safe.toml
  handoffs/p21
  handoffs/p21/PHASE_00_REPORT.md
  handoffs/p21/PHASE_01_BUILD_CERTIFICATION.md
  handoffs/p21/PHASE_02_REPORT.md
  handoffs/p21/PHASE_03_REPORT.md
  handoffs/p21/PHASE_04_REPORT.md
  handoffs/p21/PHASE_05_REPORT.md
  handoffs/p21/PHASE_06_REPORT.md
  handoffs/p21/PHASE_07_REPORT.md
  handoffs/p21/PHASE_08_REPORT.md
  handoffs/p21/PHASE_09_REPORT.md
  handoffs/p21/PHASE_10_REPORT.md
  handoffs/p21/FINAL_AUDIT_REPORT.md
  handoffs/p21/KNOWN_LIMITATIONS.md
  audit/p21
  audit/p21/P21_SOURCE_BASIS_AND_CODE_FIRST_AUDIT.md
  scripts/p21_daemon_smoke.sh
  examples/daemon-safe/README.md
  crates/aidens-integration-tests/tests/phase_09_daemon_smoke.rs
)

required_file="$TMP/required_paths.txt"
missing_file="$TMP/missing_paths.txt"
printf '%s\n' "${required_paths[@]}" > "$required_file"
: > "$missing_file"

for p in "${required_paths[@]}"; do
  if [[ ! -e "$p" ]]; then
    echo "missing in release archive: $p"
    printf '%s\n' "$p" >> "$missing_file"
  fi
done
missing_count="$(wc -l < "$missing_file" | tr -d ' ')"
required_count="${#required_paths[@]}"

package_status=0
package_log="$TMP/package_integrity.log"
if [[ -f scripts/p21_scan_package_integrity.py ]]; then
  if python3 scripts/p21_scan_package_integrity.py . > "$package_log" 2>&1; then
    package_status=0
  else
    package_status=$?
  fi
  cat "$package_log"
else
  package_status=1
  echo "missing p21 scanner in release archive" | tee "$package_log"
fi

p21_verify_status=0
p21_verify_log="$TMP/p21_verify.log"
if [[ "$missing_count" == "0" && -f scripts/p21_verify.sh ]]; then
  if bash scripts/p21_verify.sh > "$p21_verify_log" 2>&1; then
    p21_verify_status=0
  else
    p21_verify_status=$?
  fi
  cat "$p21_verify_log"
else
  p21_verify_status=1
  echo "skipping p21_verify.sh because required archive paths are missing" | tee "$p21_verify_log"
fi

mkdir -p "$(dirname "$REPORT_OUT")"
python3 - "$REPORT_OUT" "$ZIP_ABS" "$ARCHIVE_ROOT" "$required_file" "$missing_file" "$package_status" "$p21_verify_status" <<'PY'
import json
import pathlib
import sys

report_out = pathlib.Path(sys.argv[1])
zip_path = pathlib.Path(sys.argv[2])
archive_root = pathlib.Path(sys.argv[3])
required = pathlib.Path(sys.argv[4]).read_text().splitlines()
missing = pathlib.Path(sys.argv[5]).read_text().splitlines()
package_status = int(sys.argv[6])
p21_verify_status = int(sys.argv[7])
report = {
    "archive_root": str(archive_root),
    "missing_file_count": len(missing),
    "missing_paths": missing,
    "ok": not missing and package_status == 0 and p21_verify_status == 0,
    "package_integrity_status": package_status,
    "p21_verify_status": p21_verify_status,
    "required_path_count": len(required),
    "required_paths": required,
    "zip_path": str(zip_path),
}
report_out.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
print(f"archive_verifier_report={report_out}")
print(f"missing_file_count={len(missing)}")
PY

if [[ "$missing_count" != "0" || "$package_status" != "0" || "$p21_verify_status" != "0" ]]; then
  exit 1
fi
echo "release archive replay verified: $ZIP_ABS"
