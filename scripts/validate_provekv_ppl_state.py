#!/usr/bin/env python3
import json, sys
from pathlib import Path

if len(sys.argv) != 2:
    raise SystemExit("usage: validate_provekv_ppl_state.py <state.json>")
path = Path(sys.argv[1])
state = json.loads(path.read_text())
missing = [k for k in ["model", "corpus", "phase0", "phase1"] if k not in state]
if missing:
    raise SystemExit(f"missing required keys: {missing}")
phase0 = state["phase0"]
phase1 = state["phase1"]
for label, phase in [("phase0", phase0), ("phase1", phase1)]:
    ppl = phase.get("ppl")
    if not isinstance(ppl, (int, float)) or ppl <= 0:
        raise SystemExit(f"{label}.ppl must be positive number")
ratio = phase1.get("compression_ratio") or phase1.get("manifest", {}).get("compression_ratio")
if not isinstance(ratio, (int, float)) or ratio <= 1.0:
    raise SystemExit("phase1 compression_ratio must be > 1.0")
if "delta_ppl_pct" not in phase1:
    raise SystemExit("phase1.delta_ppl_pct missing")
print(f"provekv ppl state: PASS model={state.get('model')} corpus={state.get('corpus')} ratio={ratio:.4f}x")
