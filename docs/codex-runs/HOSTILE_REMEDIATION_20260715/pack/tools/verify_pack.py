#!/usr/bin/env python3
from __future__ import annotations
import argparse, hashlib, json, sys
from pathlib import Path
def sha(path):
    h=hashlib.sha256()
    with path.open("rb") as f:
        for c in iter(lambda:f.read(1024*1024),b""): h.update(c)
    return h.hexdigest()
def main():
    p=argparse.ArgumentParser(); p.add_argument("--pack",required=True); a=p.parse_args(); pack=Path(a.pack).expanduser().resolve()
    m=json.loads((pack/"MANIFEST.json").read_text()); errors=[]; expected=set()
    for x in m.get("files",[]):
        rel=x["path"]; expected.add(rel); path=pack/rel
        if not path.is_file(): errors.append("missing: "+rel)
        elif sha(path)!=x["sha256"]: errors.append("hash mismatch: "+rel)
    actual={str(p.relative_to(pack)) for p in pack.rglob("*") if p.is_file() and p.name not in {"MANIFEST.json","SHA256SUMS"}}
    if actual-expected: errors.append("unmanifested files: "+repr(sorted(actual-expected)))
    if errors: print("\n".join(errors),file=sys.stderr); return 1
    print(f"pack verified: {len(expected)} files"); return 0
if __name__=="__main__": raise SystemExit(main())
