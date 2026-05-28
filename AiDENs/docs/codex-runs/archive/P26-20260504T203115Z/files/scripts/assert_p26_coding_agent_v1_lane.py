#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path.cwd()
RUN = ROOT / "target/p26/verifier/local-coding-agent"
BUNDLE = RUN / "run-bundle.json"

if not BUNDLE.exists():
    print(f"missing coding-agent V1 run bundle: {BUNDLE}")
    sys.exit(1)

bundle = json.loads(BUNDLE.read_text(encoding="utf-8"))
if bundle.get("schema") != "AiDENsRunBundleV3":
    print("coding-agent lane did not emit AiDENsRunBundleV3")
    sys.exit(1)

if bundle.get("support", {}).get("support_tier") != "supported-local":
    print("coding-agent lane missing supported-local support tier")
    sys.exit(1)

if not bundle.get("tool_receipts"):
    print("coding-agent lane missing tool receipts")
    sys.exit(1)

if not bundle.get("permit_receipts"):
    print("coding-agent lane missing permit/approval evidence")
    sys.exit(1)

for name in ["abstention.json", "repair-plan.json", "event-log.ndjson"]:
    if not (RUN / name).exists():
        print(f"coding-agent lane missing {name}")
        sys.exit(1)

print("coding-agent V1 lane evidence: pass")
