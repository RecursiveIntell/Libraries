#!/usr/bin/env python3
import argparse, json, sys
from pathlib import Path

REQUIRED_FIELDS = {'id','risk_surface','input','expected_policy','required_receipts','forbidden_behavior'}
VALID_POLICIES = {'allow','allow_with_disclosure','require_alternatives','require_user_confirmation','defer_to_professional_or_external_source','block','quarantine'}

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('path', nargs='?', default='evals/p20_agency_eval_cases.jsonl')
    args = ap.parse_args()
    path = Path(args.path)
    if not path.exists():
        print(f'missing agency eval file: {path}', file=sys.stderr)
        sys.exit(1)
    ids = set(); count = 0
    errors = []
    for i,line in enumerate(path.read_text(encoding='utf-8').splitlines(), 1):
        if not line.strip():
            continue
        count += 1
        try:
            obj = json.loads(line)
        except Exception as e:
            errors.append(f'line {i}: invalid json: {e}'); continue
        missing = REQUIRED_FIELDS - set(obj)
        if missing:
            errors.append(f'line {i}: missing {sorted(missing)}')
        if obj.get('id') in ids:
            errors.append(f'line {i}: duplicate id {obj.get("id")}')
        ids.add(obj.get('id'))
        if obj.get('expected_policy') not in VALID_POLICIES:
            errors.append(f'line {i}: invalid expected_policy {obj.get("expected_policy")}')
        for arr in ['required_receipts','forbidden_behavior']:
            if not isinstance(obj.get(arr), list) or not all(isinstance(x,str) for x in obj.get(arr, [])):
                errors.append(f'line {i}: {arr} must be list[str]')
        if not isinstance(obj.get('input'), dict):
            errors.append(f'line {i}: input must be object')
    if count < 8:
        errors.append(f'expected at least 8 cases, found {count}')
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        sys.exit(1)
    print(f'agency eval cases valid: {count}')
if __name__ == '__main__':
    main()
