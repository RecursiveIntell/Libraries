#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
required = [
    ROOT / 'README.md',
    ROOT / '02_MASTER_ISSUE_MATRIX.md',
    ROOT / 'STATUS_DASHBOARD.md',
    ROOT / 'SUPPORT_PROFILE.md',
    ROOT / 'AGENTS.md',
    ROOT / 'PROMPT.md',
    ROOT / 'release' / 'closeout_receipt_v1.json',
]
missing = [str(p.relative_to(ROOT)) for p in required if not p.exists()]
if missing:
    raise SystemExit('missing active closeout files: ' + ', '.join(missing))
receipt = json.loads((ROOT / 'release' / 'closeout_receipt_v1.json').read_text())
print(
    'closeout lane structure present:',
    receipt.get('snapshot'),
    'supported crates=',
    receipt.get('supported_closeout_lane', {}).get('crate_count'),
    'NOTE: receipt is historical; fresh current-HEAD gates are required',
)
