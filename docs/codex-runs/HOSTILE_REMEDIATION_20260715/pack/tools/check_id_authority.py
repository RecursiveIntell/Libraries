#!/usr/bin/env python3
from __future__ import annotations
import argparse, fnmatch, json, os, re, tomllib
from pathlib import Path
SKIP={".git","target",".worktrees","node_modules",".venv","venv"}
TYPE_RE=re.compile(r"^\s*pub\s+(?:struct|enum|type)\s+([A-Za-z_][A-Za-z0-9_]*Id)\b")
FIELD_RE=re.compile(r"^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*_id)\s*:\s*(?:Option\s*<\s*)?String\b")
DIRECT_RE=re.compile(r"\b(?:uuid::Uuid|ulid::Ulid|Uuid::new_v[0-9]|Ulid::new)\b")

def allowed(rel,kind,line,exceptions):
    for e in exceptions:
        if fnmatch.fnmatch(rel,e.get("path","")) and e.get("pattern") in (kind,line):
            if {"owner","reason","issue","removal_condition","expires_on"}.issubset(e): return True
    return False

def main():
    p=argparse.ArgumentParser(); p.add_argument("--repo",required=True); p.add_argument("--allowlist",required=True); p.add_argument("--json-output"); a=p.parse_args()
    repo=Path(a.repo).expanduser().resolve(); exceptions=json.loads(Path(a.allowlist).read_text()).get("exceptions",[]); findings=[]
    for m in repo.rglob("Cargo.toml"):
        if any(x in SKIP for x in m.parts) or m.parent.name=="stack-ids": continue
        try: d=tomllib.loads(m.read_text())
        except Exception: continue
        for table in ("dependencies","build-dependencies"):
            for dep in ("uuid","ulid"):
                if dep in d.get(table,{}): findings.append({"kind":"direct_dependency","path":str(m.relative_to(repo)),"line":f"{table}.{dep}"})
    for root,dirs,files in os.walk(repo):
        dirs[:]=[d for d in dirs if d not in SKIP]
        rp=Path(root); relroot=rp.relative_to(repo)
        if relroot.parts and relroot.parts[0]=="stack-ids": continue
        for f in files:
            if not f.endswith(".rs"): continue
            path=rp/f
            if any(x in {"tests","benches","examples","fixtures"} for x in path.parts): continue
            rel=str(path.relative_to(repo))
            for n,line in enumerate(path.read_text(errors="replace").splitlines(),1):
                if TYPE_RE.match(line): findings.append({"kind":"public_local_id_type","path":rel,"line":n,"text":line.strip()})
                if FIELD_RE.match(line): findings.append({"kind":"public_raw_id_string","path":rel,"line":n,"text":line.strip()})
                if DIRECT_RE.search(line): findings.append({"kind":"direct_id_generation","path":rel,"line":n,"text":line.strip()})
    dis=[f for f in findings if not allowed(f["path"],f["kind"],str(f.get("text") or f.get("line")),exceptions)]
    report={"schema_version":"libraries.id-authority-report.v1","findings":findings,"disallowed":dis}
    text=json.dumps(report,indent=2); print(text)
    if a.json_output: Path(a.json_output).write_text(text+"\n")
    return 1 if dis else 0
if __name__=="__main__": raise SystemExit(main())
