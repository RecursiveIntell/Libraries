#!/usr/bin/env python3
from pathlib import Path
import argparse, sys
ap=argparse.ArgumentParser(); ap.add_argument('--root',default='.'); args=ap.parse_args()
root=Path(args.root); missing=[]
manifest=root/'MANIFEST.txt'
if not manifest.exists(): print('MANIFEST.txt missing'); sys.exit(2)
for line in manifest.read_text(errors='ignore').splitlines():
    p=line.strip()
    if not p or p.startswith('#'): continue
    if not (root/p).exists(): missing.append(p)
if missing:
    print('manifest missing paths:')
    for p in missing: print('-',p)
    sys.exit(2)
print('manifest OK')
