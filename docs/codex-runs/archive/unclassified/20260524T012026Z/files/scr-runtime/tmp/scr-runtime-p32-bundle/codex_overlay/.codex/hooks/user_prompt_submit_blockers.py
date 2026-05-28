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
text = event.get("prompt") or event.get("user_prompt") or event.get("message") or ""
blocked = []
for pat in [r"@filename", r"\{feature\}", r"<placeholder>", r"\bTODO\b", r"\bTBD\b"]:
    if re.search(pat, text):
        blocked.append(pat)
if re.search(r"\b(done|complete|finished)\b", text, re.I) and "receipt" not in text.lower():
    blocked.append("completion claim without receipt language")
if blocked:
    out({"continue": False, "stopReason": "Blocked unresolved placeholder or unsafe completion claim: " + ", ".join(blocked)})
else:
    out({"continue": True})
