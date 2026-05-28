#!/usr/bin/env bash
set -euo pipefail
ZIP_PATH="${1:-target/p22/aidens-p22-release-context.zip}"
mkdir -p target/p22/audit

python3 z.py --root . --profile aidens --mode codex-context --strict --output "$ZIP_PATH" | tee target/p22/audit/zpy_release_package.log

MANIFEST_PATH="${ZIP_PATH%.zip}.manifest.json"
if [[ -f "$MANIFEST_PATH" ]]; then
  python3 scripts/assert_p22_release_package_clean.py "$MANIFEST_PATH" | tee target/p22/audit/assert_release_manifest_clean.log
fi
python3 scripts/assert_p22_release_package_clean.py "$ZIP_PATH" | tee target/p22/audit/assert_release_zip_clean.log

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
unzip -q "$ZIP_PATH" -d "$TMP"
ARCHIVE_ROOT="$TMP"
if [[ -f "$TMP/AiDENs/Cargo.toml" ]]; then
  ARCHIVE_ROOT="$TMP/AiDENs"
elif [[ ! -f "$ARCHIVE_ROOT/Cargo.toml" ]]; then
  candidate="$(find "$TMP" -maxdepth 3 -path '*/AiDENs/Cargo.toml' -print -quit || true)"
  if [[ -z "$candidate" ]]; then
    candidate="$(find "$TMP" -maxdepth 3 -name Cargo.toml -print -quit || true)"
  fi
  if [[ -n "$candidate" ]]; then
    ARCHIVE_ROOT="$(dirname "$candidate")"
  fi
fi

pushd "$ARCHIVE_ROOT" >/dev/null
python3 scripts/assert_p22_codex_archival_hygiene.py . --current-run P22 | tee "$OLDPWD/target/p22/audit/assert_unzipped_hygiene.log"
if [[ "${P22_REPLAY_REQUIRE_CARGO:-0}" == "1" ]]; then
  cargo check --workspace --all-targets --all-features | tee "$OLDPWD/target/p22/audit/replay_cargo_check.log"
fi
popd >/dev/null

python3 - "$ZIP_PATH" "$ARCHIVE_ROOT" > target/p22/archive_verifier_report.final.json <<'PYREPORT'
import hashlib, json, pathlib, sys
zip_path = pathlib.Path(sys.argv[1])
archive_root = pathlib.Path(sys.argv[2])
report = {
    "ok": True,
    "zip_path": str(zip_path.resolve()),
    "sha256": hashlib.sha256(zip_path.read_bytes()).hexdigest(),
    "archive_root": str(archive_root),
    "normal_package_excludes_codex_archive": True,
}
print(json.dumps(report, indent=2, sort_keys=True))
PYREPORT
cat target/p22/archive_verifier_report.final.json
