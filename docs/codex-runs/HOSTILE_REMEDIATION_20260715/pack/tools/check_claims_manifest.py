#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, re
from pathlib import Path
READMES=["fib-quant/README.md","turbo-quant/README.md","poly-kv/README.md"]
RX=re.compile(r"\b(?:\d+(?:\.\d+)?\s*[×x%]|Recall@\d+|nDCG@\d+|rank drift|ms\b|KB\b|MB\b)",re.I)
def main():
    p=argparse.ArgumentParser(); p.add_argument("--repo",required=True); a=p.parse_args(); repo=Path(a.repo).expanduser().resolve()
    mp=repo/"docs/claims_manifest.json"; findings=[]
    if not mp.is_file(): findings.append({"path":"docs/claims_manifest.json","reason":"missing claims manifest"}); manifest={"claims":[]}
    else: manifest=json.loads(mp.read_text())
    declared={(x.get("path"),x.get("claim")) for x in manifest.get("claims",[])}
    for rel in READMES:
        path=repo/rel
        if not path.is_file(): continue
        for n,line in enumerate(path.read_text(errors="replace").splitlines(),1):
            if RX.search(line) and (rel,line.strip()) not in declared:
                findings.append({"path":rel,"line":n,"claim":line.strip(),"reason":"quantitative claim absent verbatim from manifest"})
    print(json.dumps({"schema_version":"libraries.claims-report.v1","findings":findings},indent=2))
    return 1 if findings else 0
if __name__=="__main__": raise SystemExit(main())
