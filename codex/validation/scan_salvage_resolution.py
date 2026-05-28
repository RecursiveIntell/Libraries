#!/usr/bin/env python3
import argparse, pathlib, sys
parser=argparse.ArgumentParser(description='Check that a Libraries2 salvage item has terminal decision evidence.')
parser.add_argument('--crate', default='')
parser.add_argument('--libraries-root', default='~/Coding/Libraries')
args=parser.parse_args()
root=pathlib.Path(args.libraries_root).expanduser().resolve()
crate=args.crate
search_roots=[root/'docs', root/'_salvage_from_libraries2', root]
terms=[crate, crate.replace('-','_'), crate.replace('_','-')]
found=[]
for sr in search_roots:
    if not sr.exists(): continue
    for p in sr.rglob('*'):
        if not p.is_file(): continue
        rel=str(p.relative_to(root))
        if any(t and t in rel for t in terms): found.append(rel)
        elif p.suffix.lower() in {'.md','.json','.csv','.toml'}:
            text=p.read_text(encoding='utf-8', errors='replace')
            if any(t and t in text for t in terms): found.append(rel)
if found:
    print(f'SALVAGE_EVIDENCE_FOUND {crate}')
    for x in sorted(set(found))[:100]: print(x)
    sys.exit(0)
print(f'SALVAGE_EVIDENCE_MISSING {crate}')
sys.exit(6)
