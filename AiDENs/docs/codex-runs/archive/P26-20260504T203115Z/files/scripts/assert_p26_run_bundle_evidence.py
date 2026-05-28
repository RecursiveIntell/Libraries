#!/usr/bin/env python3
from pathlib import Path
import json, sys
found = list(Path.cwd().rglob("*run-bundle*.json"))
if not found:
    print("no run bundle json found")
    sys.exit(1)
required_keys = {
    "schema",
    "run_id",
    "trace_ctx",
    "attempt_family_id",
    "attempt_id",
    "trial_id",
    "agent_spec_digest",
    "memory_grounding_receipts",
    "tool_receipts",
    "permit_receipts",
    "verification_receipts",
    "support_labels",
    "replay_instructions",
}
for path in found:
    try:
        data=json.loads(path.read_text())
    except Exception:
        continue
    if data.get("schema") == "AiDENsRunBundleV3" and required_keys.issubset(data.keys()):
        print("AiDENsRunBundleV3 evidence:", path)
        sys.exit(0)
print("no AiDENsRunBundleV3 with required P26 evidence keys found")
sys.exit(1)
