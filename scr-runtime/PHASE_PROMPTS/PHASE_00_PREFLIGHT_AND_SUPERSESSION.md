# PHASE 00 PREFLIGHT AND SUPERSESSION

Run preflight, archive/supersede old P32 control files if present, create run dirs, and produce source-basis receipt. Do not edit app code.

At the end of this phase, run:

```bash
python3 scripts/p32r3_phase_gate.py --run-id P32R3 --phase phase_00_preflight_and_supersession --status pass
```

If blocked, use `--status blocked --note "<reason>"` and stop.
