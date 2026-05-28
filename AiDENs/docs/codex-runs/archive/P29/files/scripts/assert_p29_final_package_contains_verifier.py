#!/usr/bin/env python3
from pathlib import Path
import sys, os

p = Path("scripts/p29_verify.sh")
if not p.exists():
    print("scripts/p29_verify.sh missing")
    sys.exit(1)
text = p.read_text(encoding="utf-8", errors="ignore")
if "assert_p29_run_identity.py" not in text:
    print("p29 verifier does not call P29 assertions")
    sys.exit(1)
vc = Path("scripts/verify_current.sh")
if vc.exists():
    vct = vc.read_text(encoding="utf-8", errors="ignore")
    if "p29_verify.sh" not in vct:
        print("verify_current.sh does not delegate to p29_verify.sh")
        sys.exit(1)
print("p29 verifier present and current wrapper delegates")
