#!/usr/bin/env python3
import json, os, re, sys, pathlib

def read_event():
    try:
        raw = sys.stdin.read()
        return json.loads(raw) if raw.strip() else {}
    except Exception as e:
        print(f"hook input parse warning: {e}", file=sys.stderr)
        return {}

def repo_root(event):
    cwd = pathlib.Path(event.get("cwd") or os.getcwd())
    return cwd

def out(obj):
    print(json.dumps(obj, separators=(",", ":")))

event = read_event()
root = repo_root(event)
required = [
    "docs/P32_COMPLETION_REPORT.md",
    "docs/P32_COMMAND_RECEIPTS.md",
    "docs/P32_CHANGED_FILES.md",
    "docs/P32_UNRESOLVED_RISKS.md",
    "docs/P32_HOSTILE_AUDITOR_HANDOFF.md",
    "docs/P32_ROLLBACK_PLAN.md",
]
missing = [p for p in required if not (root / p).exists()]
if missing:
    out({"continue": False, "stopReason": "Missing required P32 final artifacts: " + ", ".join(missing)})
else:
    out({"continue": True})
