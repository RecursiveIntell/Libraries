#!/usr/bin/env python3
import sys
from pathlib import Path

root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('.')
active = ['AGENTS.md','README.md','STATUS.md','SOURCE_BASIS.md','SUPPORT_PROFILE.md']
missing=[]
bad=[]
for name in active:
    p=root/name
    if not p.exists():
        missing.append(name)
        continue
    text=p.read_text(errors='ignore')
    if 'P27' not in text and name != 'SUPPORT_PROFILE.md':
        bad.append((name,'does not mention P27'))
    lowered=text.lower()
    stale_phrases=['current run | `p22`','current run: `p22`','current run | `p23`','current run: `p23`','current run | `p24`','current run: `p24`','current run | `p25`','current run: `p25`','current run | `p26`','current run: `p26`']
    for phrase in stale_phrases:
        if phrase in lowered:
            bad.append((name,f'stale phrase: {phrase}'))
if missing:
    print('missing active docs:', missing, file=sys.stderr)
    sys.exit(1)
if bad:
    for row in bad:
        print('bad active-run truth:', row, file=sys.stderr)
    sys.exit(2)
print('p27 current-run truth OK')
