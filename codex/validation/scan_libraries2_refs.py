#!/usr/bin/env python3
import argparse, pathlib, sys, json
parser=argparse.ArgumentParser(description='Scan active source for Libraries2 references.')
parser.add_argument('--root', default='.')
parser.add_argument('--allow-archives', action='store_true')
parser.add_argument('--json-out', default='')
args=parser.parse_args()
root=pathlib.Path(args.root).expanduser().resolve()
patterns=['Libraries2','_vendor/Libraries2','Libraries.BAK']
archive_markers=['archive','archives','codex-runs','receipts','generated','package-archives','root-markdown-archive','canonical_ownership_source_drift_audit','libraries2_salvage_into_libraries_pack']
skip_dirs={'.git','target','node_modules','.venv','venv','dist','build','__pycache__'}
text_ext={'.rs','.toml','.md','.json','.yaml','.yml','.py','.js','.ts','.tsx','.jsx','.sh','.txt','.lock'}
findings=[]
for p in root.rglob('*'):
    if not p.is_file(): continue
    if any(part in skip_dirs for part in p.parts): continue
    if p.suffix.lower() not in text_ext and p.name not in {'Cargo.lock','package-lock.json','pnpm-lock.yaml'}: continue
    rel=str(p.relative_to(root))
    is_archive=any(marker in rel.lower() for marker in archive_markers)
    if args.allow_archives and is_archive: continue
    try: text=p.read_text(encoding='utf-8', errors='replace')
    except Exception: continue
    for pat in patterns:
        if pat in text or pat in rel:
            findings.append({'path':rel,'pattern':pat,'archive_like':is_archive})
            break
if args.json_out: pathlib.Path(args.json_out).write_text(json.dumps(findings,indent=2),encoding='utf-8')
if findings:
    print(f'ACTIVE_LIBRARIES2_REFS={len(findings)}')
    for f in findings[:500]: print(f"{f['path']} :: {f['pattern']}")
    sys.exit(4)
print('ACTIVE_LIBRARIES2_REFS=0')
