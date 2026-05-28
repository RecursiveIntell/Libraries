#!/usr/bin/env python3
import argparse, pathlib, json, sys
parser=argparse.ArgumentParser(description='Find stale vendor paths that should not be active after salvage.')
parser.add_argument('--root', default='.')
parser.add_argument('--json-out', default='')
args=parser.parse_args()
root=pathlib.Path(args.root).expanduser().resolve()
forbidden=['_vendor/Libraries2','_vendor/Libraries/semantic-memory','_vendor/Libraries/stack-ids','Libraries.BAK']
findings=[]
for p in root.rglob('*'):
    rel=str(p.relative_to(root))
    if any(seg in rel for seg in ['.git/','target/','node_modules/','docs/codex-runs/archive/']): continue
    if any(tok in rel for tok in forbidden): findings.append({'path':rel,'where':'path'})
    if p.is_file() and p.suffix.lower() in {'.toml','.rs','.md','.json','.py','.ts','.js','.yaml','.yml','.sh'}:
        text=p.read_text(encoding='utf-8', errors='replace')
        for tok in forbidden:
            if tok in text: findings.append({'path':rel,'where':'content','token':tok})
if args.json_out: pathlib.Path(args.json_out).write_text(json.dumps(findings,indent=2),encoding='utf-8')
if findings:
    print(f'FORBIDDEN_VENDOR_REFS={len(findings)}')
    for f in findings[:500]: print(f)
    sys.exit(5)
print('FORBIDDEN_VENDOR_REFS=0')
