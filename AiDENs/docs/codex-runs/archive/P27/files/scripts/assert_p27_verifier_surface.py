#!/usr/bin/env python3
import re
import sys
from pathlib import Path

root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('.')
required = [
    root/'scripts'/'p27_verify.sh',
    root/'scripts'/'verify_current.sh',
    root/'scripts'/'verify.sh',
]
missing = [str(p) for p in required if not p.exists()]
if missing:
    print('missing required verifier entrypoints:', missing, file=sys.stderr)
    sys.exit(1)

refs = []
for path in list((root/'scripts').glob('**/*')) + list((root/'.github'/'workflows').glob('**/*')):
    if not path.is_file():
        continue
    try:
        text = path.read_text(errors='ignore')
    except Exception:
        continue
    for m in re.finditer(r'scripts/(p\d+_verify\.sh|verify_current\.sh|verify\.sh)|\b(p\d+_verify\.sh)\b', text):
        token = m.group(1) or m.group(2)
        refs.append((path, token))

bad = []
for path, token in refs:
    if token.startswith('p') and token != 'p27_verify.sh':
        bad.append((str(path), token, 'historical verifier reference'))
    target = root/'scripts'/token if token.endswith('.sh') else None
    if target and not target.exists():
        bad.append((str(path), token, 'target missing'))

if bad:
    for row in bad:
        print('bad verifier reference:', row, file=sys.stderr)
    sys.exit(2)

print('p27 verifier surface OK')
