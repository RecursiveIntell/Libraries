#!/usr/bin/env python3
import json, sys, os, re, hashlib, datetime
from pathlib import Path

def read_payload():
    try:
        raw = sys.stdin.read()
        return json.loads(raw) if raw.strip() else {}
    except Exception:
        return {}

def repo_root():
    try:
        import subprocess
        out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip()
        return Path(out)
    except Exception:
        return Path.cwd()

def receipts_dir():
    root = repo_root()
    d = root / ".codex-runs" / "_hook_receipts"
    d.mkdir(parents=True, exist_ok=True)
    return d

def write_receipt(kind, payload, status="ok"):
    now = datetime.datetime.utcnow().replace(microsecond=0).isoformat() + "Z"
    data = {"kind": kind, "status": status, "recorded_time": now, "payload_keys": sorted(payload.keys())}
    h = hashlib.sha256(json.dumps(data, sort_keys=True).encode()).hexdigest()[:16]
    (receipts_dir() / f"{now.replace(':','')}-{kind}-{h}.json").write_text(json.dumps(data, indent=2), encoding="utf-8")

payload = read_payload()
text = json.dumps(payload)
bad = re.findall(r"@filename|\{feature\}|<placeholder>", text, flags=re.I)
if bad:
    print("Blocked unresolved placeholder(s): " + ", ".join(sorted(set(bad))), file=sys.stderr)
    sys.exit(2)
write_receipt("user_prompt_submit", payload)
sys.exit(0)
