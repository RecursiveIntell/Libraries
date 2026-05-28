#!/usr/bin/env python3
import argparse, pathlib, json
parser=argparse.ArgumentParser(description='Classify generated package/Codex artifacts.')
parser.add_argument('--root', default='.')
parser.add_argument('--json-out', default='')
args=parser.parse_args()
root=pathlib.Path(args.root).expanduser().resolve()
patterns=['next-codex-context','codex-context','.findings.json','.manifest.json','.excluded.json','.report.md','.codex-archive.json','.codex-runs','docs/codex-runs','projection.sqlite','memory.db','state.db']
findings=[]
for p in root.rglob('*'):
    rel=str(p.relative_to(root))
    if '.git/' in rel or 'target/' in rel or 'node_modules/' in rel: continue
    hit=[pat for pat in patterns if pat in rel]
    if hit:
        archive_like=any(tok in rel.lower() for tok in ['archive','receipts','docs/codex-runs/archive','package-archives'])
        findings.append({'path':rel,'patterns':hit,'archive_like':archive_like})
if args.json_out: pathlib.Path(args.json_out).write_text(json.dumps(findings,indent=2),encoding='utf-8')
print(f'GENERATED_ARTIFACT_CANDIDATES={len(findings)}')
for f in findings[:500]: print(f)
