#!/usr/bin/env python3
import argparse, pathlib, sys, json
import tomllib
parser=argparse.ArgumentParser(description='Scan Cargo.toml path dependencies for missing targets.')
parser.add_argument('--root', default='.', help='root to scan')
parser.add_argument('--json-out', default='')
parser.add_argument('--fail-on-salvage', action='store_true', help='Treat archived salvage path dependency gaps as failures.')
args=parser.parse_args()
root=pathlib.Path(args.root).expanduser().resolve()
findings=[]
parse_errors=[]
def scan_table(table, manifest, section):
    if not isinstance(table, dict):
        return
    for name, spec in table.items():
        if not isinstance(spec, dict) or 'path' not in spec:
            continue
        rel=spec['path']
        target=(manifest.parent/rel).resolve()
        if not target.exists():
            manifest_rel=str(manifest.relative_to(root))
            zone='salvage' if '_salvage_from_libraries2' in manifest_rel else 'active'
            findings.append({'manifest':str(manifest),'manifest_rel':manifest_rel,'zone':zone,'section':section,'dependency':name,'path':rel,'resolved':str(target),'severity':'broken-path-dep'})
for toml in root.rglob('Cargo.toml'):
    if any(part in {'.git','target','node_modules'} for part in toml.parts): continue
    try:
        data=tomllib.loads(toml.read_text(encoding='utf-8', errors='replace'))
    except Exception as exc:
        parse_errors.append({'manifest':str(toml),'error':str(exc)})
        continue
    for section in ('dependencies','dev-dependencies','build-dependencies'):
        scan_table(data.get(section), toml, section)
    workspace=data.get('workspace') or {}
    scan_table(workspace.get('dependencies'), toml, 'workspace.dependencies')
    for target_name, target in (data.get('target') or {}).items():
        if isinstance(target, dict):
            for section in ('dependencies','dev-dependencies','build-dependencies'):
                scan_table(target.get(section), toml, f'target.{target_name}.{section}')
if args.json_out:
    pathlib.Path(args.json_out).write_text(json.dumps({'findings':findings,'parse_errors':parse_errors},indent=2),encoding='utf-8')
active=[f for f in findings if f['zone']=='active']
salvage=[f for f in findings if f['zone']=='salvage']
if findings or parse_errors:
    print(f'BROKEN_PATH_DEPS={len(findings)} active={len(active)} salvage={len(salvage)} parse_errors={len(parse_errors)}')
    for f in findings: print(f"{f['manifest']}: path={f['path']} -> MISSING {f['resolved']}")
    for e in parse_errors: print(f"{e['manifest']}: PARSE_ERROR {e['error']}")
    if active or parse_errors or args.fail_on_salvage:
        sys.exit(2)
print(f'BROKEN_PATH_DEPS=0 active=0 salvage={len(salvage)}')
