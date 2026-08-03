#!/usr/bin/env python3
"""Opt-in AEW observer: append each Hermes JSON event, never block the caller."""
import json, os, sys
from datetime import datetime, timezone
path=os.environ.get("AEW_EVENTS_PATH")
for line in sys.stdin:
    if not path: continue
    try:
        event=json.loads(line)
        event["aew_received_at"]=datetime.now(timezone.utc).isoformat()
        with open(path,"a",encoding="utf-8") as f:
            f.write(json.dumps(event,sort_keys=True,separators=(",",":"))+"\n")
    except Exception:
        continue
