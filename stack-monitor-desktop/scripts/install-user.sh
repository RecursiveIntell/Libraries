#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
BIN_DIR="${HOME}/.local/bin"
SERVICE_DIR="${XDG_CONFIG_HOME:-${HOME}/.config}/systemd/user"
APPLY=0
ACTIVATE=0

for arg in "$@"; do
  case "$arg" in
    --apply) APPLY=1 ;;
    --activate) APPLY=1; ACTIVATE=1 ;;
    --help)
      printf '%s\n' 'Usage: install-user.sh [--apply] [--activate]'
      printf '%s\n' 'Default: dry-run only. --apply installs files; --activate also enables/restarts the user service.'
      exit 0
      ;;
    *) printf 'unknown argument: %s\n' "$arg" >&2; exit 2 ;;
  esac
done

COLLECTOR="${ROOT}/target/release/stack-monitor-collector"
DESKTOP="${ROOT}/target/release/stack-monitor-desktop"
SERVICE="${ROOT}/stack-monitor-desktop/packaging/stack-monitor-collector.service"
for artifact in "$COLLECTOR" "$DESKTOP" "$SERVICE"; do
  test -e "$artifact" || { printf 'missing release artifact: %s\n' "$artifact" >&2; exit 1; }
done

printf 'collector → %s/stack-monitor-collector\n' "$BIN_DIR"
printf 'desktop   → %s/stack-monitor-desktop\n' "$BIN_DIR"
printf 'service   → %s/stack-monitor-collector.service\n' "$SERVICE_DIR"

if (( ! APPLY )); then
  printf '%s\n' 'DRY_RUN: no files copied and no service state changed.'
  exit 0
fi

install -Dm755 "$COLLECTOR" "${BIN_DIR}/stack-monitor-collector"
install -Dm755 "$DESKTOP" "${BIN_DIR}/stack-monitor-desktop"
install -Dm644 "$SERVICE" "${SERVICE_DIR}/stack-monitor-collector.service"
systemctl --user daemon-reload

if (( ACTIVATE )); then
  systemctl --user enable --now stack-monitor-collector.service
  printf '%s\n' 'ACTIVATED: stack-monitor-collector.service'
else
  printf '%s\n' 'INSTALLED: service not enabled or started; use --activate explicitly.'
fi
