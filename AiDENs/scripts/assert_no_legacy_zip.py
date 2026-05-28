#!/usr/bin/env python3
"""Fail if legacy zip.py remains a runnable alternate packager."""
from pathlib import Path
import sys
root = Path(sys.argv[1] if len(sys.argv) > 1 else ".")
zip_py = root / "zip.py"
if not zip_py.exists():
    print("ok: no legacy zip.py")
    sys.exit(0)
text = zip_py.read_text(errors="replace")
markers = ["Use z.py", "deprecated", "disabled", "sys.exit", "raise SystemExit"]
if all(m.lower() in text.lower() for m in ["z.py", "deprecated"]) and ("sys.exit" in text or "SystemExit" in text):
    print("ok: zip.py is a deprecating wrapper")
    sys.exit(0)
print("FAIL: legacy zip.py appears runnable. Remove it, archive it, or convert to hard-failing wrapper.", file=sys.stderr)
sys.exit(2)
