#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, re, sys
from pathlib import Path
def validate(s,v,path="$"):
    e=[]
    if "const" in s and v!=s["const"]: e.append(f"{path}: expected {s['const']!r}, got {v!r}")
    if "enum" in s and v not in s["enum"]: e.append(f"{path}: {v!r} not in enum")
    t=s.get("type")
    if t:
        allowed=t if isinstance(t,list) else [t]; ok=False
        for x in allowed:
            ok|=(x=="object" and isinstance(v,dict)) or (x=="array" and isinstance(v,list)) or (x=="string" and isinstance(v,str)) or (x=="integer" and isinstance(v,int) and not isinstance(v,bool)) or (x=="boolean" and isinstance(v,bool)) or (x=="null" and v is None)
        if not ok: return [f"{path}: expected {allowed}, got {type(v).__name__}"]
    if isinstance(v,str) and "minLength" in s and len(v)<s["minLength"]: e.append(f"{path}: too short")
    if isinstance(v,str) and "pattern" in s and not re.search(s["pattern"],v): e.append(f"{path}: pattern mismatch")
    if isinstance(v,dict):
        for k in s.get("required",[]):
            if k not in v: e.append(f"{path}: missing {k!r}")
        props=s.get("properties",{})
        for k,x in v.items():
            if k in props: e+=validate(props[k],x,f"{path}.{k}")
            elif isinstance(s.get("additionalProperties"),dict): e+=validate(s["additionalProperties"],x,f"{path}.{k}")
    if isinstance(v,list) and isinstance(s.get("items"),dict):
        for i,x in enumerate(v): e+=validate(s["items"],x,f"{path}[{i}]")
    return e
def main():
    p=argparse.ArgumentParser(); p.add_argument("--schema",required=True); p.add_argument("--document",required=True); a=p.parse_args()
    e=validate(json.loads(Path(a.schema).read_text()),json.loads(Path(a.document).read_text()))
    if e: print("\n".join(e),file=sys.stderr); return 1
    print("valid"); return 0
if __name__=="__main__": raise SystemExit(main())
