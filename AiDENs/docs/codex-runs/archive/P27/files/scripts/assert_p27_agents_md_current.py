#!/usr/bin/env python3
import sys
from pathlib import Path
root = Path(sys.argv[1]) if len(sys.argv)>1 else Path('.')
p = root/'AGENTS.md'
if not p.exists():
    print('AGENTS.md missing', file=sys.stderr)
    sys.exit(1)
text = p.read_text(errors='ignore')
required = ['P27 Super-Pass','11A','canonical sibling','verify_current.sh','P27_OPERATOR_PASTE_FIRST.md']
missing = [s for s in required if s not in text]
if missing:
    print('AGENTS.md missing required markers:', missing, file=sys.stderr)
    sys.exit(2)
for stale in ['P23 AiDENs Product-Capability','This Codex run is P23','V29 Agent Coordination']:
    if stale in text:
        print('AGENTS.md still contains stale marker:', stale, file=sys.stderr)
        sys.exit(3)
print('p27 AGENTS.md OK')
