#!/usr/bin/env python3
"""P27 guard for ownership scanner fail-closed behavior."""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

root = Path(sys.argv[1]) if len(sys.argv)>1 else Path('.')
p = root/'scripts'/'make_type_ownership_inventory.py'
if not p.exists():
    print('ownership scanner missing', file=sys.stderr)
    sys.exit(1)
text = p.read_text(errors='ignore')
markers = ['canonical_inventory_unavailable','fail', 'canonical']
if 'canonical_inventory_unavailable' not in text:
    print('ownership scanner does not expose canonical_inventory_unavailable fail-closed marker', file=sys.stderr)
    sys.exit(2)
if not any(s in text for s in ['sys.exit(2)', 'sys.exit(1)', 'raise SystemExit']):
    print('ownership scanner does not obviously fail closed', file=sys.stderr)
    sys.exit(3)

with tempfile.TemporaryDirectory(prefix='p27_ownership_absent_') as td:
    fixture = Path(td) / 'AiDENs'
    lib = fixture / 'crates' / 'aidens-contracts' / 'src' / 'lib.rs'
    lib.parent.mkdir(parents=True)
    lib.write_text('pub struct LocalOnlyFixtureV1;\\n', encoding='utf-8')
    result = subprocess.run(
        [sys.executable, str(p.resolve()), '--root', str(fixture)],
        text=True,
        capture_output=True,
    )
    status_path = fixture / 'docs' / 'contract-ownership' / 'OWNERSHIP_SCAN_STATUS.json'
    if result.returncode == 0:
        print('ownership scanner did not fail closed for absent canonical baseline', file=sys.stderr)
        sys.exit(4)
    combined = result.stdout + result.stderr
    if 'canonical_inventory_unavailable=true' not in combined:
        print('ownership scanner absent-baseline output lacks canonical_inventory_unavailable=true', file=sys.stderr)
        sys.exit(5)
    if not status_path.exists():
        print('ownership scanner did not write OWNERSHIP_SCAN_STATUS.json for absent baseline', file=sys.stderr)
        sys.exit(6)
    status = json.loads(status_path.read_text(encoding='utf-8'))
    if status.get('canonical_inventory_unavailable') is not True:
        print('ownership scanner status does not mark canonical_inventory_unavailable=true', file=sys.stderr)
        sys.exit(7)

print('ownership scanner fail-closed behavior OK')
