#!/usr/bin/env python3
import argparse, pathlib, re, sys, json
parser=argparse.ArgumentParser(description='Scan Cargo package name duplicates.')
parser.add_argument('--root', default='.')
parser.add_argument('--allow', action='append', default=[], help='package name allowed to duplicate')
parser.add_argument('--json-out', default='')
parser.add_argument('--allow-salvage-duplicates', action='store_true', default=True)
args=parser.parse_args()
root=pathlib.Path(args.root).expanduser().resolve()
name_re=re.compile(r"^\s*name\s*=\s*['\"]([^'\"]+)['\"]", re.M)
packages={}
for toml in root.rglob('Cargo.toml'):
    if any(part in {'.git','target','node_modules','docs','archive'} for part in toml.parts): continue
    text=toml.read_text(encoding='utf-8', errors='replace')
    m=name_re.search(text)
    if m:
        rel=str(toml.relative_to(root))
        zone='salvage' if '_salvage_from_libraries2' in rel else 'active'
        packages.setdefault(m.group(1),[]).append({'path':str(toml),'rel':rel,'zone':zone})
dups={}
for name, entries in packages.items():
    if name in set(args.allow):
        continue
    active=[e for e in entries if e['zone']=='active']
    if len(active)>1 or (not args.allow_salvage_duplicates and len(entries)>1):
        dups[name]=entries
if args.json_out: pathlib.Path(args.json_out).write_text(json.dumps(dups,indent=2),encoding='utf-8')
if dups:
    print(f'DUPLICATE_PACKAGES={len(dups)}')
    for k,v in sorted(dups.items()):
        print(f'[{k}]')
        for p in v: print(f"  {p['zone']} {p['path']}")
    sys.exit(3)
all_duplicate_names=sum(1 for v in packages.values() if len(v)>1)
print(f'DUPLICATE_PACKAGES=0 active=0 duplicate_names_including_salvage={all_duplicate_names}')
