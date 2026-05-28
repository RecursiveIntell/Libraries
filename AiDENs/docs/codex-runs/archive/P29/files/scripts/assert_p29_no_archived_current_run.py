#!/usr/bin/env python3
from pathlib import Path
import json, sys

archive_reports = list(Path(".").glob("*codex-archive.json")) + list(Path(".").glob("target/**/*.codex-archive.json"))
bad = []
for report in archive_reports:
    try:
        data = json.loads(report.read_text())
    except Exception:
        continue
    moved = data.get("moved", []) or data.get("codex_archive", {}).get("moved", [])
    for item in moved:
        orig = item.get("original_path","")
        run_id = item.get("run_id","")
        if run_id == "P29" or "/p29/" in orig.lower() or "P29_" in orig or "p29_" in orig:
            bad.append((str(report), orig))
if bad:
    print("Active P29 artifacts archived as stale:")
    for report, orig in bad:
        print(f" - {report}: {orig}")
    sys.exit(1)
print("no P29 current-run artifacts archived")
