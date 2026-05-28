#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path.cwd()
PATH = ROOT / "target/p26/verifier/local-coding-agent/plan-act-verify-output.json"

if not PATH.exists():
    print(f"missing PlanActVerify evidence: {PATH}")
    sys.exit(1)

data = json.loads(PATH.read_text(encoding="utf-8"))
required = [
    "plan_receipts",
    "tool_route_receipts",
    "tool_call_receipts",
    "verification_receipts",
    "finalization",
]
missing = [key for key in required if key not in data]
if missing:
    print("missing PlanActVerify keys:", missing)
    sys.exit(1)

if not data["plan_receipts"] or not data["tool_route_receipts"]:
    print("PlanActVerify did not emit plan/tool-route receipts")
    sys.exit(1)
if not data["tool_call_receipts"]:
    print("PlanActVerify did not emit tool-call receipts")
    sys.exit(1)
if not data["verification_receipts"]:
    print("PlanActVerify did not emit verification receipts")
    sys.exit(1)
if data["finalization"] is None:
    print("PlanActVerify missing finalization receipt")
    sys.exit(1)

print("PlanActVerify receipt evidence: pass")
