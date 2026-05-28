#!/usr/bin/env python3
from __future__ import annotations
import argparse
from pathlib import Path

PHASES = [f"PHASE_{i:02d}_REPORT.md" for i in range(0, 11)]

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".")
    ap.add_argument("--upto", type=int, default=10)
    args = ap.parse_args()
    root = Path(args.root)
    missing = []
    for i in range(args.upto + 1):
        p = root / "docs" / "p20" / "reports" / f"PHASE_{i:02d}_REPORT.md"
        if not p.exists():
            missing.append(str(p))
    if missing:
        print("Missing phase reports:")
        for m in missing:
            print("-", m)
        raise SystemExit(2)
    print("Phase report gate passed")

if __name__ == "__main__":
    main()
