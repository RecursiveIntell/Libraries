#!/usr/bin/env bash
set -euo pipefail
ROOT=${1:-.}
STATUS_FILE="$ROOT/STATUS.md"
FAIL=0

SCAFFOLD_ONLY_CRATES=(
  aidens-profile-daemon
  aidens-profile-desktop
  aidens-profile-memory
  aidens-profile-research
)

is_scaffold_only_crate() {
  local needle="$1"
  local crate
  for crate in "${SCAFFOLD_ONLY_CRATES[@]}"; do
    if [ "$crate" = "$needle" ]; then
      return 0
    fi
  done
  return 1
}

record_failure() {
  echo "$1" >&2
  FAIL=1
}

if [ ! -f "$STATUS_FILE" ]; then
  record_failure "Missing STATUS.md source-basis status file."
else
  for crate_dir in "$ROOT"/crates/*; do
    [ -d "$crate_dir" ] || continue
    crate="$(basename "$crate_dir")"
    if ! grep -Fq "| \`$crate\` |" "$STATUS_FILE"; then
      record_failure "STATUS.md does not list crate: $crate"
      continue
    fi
    if ! grep -Eq "^\| \`$crate\` \| (implemented|partial|scaffold-only) \|" "$STATUS_FILE"; then
      record_failure "STATUS.md uses an invalid status for crate: $crate"
    fi
  done

  for crate in "${SCAFFOLD_ONLY_CRATES[@]}"; do
    if ! grep -Fq "| \`$crate\` | scaffold-only |" "$STATUS_FILE"; then
      record_failure "STATUS.md must mark $crate as scaffold-only."
    fi
  done
fi

MARKER_FILES="$(grep -RIl --include '*.rs' -E 'Scaffolded for future AiDENs implementation|scaffolded; implement according to AiDENs docs' "$ROOT"/crates 2>/dev/null || true)"
while IFS= read -r file; do
  [ -n "$file" ] || continue
  rel="${file#"$ROOT"/crates/}"
  crate="${rel%%/*}"
  if ! is_scaffold_only_crate "$crate"; then
    record_failure "Scaffold marker appears outside the deferred crate list: $file"
  fi
done <<< "$MARKER_FILES"

PROMOTION_PATTERN='not implemented but healthy|scaffold.*healthy|deferred.*healthy|scaffold.*ready|deferred.*ready|scaffold.*production|deferred.*production'
for doc in "$ROOT"/README.md "$STATUS_FILE"; do
  [ -f "$doc" ] || continue
  if grep -InE "$PROMOTION_PATTERN" "$doc" 2>/dev/null; then
    record_failure "README/STATUS promotes scaffold or deferred surfaces."
  fi
done

if command -v cargo >/dev/null 2>&1 && [ -f "$ROOT/examples/aidens.mock.toml" ]; then
  DOCTOR_OUTPUT="$(cd "$ROOT" && cargo run -q -p aidens-cli -- doctor --config examples/aidens.mock.toml)"
  for crate in "${SCAFFOLD_ONLY_CRATES[@]}"; do
    block="$(printf '%s\n' "$DOCTOR_OUTPUT" | grep -A12 -F "\"capability_id\": \"crate:$crate\"" || true)"
    if [ -z "$block" ]; then
      record_failure "Doctor report does not list scaffold crate: $crate"
      continue
    fi
    if ! printf '%s\n' "$block" | grep -Eq '"(disabled|deferred|blocked-by-policy)"'; then
      record_failure "Doctor report does not mark $crate as deferred/blocked."
    fi
    if printf '%s\n' "$block" | grep -F '"healthy"' >/dev/null 2>&1; then
      record_failure "Doctor report marks scaffold crate healthy: $crate"
    fi
  done
else
  echo "cargo not available; skipping doctor scaffold-state check" >&2
fi

if [ "$FAIL" -ne 0 ]; then
  echo "Scaffold/deferred surface promotion check failed." >&2
  exit 1
fi

echo "No scaffold promotion patterns found."
