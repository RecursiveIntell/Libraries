#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, subprocess, sys
from pathlib import Path
def main():
    p=argparse.ArgumentParser(); p.add_argument("--repo",required=True); p.add_argument("--pack-dir")
    p.add_argument("--matrix",required=True); p.add_argument("--output-dir",required=True)
    p.add_argument("--stage",required=True,choices=["baseline","task","phase","final"]); p.add_argument("--continue-on-failure",action="store_true"); a=p.parse_args()
    repo=Path(a.repo).expanduser().resolve(); matrix_path=Path(a.matrix).expanduser().resolve()
    pack=Path(a.pack_dir).expanduser().resolve() if a.pack_dir else matrix_path.parent.parent
    out=Path(a.output_dir).expanduser().resolve(); out.mkdir(parents=True,exist_ok=True)
    matrix=json.loads(matrix_path.read_text()); results=[]; overall=0; runner=pack/"tools/run_with_receipt.py"
    for item in matrix.get("commands",[]):
        if a.stage not in item.get("stage",[]): continue
        cwd=(repo/item.get("cwd",".")).resolve(); argv=[x.replace("{pack_dir}",str(pack)) for x in item["argv"]]
        if not cwd.is_dir():
            status="blocked" if item.get("required",True) else "skipped"; results.append({"id":item["id"],"status":status,"reason":f"cwd missing: {cwd}","receipt":None})
            if item.get("required",True): overall=1
            if overall and not a.continue_on_failure: break
            continue
        cmd=[sys.executable,str(runner),"--repo",str(cwd),"--output-dir",str(out),"--name",item["id"],"--stage",a.stage,"--allow-dirty","--",*argv]
        c=subprocess.run(cmd,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        receipt=c.stdout.strip().splitlines()[-1] if c.stdout.strip() else None
        results.append({"id":item["id"],"status":"pass" if c.returncode==0 else "fail","required":item.get("required",True),"exit_code":c.returncode,"receipt":receipt,"runner_stderr":c.stderr})
        if c.returncode and item.get("required",True):
            overall=1
            if not a.continue_on_failure: break
    summary=out/f"validation-{a.stage}-summary.json"
    summary.write_text(json.dumps({"schema_version":"libraries.validation-summary.v1","stage":a.stage,"results":results,"verdict":"pass" if overall==0 else "fail"},indent=2)+"\n")
    print(summary); return overall
if __name__=="__main__": raise SystemExit(main())
