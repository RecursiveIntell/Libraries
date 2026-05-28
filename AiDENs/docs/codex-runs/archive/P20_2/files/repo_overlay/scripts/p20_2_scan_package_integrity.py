#!/usr/bin/env python3
import argparse, json, os, re, sys
from pathlib import Path

INCLUDE_RE = re.compile(r'include_(?:str|bytes)!\s*\(\s*"([^"]+)"\s*\)', re.S)

REQUIRED = [
    'evals/p20_agency_eval_cases.jsonl',
    'scripts/p20_2_scan_package_integrity.py',
    'scripts/p20_2_scan_testkit_purity.py',
    'scripts/p20_2_validate_agency_cases.py',
    'scripts/p20_2_verify.sh',
]

def scan(root: Path):
    missing_includes = []
    include_count = 0
    for rs in root.rglob('*.rs'):
        if any(part in {'target', '.git'} for part in rs.parts):
            continue
        text = rs.read_text(errors='ignore')
        for rel in INCLUDE_RE.findall(text):
            include_count += 1
            target = (rs.parent / rel).resolve()
            try:
                target.relative_to(root.resolve())
            except ValueError:
                # External include; still report if absent.
                pass
            if not target.exists():
                missing_includes.append({'file': str(rs.relative_to(root)), 'include': rel, 'resolved': str(target)})
    missing_required = [p for p in REQUIRED if not (root / p).exists()]
    manifest_missing = []
    for mf in ['MANIFEST.txt']:
        path = root / mf
        if path.exists():
            for raw in path.read_text(errors='ignore').splitlines():
                line = raw.strip()
                if not line or line.startswith('#') or line.endswith(':'):
                    continue
                if line.startswith('- '):
                    line = line[2:].strip()
                if '/' in line or '.' in line:
                    if not (root / line).exists():
                        manifest_missing.append({'manifest': mf, 'entry': line})
    json_manifest = root / 'MANIFEST.json'
    if json_manifest.exists():
        try:
            data = json.loads(json_manifest.read_text())
            entries = data if isinstance(data, list) else data.get('files', []) if isinstance(data, dict) else []
            for entry in entries:
                if isinstance(entry, str) and not (root / entry).exists():
                    manifest_missing.append({'manifest': 'MANIFEST.json', 'entry': entry})
        except Exception as e:
            manifest_missing.append({'manifest': 'MANIFEST.json', 'entry': f'<invalid json: {e}>'})
    return {
        'include_count': include_count,
        'missing_includes': missing_includes,
        'missing_required': missing_required,
        'manifest_missing': manifest_missing,
        'ok': not missing_includes and not missing_required and not manifest_missing,
    }

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('root', nargs='?', default='.')
    ap.add_argument('--json-out', default='target/aidens-p20-2-audit/package-integrity.json')
    args = ap.parse_args()
    root = Path(args.root).resolve()
    report = scan(root)
    out = root / args.json_out
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(json.dumps(report, indent=2))
    if not report['ok']:
        sys.exit(1)
if __name__ == '__main__':
    main()
