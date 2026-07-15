#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, os, tomllib
from pathlib import Path
SKIP={".git","target",".worktrees","node_modules"}
def main():
    p=argparse.ArgumentParser(); p.add_argument("--repo",required=True); a=p.parse_args(); repo=Path(a.repo).expanduser().resolve(); findings=[]
    for root,dirs,files in os.walk(repo):
        dirs[:]=[d for d in dirs if d not in SKIP]
        if "Cargo.toml" not in files: continue
        path=Path(root)/"Cargo.toml"
        try: d=tomllib.loads(path.read_text())
        except Exception: continue
        if "package" in d and d.get("lints",{}).get("workspace") is not True:
            findings.append({"path":str(path.relative_to(repo)),"package":d["package"].get("name"),"reason":"missing [lints] workspace = true"})
    print(json.dumps({"schema_version":"libraries.lint-inheritance-report.v1","findings":findings},indent=2))
    return 1 if findings else 0
if __name__=="__main__": raise SystemExit(main())
