#!/usr/bin/env python3
from pathlib import Path
import json
import sys

ROOT = Path.cwd()
BUNDLES = [
    ROOT / "target/p26/verifier/local-coding-agent/run-bundle.json",
    ROOT / "target/p26/verifier/memory-grounded-agent/run-bundle.json",
]

for path in BUNDLES:
    if not path.exists():
        print(f"missing run bundle: {path}")
        sys.exit(1)
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("schema") != "AiDENsRunBundleV3":
        print(f"not V3: {path}")
        sys.exit(1)
    replay = data.get("replay") or {}
    if not replay.get("replay_command"):
        print(f"missing replay command: {path}")
        sys.exit(1)
    if replay.get("deterministic_compare") is not True:
        print(f"deterministic_compare not true: {path}")
        sys.exit(1)
    if not replay.get("normalized_digest"):
        print(f"missing normalized digest: {path}")
        sys.exit(1)
    if len(data.get("replay_instructions") or []) < 2:
        print(f"missing replay instructions: {path}")
        sys.exit(1)
    event_log = data.get("event_log") or {}
    if not event_log.get("digest") or not event_log.get("replay_normalized_digest"):
        print(f"missing event log digests: {path}")
        sys.exit(1)

print("AiDENsRunBundleV3 replay evidence: pass")
