#!/usr/bin/env python3
"""Check for P23 capability evidence surfaces."""
from pathlib import Path
import sys
root = Path(sys.argv[1] if len(sys.argv) > 1 else ".")
required_any = [
    ["crates/aidens-runner/src/lib.rs", "crates/aidens-cli/src/lib.rs"],
    ["examples/configs/coding-agent.toml", "fixtures/test-agent/basic-agent.toml"],
]
for group in required_any:
    if not any((root / p).exists() for p in group):
        print(f"FAIL: none found from {group}", file=sys.stderr); sys.exit(2)
text = "\n".join(p.read_text(errors="replace") for p in [root/"crates/aidens-cli/src/lib.rs", root/"crates/aidens-runner/src/lib.rs"] if p.exists())
needles = ["receipt", "support", "provider", "budget"]
missing = [n for n in needles if n.lower() not in text.lower()]
if missing:
    print("FAIL: capability surface missing terms: " + ", ".join(missing), file=sys.stderr); sys.exit(2)
print("ok: AiDENs capability contract surface present")
