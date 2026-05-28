#!/usr/bin/env python3
import argparse, json, sys
from pathlib import Path
PHASES = [f'{i:02d}' for i in range(11)]
def main():
    ap=argparse.ArgumentParser(); ap.add_argument('phase'); ap.add_argument('--report-dir', default='handoffs/p20_2/phase_reports')
    args=ap.parse_args(); p=args.phase.zfill(2)
    if p not in PHASES:
        print('invalid phase', file=sys.stderr); sys.exit(1)
    path=Path(args.report_dir)/f'PHASE_{p}_REPORT.md'
    if not path.exists():
        print(f'missing phase report {path}', file=sys.stderr); sys.exit(1)
    txt=path.read_text(errors='ignore').lower()
    required=['commands run','invariant','pass']
    miss=[r for r in required if r not in txt]
    if miss:
        print(f'phase report missing sections: {miss}', file=sys.stderr); sys.exit(1)
    print(f'phase {p} report gate ok')
if __name__=='__main__': main()
