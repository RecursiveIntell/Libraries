#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, os, re
from pathlib import Path
SKIP={".git","target",".worktrees","tests","benches","examples","fixtures"}
RULES=[
("decode_identity_passthrough",re.compile(r"Ok\s*\(\s*data\.to_vec\s*\(\s*\)\s*\)")),
("decode_placeholder_todo",re.compile(r"TODO.*(?:decode|decompress)",re.I)),
("q8_q4_silent_uncompressed",re.compile(r"CodecProfile::Q[48].*CodecId::Uncompressed")),
("ignored_dispatch",re.compile(r"\b_dispatch\b")),
("placeholder_comment",re.compile(r"\bplaceholder\b.*(?:decode|codec)",re.I))]
def main():
    p=argparse.ArgumentParser(); p.add_argument("--repo",required=True); p.add_argument("--json-output"); a=p.parse_args()
    repo=Path(a.repo).expanduser().resolve(); findings=[]
    for root,dirs,files in os.walk(repo):
        dirs[:]=[d for d in dirs if d not in SKIP]
        for f in files:
            if not f.endswith(".rs"): continue
            path=Path(root)/f; rel=str(path.relative_to(repo))
            for n,line in enumerate(path.read_text(errors="replace").splitlines(),1):
                for rule,rx in RULES:
                    if rx.search(line): findings.append({"rule":rule,"path":rel,"line":n,"text":line.strip()})
    report={"schema_version":"libraries.placeholder-codec-report.v1","findings":findings}; text=json.dumps(report,indent=2); print(text)
    if a.json_output: Path(a.json_output).write_text(text+"\n")
    return 1 if findings else 0
if __name__=="__main__": raise SystemExit(main())
