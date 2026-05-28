#!/usr/bin/env python3
import sys
from pathlib import Path
root = Path(sys.argv[1]) if len(sys.argv)>1 else Path('.')
status = root/'SUPPORT_PROFILE.md'
scaffold_crates = [
    'aidens-profile-daemon',
    'aidens-profile-desktop',
    'aidens-profile-memory',
    'aidens-profile-research',
]
if status.exists():
    text=status.read_text(errors='ignore').lower()
    forbidden=['profile-daemon supported','profile-desktop supported','profile-memory supported','profile-research supported']
    for f in forbidden:
        if f in text:
            print('support profile may promote scaffold profile:', f, file=sys.stderr)
            sys.exit(2)
    if '## scaffold-only' not in text:
        print('support profile missing scaffold-only section', file=sys.stderr)
        sys.exit(3)
    for crate in scaffold_crates:
        if crate not in text:
            print('support profile does not fence scaffold crate:', crate, file=sys.stderr)
            sys.exit(4)
print('scaffold support guard OK')
