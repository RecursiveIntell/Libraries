#!/usr/bin/env python3
import argparse, pathlib, sys
parser=argparse.ArgumentParser(description='Validate final receipt minimum fields.')
parser.add_argument('--root', default='.')
parser.add_argument('--report', default='docs/post-salvage-validation/FINAL_REPORT.md')
args=parser.parse_args()
root=pathlib.Path(args.root).expanduser().resolve()
report=(root/args.report).resolve() if not pathlib.Path(args.report).is_absolute() else pathlib.Path(args.report)
if not report.exists():
    print(f'MISSING_FINAL_REPORT {report}')
    sys.exit(7)
text=report.read_text(encoding='utf-8', errors='replace').lower()
required=['changed files','commands run','tests','skipped','rollback','libraries2','duplicate','path dep','unresolved']
missing=[r for r in required if r not in text]
if missing:
    print('FINAL_REPORT_MISSING_FIELDS=' + ','.join(missing))
    sys.exit(8)
print('FINAL_REPORT_RECEIPT_FIELDS_PRESENT')
