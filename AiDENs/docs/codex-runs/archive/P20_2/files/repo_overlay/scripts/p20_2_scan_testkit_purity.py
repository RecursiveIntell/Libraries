#!/usr/bin/env python3
import argparse, json, re, sys
from pathlib import Path

ALLOWED = {
    'aidens-contracts',
    'chrono', 'serde', 'serde_json', 'thiserror', 'uuid', 'anyhow', 'toml'
}
FORBIDDEN_PREFIX = 'aidens-'

def dependency_names(cargo_toml: Path):
    text = cargo_toml.read_text(errors='ignore')
    deps = set()
    in_deps = False
    for line in text.splitlines():
        s = line.strip()
        if s.startswith('['):
            in_deps = s in {'[dependencies]', '[dev-dependencies]', '[build-dependencies]'}
            continue
        if in_deps and '=' in s and not s.startswith('#'):
            name = s.split('=',1)[0].strip().strip('"')
            deps.add(name)
    return deps

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('root', nargs='?', default='.')
    ap.add_argument('--require-integration-crate', action='store_true')
    ap.add_argument('--json-out', default='target/aidens-p20-2-audit/testkit-purity.json')
    args = ap.parse_args()
    root = Path(args.root).resolve()
    cargo = root / 'crates' / 'aidens-testkit' / 'Cargo.toml'
    report = {'ok': True, 'forbidden_dependencies': [], 'missing_testkit': False, 'missing_integration_crate': False}
    if not cargo.exists():
        report['ok'] = False
        report['missing_testkit'] = True
    else:
        deps = sorted(dependency_names(cargo))
        forbidden = [d for d in deps if d.startswith(FORBIDDEN_PREFIX) and d not in ALLOWED]
        report['dependencies'] = deps
        report['forbidden_dependencies'] = forbidden
        if forbidden:
            report['ok'] = False
    if args.require_integration_crate and not (root / 'crates' / 'aidens-integration-tests' / 'Cargo.toml').exists():
        report['ok'] = False
        report['missing_integration_crate'] = True
    out = root / args.json_out
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(json.dumps(report, indent=2))
    if not report['ok']:
        sys.exit(1)
if __name__ == '__main__':
    main()
