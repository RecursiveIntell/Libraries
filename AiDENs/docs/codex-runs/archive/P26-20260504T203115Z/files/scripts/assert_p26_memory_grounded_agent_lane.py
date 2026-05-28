#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path.cwd()
SPEC = ROOT / "examples/agents/memory-grounded-agent/agent.json"
BUNDLE = ROOT / "target/p26/verifier/memory-grounded-agent/run-bundle.json"

if not SPEC.exists() or not BUNDLE.exists():
    print("missing memory-grounded spec or run bundle")
    sys.exit(1)

spec = json.loads(SPEC.read_text(encoding="utf-8"))
bundle = json.loads(BUNDLE.read_text(encoding="utf-8"))

policy = spec.get("memory_policy", {})
if not policy.get("enabled") or policy.get("mode") != "canonical-seam":
    print("memory-grounded example does not require canonical seam")
    sys.exit(1)

receipts = bundle.get("memory_grounding_receipts") or []
if not receipts or not all("memory-grounding" in str(receipt) for receipt in receipts):
    print("memory grounding receipts missing or malformed")
    sys.exit(1)

if bundle.get("failure", {}).get("blocked") and not bundle.get("repair_plan_receipts"):
    print("blocked memory-grounding run lacks repair display receipt")
    sys.exit(1)

if bundle.get("support", {}).get("support_tier") != "supported-local":
    print("memory-grounded run lost supported-local wrapper label")
    sys.exit(1)

print("memory-grounded agent evidence: pass")
