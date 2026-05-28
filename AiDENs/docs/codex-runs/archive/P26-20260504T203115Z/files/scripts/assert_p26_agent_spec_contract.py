#!/usr/bin/env python3
from pathlib import Path
import json, sys
root = Path.cwd()
required = [
    "examples/agents/local-coding-agent/agent.json",
    "examples/agents/memory-grounded-agent/agent.json",
    "schemas/agent-spec/v1.schema.json",
    "schemas/aidens-run-bundle/v3.schema.json",
    "tests/fixtures/p26/agent_spec_v1.json",
    "tests/fixtures/p26/agent_spec_v1_invalid.json",
]
missing = [p for p in required if not (root/p).exists()]
if missing:
    print("missing AgentSpecV1 artifacts:", missing)
    sys.exit(1)
valid = json.loads((root/"tests/fixtures/p26/agent_spec_v1.json").read_text())
invalid = json.loads((root/"tests/fixtures/p26/agent_spec_v1_invalid.json").read_text())
if valid.get("schema") != "AgentSpecV1":
    print("valid fixture is not AgentSpecV1")
    sys.exit(1)
if invalid.get("agent_id") != "":
    print("invalid fixture no longer exercises empty agent_id")
    sys.exit(1)
print("AgentSpecV1 contract artifacts: pass")
