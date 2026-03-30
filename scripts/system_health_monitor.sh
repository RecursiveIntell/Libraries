#!/usr/bin/env bash

set -euo pipefail

INTERVAL_SECONDS=15
LOG_FILE="${HOME}/.local/state/system-health-monitor/system-health.log"
MAX_BYTES=$((5 * 1024 * 1024))
KEEP_FILES=5
TOP_N=10
RUN_ONCE=0

KERNEL_PATTERN='rtw89|thermal|thrott|watchdog|nvme|amdgpu|pcie|alloc_pages_slowpath|kswapd|oom'

usage() {
  cat <<'EOF'
Usage: system_health_monitor.sh [options]

Options:
  --interval SECONDS   Sample interval. Default: 15
  --log-file PATH      Log file path. Default: ~/.local/state/system-health-monitor/system-health.log
  --max-bytes BYTES    Rotate when the active log reaches this size. Default: 5242880
  --keep COUNT         Number of rotated log archives to keep. Default: 5
  --top N              Number of processes to keep in the CPU snapshot. Default: 10
  --once               Capture one sample and exit
  --help               Show this help text

Notes:
  - Kernel log access may require sudo on some systems.
  - Rotation keeps PATH plus PATH.1 .. PATH.COUNT.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --interval)
      INTERVAL_SECONDS="$2"
      shift 2
      ;;
    --log-file)
      LOG_FILE="$2"
      shift 2
      ;;
    --max-bytes)
      MAX_BYTES="$2"
      shift 2
      ;;
    --keep)
      KEEP_FILES="$2"
      shift 2
      ;;
    --top)
      TOP_N="$2"
      shift 2
      ;;
    --once)
      RUN_ONCE=1
      shift
      ;;
    --help)
      usage
      exit 0
      ;;
    *)
      printf 'Unknown option: %s\n' "$1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

for value_name in INTERVAL_SECONDS MAX_BYTES KEEP_FILES TOP_N; do
  value="${!value_name}"
  if ! [[ "$value" =~ ^[0-9]+$ ]] || [[ "$value" -lt 1 ]]; then
    printf '%s must be a positive integer\n' "$value_name" >&2
    exit 1
  fi
done

LOG_DIR="$(dirname "$LOG_FILE")"
STATE_DIR="${LOG_DIR}/.state"
LOCK_FILE="${STATE_DIR}/monitor.lock"
JOURNAL_STATE_FILE="${STATE_DIR}/journal-since-epoch"
JOURNAL_PROBE_FILE="${STATE_DIR}/journal-probe"

mkdir -p "$LOG_DIR" "$STATE_DIR"
touch "$LOG_FILE"

if command -v flock >/dev/null 2>&1; then
  exec 9>"$LOCK_FILE"
  if ! flock -n 9; then
    printf 'Another monitor instance is already using %s\n' "$LOG_FILE" >&2
    exit 1
  fi
fi

rotate_logs() {
  local size

  size="$(stat -c '%s' "$LOG_FILE" 2>/dev/null || printf '0')"
  if [[ "$size" -lt "$MAX_BYTES" ]]; then
    return
  fi

  if [[ -f "${LOG_FILE}.${KEEP_FILES}" ]]; then
    rm -f "${LOG_FILE}.${KEEP_FILES}"
  fi

  if [[ "$KEEP_FILES" -gt 1 ]]; then
    local index
    for ((index = KEEP_FILES - 1; index >= 1; index--)); do
      if [[ -f "${LOG_FILE}.${index}" ]]; then
        mv "${LOG_FILE}.${index}" "${LOG_FILE}.$((index + 1))"
      fi
    done
  fi

  mv "$LOG_FILE" "${LOG_FILE}.1"
  : >"$LOG_FILE"
}

write_block() {
  local block="$1"
  local block_bytes
  local current_size

  block_bytes="$(printf '%s' "$block" | wc -c | tr -d ' ')"
  current_size="$(stat -c '%s' "$LOG_FILE" 2>/dev/null || printf '0')"

  if [[ "$current_size" -gt 0 ]] && (( current_size + block_bytes > MAX_BYTES )); then
    rotate_logs
  fi

  printf '%s' "$block" >>"$LOG_FILE"

  if [[ -t 1 ]]; then
    printf '%s' "$block"
  fi
}

safe_cmd() {
  if "$@" 2>/dev/null; then
    return 0
  fi
  "$@" 2>&1 || true
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

journal_cmd() {
  if [[ -f "$JOURNAL_PROBE_FILE" ]]; then
    cat "$JOURNAL_PROBE_FILE"
    return
  fi

  if journalctl -k -b -n 1 --no-pager >/dev/null 2>&1; then
    printf 'journalctl' | tee "$JOURNAL_PROBE_FILE" >/dev/null
    return
  fi

  if sudo -n journalctl -k -b -n 1 --no-pager >/dev/null 2>&1; then
    printf 'sudo -n journalctl' | tee "$JOURNAL_PROBE_FILE" >/dev/null
    return
  fi

  printf 'unavailable' | tee "$JOURNAL_PROBE_FILE" >/dev/null
}

run_journal() {
  local mode
  mode="$(journal_cmd)"

  case "$mode" in
    journalctl)
      journalctl "$@"
      ;;
    "sudo -n journalctl")
      sudo -n journalctl "$@"
      ;;
    *)
      return 1
      ;;
  esac
}

collect_kernel_events() {
  local now
  local since
  local output

  now="$(date +%s)"

  if [[ -f "$JOURNAL_STATE_FILE" ]]; then
    since="$(cat "$JOURNAL_STATE_FILE")"
  else
    since="$((now - INTERVAL_SECONDS))"
  fi

  if ! [[ "$since" =~ ^[0-9]+$ ]]; then
    since="$((now - INTERVAL_SECONDS))"
  fi

  output="$(
    run_journal -k -b --since "@$since" --until "@$now" --no-pager -o short-iso 2>/dev/null |
      grep -Ei "$KERNEL_PATTERN" || true
  )"
  printf '%s\n' "$now" >"$JOURNAL_STATE_FILE"

  if [[ -z "$output" ]]; then
    if [[ "$(journal_cmd)" == "unavailable" ]]; then
      printf '(kernel log unavailable without journal access)\n'
    else
      printf '(none)\n'
    fi
    return
  fi

  printf '%s\n' "$output"
}

collect_temps() {
  if have_cmd sensors; then
    safe_cmd sensors | awk '
      /^(k10temp|amdgpu|nvme|acpitz|BAT0|hp-isa)/ { print; next }
      /^[[:space:]]*(Tctl:|edge:|Composite:|temp1:|power1:|fan1:|fan2:)/ { print }
    '
    return
  fi

  printf 'sensors not installed\n'
}

collect_rtw89_params() {
  local files=(
    /sys/module/rtw89_core/parameters/disable_ps_mode
    /sys/module/rtw89_pci/parameters/disable_clkreq
    /sys/module/rtw89_pci/parameters/disable_aspm_l1
    /sys/module/rtw89_pci/parameters/disable_aspm_l1ss
  )
  local file
  local name

  for file in "${files[@]}"; do
    name="$(basename "$file")"
    if [[ -r "$file" ]]; then
      printf '%s=%s\n' "$name" "$(cat "$file")"
    else
      printf '%s=unavailable\n' "$name"
    fi
  done
}

collect_snapshot() {
  local timestamp
  local block

  timestamp="$(date -Is)"
  block="$(
    cat <<EOF
=== $timestamp ===
uptime:
$(safe_cmd uptime)

memory:
$(safe_cmd free -h | awk 'NR == 1 || /^Mem:|^Swap:/')

disk:
$(safe_cmd df -h / | awk 'NR <= 2')

temperatures:
$(collect_temps)

rtw89:
$(collect_rtw89_params)

top_cpu:
$(safe_cmd ps -eo pid,comm,%cpu,%mem --sort=-%cpu | head -n "$((TOP_N + 1))")

kernel_events:
$(collect_kernel_events)

EOF
  )"

  write_block "$block"
}

while true; do
  collect_snapshot
  rotate_logs

  if [[ "$RUN_ONCE" -eq 1 ]]; then
    exit 0
  fi

  sleep "$INTERVAL_SECONDS"
done
