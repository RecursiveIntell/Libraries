#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, os, subprocess, sys, tomllib
from pathlib import Path
from common import atomic_write_json, sha256_file, source_state, utc_now
SKIP={".git","target",".worktrees","node_modules",".venv","venv"}

def cargo_tomls(repo: Path):
    out=[]
    for root,dirs,files in os.walk(repo):
        dirs[:]=[d for d in dirs if d not in SKIP]
        if "Cargo.toml" in files: out.append(Path(root)/"Cargo.toml")
    return sorted(out)

def metadata(root: Path):
    c=subprocess.run(["cargo","metadata","--format-version","1","--no-deps"],cwd=root,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
    r={"exit_code":c.returncode,"stderr":c.stderr}
    if c.returncode==0:
        try:
            d=json.loads(c.stdout); r["workspace_root"]=d.get("workspace_root"); r["workspace_members"]=d.get("workspace_members",[])
            r["packages"]=[{"name":x.get("name"),"version":x.get("version"),"manifest_path":x.get("manifest_path"),"id":x.get("id")} for x in d.get("packages",[])]
        except json.JSONDecodeError as e: r["parse_error"]=str(e)
    return r

def main():
    p=argparse.ArgumentParser(); p.add_argument("--repo",required=True); p.add_argument("--output",required=True); a=p.parse_args()
    repo=Path(a.repo).expanduser().resolve(); workspaces=[]; packages=[]
    for m in cargo_tomls(repo):
        try: d=tomllib.loads(m.read_text())
        except Exception as e: packages.append({"path":str(m.relative_to(repo)),"parse_error":str(e)}); continue
        rel=m.relative_to(repo); pkg=d.get("package")
        if isinstance(pkg,dict):
            packages.append({"path":str(rel),"name":pkg.get("name"),"version":pkg.get("version"),"rust_version":pkg.get("rust-version"),
                             "lints_workspace":d.get("lints",{}).get("workspace") is True,"sha256":sha256_file(m)})
        if "workspace" in d:
            root=m.parent; ws=d.get("workspace",{}); members=ws.get("members",[])
            missing=[x for x in members if not (root/x/"Cargo.toml").is_file()]
            workspaces.append({"path":"." if root==repo else str(root.relative_to(repo)),"manifest":str(rel),
              "members_declared":members,"default_members":ws.get("default-members",[]),"missing_declared_members":missing,
              "metadata":metadata(root),"manifest_sha256":sha256_file(m),"lock_sha256":sha256_file(root/"Cargo.lock")})
    result={"schema_version":"libraries.workspace-inventory.v1","captured_at":utc_now(),"source":source_state(repo),"workspaces":workspaces,"packages":packages}
    atomic_write_json(Path(a.output).expanduser().resolve(),result)
    bad=[w for w in workspaces if w["missing_declared_members"] or w["metadata"].get("exit_code")!=0]
    if bad: print(json.dumps(bad,indent=2),file=sys.stderr); return 1
    return 0
if __name__=="__main__": raise SystemExit(main())
