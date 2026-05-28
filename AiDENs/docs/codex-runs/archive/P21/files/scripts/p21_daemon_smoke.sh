#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-target/p21/phase09/daemon-smoke}"
ROOT="${2:-$OUT_DIR/queue-root}"
NAME="${P21_DAEMON_SMOKE_NAME:-p21-stretch-smoke}"
OWNER="${P21_DAEMON_SMOKE_OWNER:-daemon-a}"

mkdir -p "$OUT_DIR"
rm -rf "$ROOT"
mkdir -p "$ROOT"

run_queue() {
  local log_name="$1"
  shift
  cargo run --quiet -p aidens-cli -- queue "$@" > "$OUT_DIR/$log_name.json"
}

run_queue namespace namespace --root "$ROOT" --name "$NAME" --owner "$OWNER"
run_queue schedule_first \
  schedule \
  --root "$ROOT" \
  --name "$NAME" \
  --owner "$OWNER" \
  --schedule-id once \
  --occurrence-key p21-stretch-once \
  --due-at 2026-05-01T00:00:00Z \
  --payload '{"task":"read-only-refresh"}' \
  --risk read-only
run_queue schedule_duplicate \
  schedule \
  --root "$ROOT" \
  --name "$NAME" \
  --owner "$OWNER" \
  --schedule-id once \
  --occurrence-key p21-stretch-once \
  --due-at 2026-05-01T00:00:00Z \
  --payload '{"task":"read-only-refresh"}' \
  --risk read-only
run_queue lease \
  lease \
  --root "$ROOT" \
  --name "$NAME" \
  --owner "$OWNER" \
  --ttl-seconds 60
run_queue safe_mode \
  safe-mode \
  --root "$ROOT" \
  --name "$NAME" \
  --owner "$OWNER" \
  --enabled \
  --reason p21-stretch-safe-mode
run_queue risky_wake_blocked \
  wake \
  --root "$ROOT" \
  --name "$NAME" \
  --owner "$OWNER" \
  --source filesystem \
  --signal-key risky-shell \
  --payload '{"cmd":"cargo test"}' \
  --risk shell
run_queue read_only_wake \
  wake \
  --root "$ROOT" \
  --name "$NAME" \
  --owner "$OWNER" \
  --source filesystem \
  --signal-key inspect-readme \
  --payload '{"path":"README.md"}' \
  --risk read-only
run_queue drain \
  drain \
  --root "$ROOT" \
  --name "$NAME" \
  --owner "$OWNER" \
  --reason p21-stretch-drain
run_queue final_snapshot list --root "$ROOT" --name "$NAME" --owner "$OWNER"

python3 - "$OUT_DIR" <<'PY'
import json
import pathlib
import sys

out = pathlib.Path(sys.argv[1])

def load(name):
    return json.loads((out / f"{name}.json").read_text())

namespace = load("namespace")
first = load("schedule_first")
duplicate = load("schedule_duplicate")
lease = load("lease")
safe_mode = load("safe_mode")
blocked = load("risky_wake_blocked")
read_only = load("read_only_wake")
drain = load("drain")
snapshot = load("final_snapshot")

assert namespace["namespace_id"].startswith("daemon-namespace:"), namespace
assert first["enqueued"] is True, first
assert first["queue_hop_receipt"]["hop"] == "enqueued", first
assert duplicate["enqueued"] is False, duplicate
assert duplicate["duplicate_suppression_receipt"], duplicate
assert lease["lease"]["active"] is True, lease
assert lease["queue_hop_receipt"]["hop"] == "lease-acquired", lease
assert safe_mode["enabled"] is True, safe_mode
assert safe_mode["operation"] == "entered", safe_mode
assert blocked["enqueued"] is False, blocked
assert blocked["safe_mode_receipt"]["operation"] == "blocked-risky-job", blocked
assert "safe-mode-blocked-new-risky-job" in blocked["safe_mode_receipt"]["reason_codes"], blocked
assert read_only["enqueued"] is True, read_only
assert len(drain) >= 2, drain
assert snapshot["safe_mode_enabled"] is True, snapshot
assert all(job["state"] == "cancelled" for job in snapshot["jobs"]), snapshot

report = {
    "blocked_risky_wake": True,
    "duplicate_suppressed": True,
    "drained_count": len(drain),
    "final_job_states": [job["state"] for job in snapshot["jobs"]],
    "leased_job_id": lease["lease"]["job_id"],
    "namespace_id": namespace["namespace_id"],
    "ok": True,
    "read_only_wake_enqueued": True,
    "safe_mode_enabled": snapshot["safe_mode_enabled"],
    "scheduled_job_id": first["job"]["job_id"],
}
(out / "daemon_smoke_report.json").write_text(
    json.dumps(report, indent=2, sort_keys=True) + "\n"
)
print("daemon_smoke_ok=true")
print(f"daemon_smoke_report={out / 'daemon_smoke_report.json'}")
print(f"drained_count={len(drain)}")
PY
