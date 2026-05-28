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
tool = event.get("tool_name", "")
inp = event.get("tool_input") or {}
cmd = inp.get("command") if isinstance(inp, dict) else ""
danger = [
    r"\brm\s+-rf\s+/(?:\s|$)",
    r"\bgit\s+reset\s+--hard\b",
    r"\bgit\s+clean\s+-fdx\b",
    r"\bcargo\s+publish\b",
    r"\bgh\s+release\s+create\b",
]
if cmd:
    for pat in danger:
        if re.search(pat, cmd):
            out({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny", "permissionDecisionReason": f"Blocked dangerous command by SCR P32 policy: {pat}"}})
            raise SystemExit(0)
out({"hookSpecificOutput": {"hookEventName": "PreToolUse", "additionalContext": "SCR P32 policy: record material commands in docs/P32_COMMAND_RECEIPTS.md and do not claim completion without gates."}})
