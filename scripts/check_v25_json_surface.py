#!/usr/bin/env python3
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def load_json(path: Path):
    try:
        return json.loads(path.read_text(encoding='utf-8'))
    except Exception as exc:
        raise SystemExit(f'failed to parse JSON at {path}: {exc}')


required_schema_examples = [
    'applicability-context-v1',
    'profile-set-v1',
    'composition-rule-set-v1',
    'composition-receipt-v1',
    'effective-constitution-v1',
    'compiled-obligation-set-v1',
    'composition-conflict-set-v1',
    'profile-exception-bundle-v1',
    'policy-impact-diff-v1',
]

support_examples = [
    'effect-policy-profile-v1',
    'delegation-policy-profile-v1',
    'release-policy-profile-v1',
    'continuity-policy-profile-v1',
    'effective-constitution-view-v1',
    'compiled-obligation-runtime-view-v1',
    'composition-conflict-runtime-view-v1',
    'policy-impact-diff-runtime-view-v1',
]

# Validate wave manifest and schemas
wave_manifest = load_json(ROOT / 'contracts/schemas/v25/manifest.json')
for rel in wave_manifest['schema_files']:
    schema_path = ROOT / 'schemas' / Path(rel).name
    if not schema_path.exists():
        raise SystemExit(f'missing schema listed in v25 manifest: {schema_path}')
    load_json(schema_path)

for stem in required_schema_examples:
    schema = ROOT / 'schemas' / f'{stem}.schema.json'
    example = ROOT / 'examples' / f'{stem}.example.json'
    if not schema.exists():
        raise SystemExit(f'missing schema: {schema}')
    if not example.exists():
        raise SystemExit(f'missing example: {example}')
    load_json(schema)
    load_json(example)

for stem in support_examples:
    schema = ROOT / 'schemas' / f'{stem}.schema.json'
    example = ROOT / 'examples' / f'{stem}.example.json'
    if schema.exists():
        load_json(schema)
    if not example.exists():
        raise SystemExit(f'missing supporting example: {example}')
    load_json(example)

fixture_manifest = load_json(ROOT / 'contracts/fixtures/v25/manifest.json')
fixture_dir = ROOT / 'contracts/fixtures/v25'
for entry in fixture_manifest['fixtures']:
    bundle_path = fixture_dir / entry['file']
    if not bundle_path.exists():
        raise SystemExit(f'missing fixture bundle: {bundle_path}')
    bundle = load_json(bundle_path)
    if bundle.get('fixture_name') != entry['name']:
        raise SystemExit(f'fixture name mismatch for {bundle_path}')
    artifacts = bundle.get('artifacts', {})
    for key in entry['required_artifacts']:
        if key not in artifacts:
            raise SystemExit(f'fixture {bundle_path} missing required artifact {key}')

conformance_manifest = ROOT / 'conformance/v25/manifest.json'
load_json(conformance_manifest)

print('v25 JSON surface checks passed')
