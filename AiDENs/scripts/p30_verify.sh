#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-.}"
cd "$ROOT"

printf '[p30] repo=%s\n' "$(pwd)"

if [ ! -f Cargo.toml ] && [ -f AiDENs/Cargo.toml ]; then
  printf '[p30] detected archive root; switching static guard repo to AiDENs\n'
  GUARD_REPO="AiDENs"
else
  GUARD_REPO="."
fi

python3 "$GUARD_REPO/scripts/p30_guard.py" --repo "$GUARD_REPO"

# Required P30 docs/scripts once installed into repo.
for p in \
  "$GUARD_REPO/P30_CODEX_SUPER_PASS_PROMPT.md" \
  "$GUARD_REPO/P30_PHASE_PLAN.md" \
  "$GUARD_REPO/P30_ACCEPTANCE_GATES.md" \
  "$GUARD_REPO/matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv"; do
  if [ ! -e "$p" ]; then
    echo "[p30] missing required artifact: $p" >&2
    exit 1
  fi
done

if command -v python3 >/dev/null 2>&1; then
  python3 "$GUARD_REPO/z.py" --root "$GUARD_REPO" --profile aidens --mode codex-context --no-strict --archive-root-markdown-noise --archive-only --include-root-markdown-archive
  python3 "$GUARD_REPO/scripts/assert_root_markdown_archive_manifest.py"
fi

# If cargo exists, run metadata/check as smoke. Full command bar remains separate because it can be expensive.
if command -v cargo >/dev/null 2>&1; then
  if [ -f AiDENs/Cargo.toml ]; then
    cargo metadata --manifest-path AiDENs/Cargo.toml --locked --format-version 1 >/tmp/p30_cargo_metadata.json
  else
    cargo metadata --locked --format-version 1 >/tmp/p30_cargo_metadata.json
  fi
  echo "[p30] cargo metadata OK"
else
  echo "[p30] cargo not found; build-certification blocker must be recorded"
fi

echo "[p30] static verification completed"
