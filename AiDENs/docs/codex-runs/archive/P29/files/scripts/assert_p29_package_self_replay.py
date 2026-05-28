#!/usr/bin/env python3
import argparse, zipfile, tempfile, subprocess, sys, os
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--package", required=True)
args = parser.parse_args()

pkg = Path(args.package)
if not pkg.exists():
    print(f"package not found: {pkg}")
    sys.exit(1)

with tempfile.TemporaryDirectory(prefix="p29_replay_") as td:
    td = Path(td)
    with zipfile.ZipFile(pkg, "r") as z:
        z.extractall(td)
    # Find AiDENs root.
    candidates = [td / "AiDENs", td]
    root = None
    for c in candidates:
        if (c / "scripts" / "verify_current.sh").exists():
            root = c
            break
    if root is None:
        print("verify_current.sh not found in extracted package")
        sys.exit(1)
    result = subprocess.run(["bash", "scripts/verify_current.sh"], cwd=root, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    print(result.stdout)
    if result.returncode != 0:
        print(f"package self-replay failed: {result.returncode}")
        sys.exit(result.returncode)
print("package self-replay passed")
