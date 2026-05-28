#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, os, re, shutil, subprocess, sys, tomllib
from pathlib import Path
from collections import defaultdict

INCLUDE_RE = re.compile(r'include_(?:str|bytes)!\s*\(\s*"([^"]+)"\s*\)')

REQUIRED_PATHS = [
    'evals/p20_agency_eval_cases.jsonl',
    'scripts/verify.sh',
    'scripts/p20_verify.sh',
]

PRODUCTION_DEPS_FORBIDDEN_IN_TESTKIT = {
    'aidens-agency-kit','aidens-boundary-kit','aidens-cli','aidens-daemon-kit',
    'aidens-governance-kit','aidens-kernel-kit','aidens-memory-kit','aidens-provider-kit',
    'aidens-repair-kit','aidens-receipts','aidens-runner','aidens-budget-kit',
    'aidens-permit-kit','aidens-tool-kit',
}

def rel(root: Path, p: Path) -> str:
    try: return p.resolve().relative_to(root.resolve()).as_posix()
    except Exception: return str(p)

def load_crates(root: Path):
    crates = {}
    for toml in (root/'crates').glob('*/Cargo.toml'):
        data = tomllib.load(open(toml, 'rb'))
        name = data.get('package', {}).get('name', toml.parent.name)
        crates[name] = {'path': rel(root, toml.parent), 'deps': {}, 'dev_deps': {}, 'build_deps': {}}
        for sec, key in [('dependencies','deps'), ('dev-dependencies','dev_deps'), ('build-dependencies','build_deps')]:
            for dep in data.get(sec, {}):
                if dep.startswith('aidens'):
                    crates[name][key][dep] = True
    return crates

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--root', default='.')
    ap.add_argument('--out', default=None)
    ap.add_argument('--markdown', default=None)
    ap.add_argument('--fail-on-blocking', action='store_true')
    ap.add_argument('--allow-testkit-production-deps', action='store_true')
    ap.add_argument(
        '--aidens-overlay-only',
        action='store_true',
        help='Allow an AiDENs-only package scan without canonical sibling crates.',
    )
    args = ap.parse_args()
    root = Path(args.root).resolve()
    findings = []
    def add(sev, category, message, path=None):
        findings.append({'severity':sev, 'category':category, 'message':message, 'path':path})
    # Required paths
    for p in REQUIRED_PATHS:
        if not (root/p).exists(): add('blocking','missing_required_path',f'required path missing: {p}',p)
    # Manifest
    manifest = root/'MANIFEST.txt'
    if manifest.exists():
        for line in manifest.read_text(errors='ignore').splitlines():
            p=line.strip()
            if not p or p.startswith('#'): continue
            if not (root/p).exists(): add('blocking','manifest_missing',f'MANIFEST.txt lists missing path: {p}',p)
    else:
        add('warning','manifest_absent','MANIFEST.txt not found')
    # include refs
    refs=0
    for rs in root.rglob('*.rs'):
        text=rs.read_text(encoding='utf-8', errors='ignore')
        for m in INCLUDE_RE.finditer(text):
            refs+=1
            target=(rs.parent/m.group(1)).resolve()
            if not target.exists():
                add('blocking','missing_include_target',f'{rel(root,rs)} includes missing {m.group(1)} -> {rel(root,target)}',rel(root,rs))
    # eval validation shallow
    eval_path=root/'evals/p20_agency_eval_cases.jsonl'
    if eval_path.exists():
        count=0
        required={'id','risk_surface','input','expected_policy','required_receipts','forbidden_behavior'}
        for i,line in enumerate(eval_path.read_text(encoding='utf-8').splitlines(),1):
            if not line.strip(): continue
            count+=1
            try: obj=json.loads(line)
            except Exception as e:
                add('blocking','invalid_agency_eval_json',f'line {i}: {e}',str(eval_path)); continue
            missing=required-set(obj)
            if missing: add('blocking','invalid_agency_eval_case',f'line {i}: missing {sorted(missing)}',str(eval_path))
            if not obj.get('required_receipts'): add('blocking','invalid_agency_eval_case',f'line {i}: required_receipts empty',str(eval_path))
        if count < 8: add('blocking','agency_eval_too_small',f'expected at least 8 agency eval cases, got {count}',str(eval_path))
    # cargo availability
    if not shutil.which('cargo'): add('warning','toolchain_unavailable','cargo not found in current environment')
    if not shutil.which('rustc'): add('warning','toolchain_unavailable','rustc not found in current environment')
    # crates topology
    crates=load_crates(root)
    testkit=crates.get('aidens-testkit')
    if testkit and not args.allow_testkit_production_deps:
        bad=sorted(PRODUCTION_DEPS_FORBIDDEN_IN_TESTKIT & set(testkit['deps']))
        for dep in bad:
            add('blocking','testkit_impure_dependency',f'aidens-testkit normal-depends on production crate {dep}','crates/aidens-testkit/Cargo.toml')
        reverse=[]
        for name,data in crates.items():
            if 'aidens-testkit' in data['dev_deps'] and name in testkit['deps']:
                reverse.append(name)
        for name in sorted(reverse):
            add('warning','testkit_reverse_dev_loop',f'{name} dev-depends on aidens-testkit while testkit depends on {name}',crates[name]['path']+'/Cargo.toml')
    # canonical baseline presence
    sibling_names=['stack-ids','semantic-memory','semantic-memory-forge','forge-memory-bridge','knowledge-runtime','llm-tool-runtime','verification-control','recursive-kernel-core']
    present=[name for name in sibling_names if (root.parent/name).exists()]
    if len(present) < 4 and not args.aidens_overlay_only:
        add('blocking','canonical_baseline_unavailable',f'canonical sibling baseline mostly unavailable from {root.parent}; present={present}. Ownership duplicate scanner cannot certify.',None)
    report={'root':str(root),'findings':findings,'summary':{'blocking':sum(f['severity']=='blocking' for f in findings),'warning':sum(f['severity']=='warning' for f in findings),'include_refs':refs,'crate_count':len(crates)}}
    if args.out:
        Path(args.out).parent.mkdir(parents=True, exist_ok=True)
        Path(args.out).write_text(json.dumps(report, indent=2), encoding='utf-8')
    md = ['# P20.1 hard code audit','',f"Blocking findings: {report['summary']['blocking']}",f"Warning findings: {report['summary']['warning']}",'']
    for sev in ['blocking','warning']:
        md.append(f'## {sev.title()} findings')
        items=[f for f in findings if f['severity']==sev]
        if not items: md.append('- None')
        for f in items:
            path=f.get('path') or ''
            md.append(f"- `{f['category']}` {path}: {f['message']}")
        md.append('')
    if args.markdown:
        Path(args.markdown).parent.mkdir(parents=True, exist_ok=True)
        Path(args.markdown).write_text('\n'.join(md), encoding='utf-8')
    else:
        print('\n'.join(md))
    if args.fail_on_blocking and report['summary']['blocking']:
        sys.exit(2)

if __name__ == '__main__': main()
