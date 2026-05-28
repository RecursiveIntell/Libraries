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
# Do not mutate repo from hook. Emit a reminder only; hooks are not the canonical receipt.
out({"systemMessage": "SCR P32 reminder: append material command/tool result to docs/P32_COMMAND_RECEIPTS.md; hook output is not sufficient evidence."})
