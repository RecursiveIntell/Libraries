#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path.cwd()
RUNS = [
    ROOT / "target/p26/verifier/local-coding-agent",
    ROOT / "target/p26/verifier/memory-grounded-agent",
]

found = 0
for run in RUNS:
    abstention = run / "abstention.json"
    repair = run / "repair-plan.json"
    if not abstention.exists() or not repair.exists():
        continue
    found += 1
    abstention_data = json.loads(abstention.read_text(encoding="utf-8"))
    repair_data = json.loads(repair.read_text(encoding="utf-8"))
    if not abstention_data.get("reason_code"):
        print(f"abstention missing reason_code under {run}")
        sys.exit(1)
    if repair_data.get("display_only") is not True:
        print(f"repair plan is not marked display_only under {run}")
        sys.exit(1)
    if not repair_data.get("candidate_repair_actions"):
        print(f"repair plan missing candidate actions under {run}")
        sys.exit(1)

if found == 0:
    print("missing abstention/repair evidence in all P26 verifier runs")
    sys.exit(1)

print("abstention and repair display evidence: pass")
