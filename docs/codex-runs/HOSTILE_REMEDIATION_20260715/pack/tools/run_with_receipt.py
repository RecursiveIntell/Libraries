#!/usr/bin/env python3
from __future__ import annotations
import argparse, os, subprocess, sys, time, uuid
from pathlib import Path
from common import atomic_write_json, environment_state, sha256_bytes, source_state, utc_now

def main() -> int:
    p=argparse.ArgumentParser(description="Run a command and write a source-bound receipt.")
    p.add_argument("--repo",required=True); p.add_argument("--output-dir",required=True)
    p.add_argument("--name",required=True); p.add_argument("--stage",default="task")
    p.add_argument("--issue",action="append",default=[]); p.add_argument("--allow-dirty",action="store_true")
    p.add_argument("command",nargs=argparse.REMAINDER); a=p.parse_args()
    if a.command and a.command[0]=="--": a.command=a.command[1:]
    if not a.command: p.error("command required after --")
    repo=Path(a.repo).expanduser().resolve(); out=Path(a.output_dir).expanduser().resolve(); out.mkdir(parents=True,exist_ok=True)
    before=source_state(repo)
    if before.get("dirty") and not a.allow_dirty:
        print("refusing dirty tree; pass --allow-dirty only for intentional recorded state",file=sys.stderr); return 125
    rid=f"{a.name}-{uuid.uuid4().hex[:12]}"
    stdout_path=out/f"{rid}.stdout.log"; stderr_path=out/f"{rid}.stderr.log"; receipt_path=out/f"{rid}.json"
    started=utc_now(); t=time.monotonic()
    c=subprocess.run(a.command,cwd=repo,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False,env=os.environ.copy())
    duration=int((time.monotonic()-t)*1000); finished=utc_now()
    stdout_path.write_bytes(c.stdout); stderr_path.write_bytes(c.stderr)
    after=source_state(repo)
    atomic_write_json(receipt_path,{
      "schema_version":"libraries.command-receipt.v1","receipt_id":rid,"name":a.name,"stage":a.stage,
      "issue_ids":a.issue,"argv":a.command,"cwd":str(repo),"started_at":started,"finished_at":finished,
      "duration_ms":duration,"exit_code":c.returncode,"source":before,"source_after":after,
      "source_changed_during_command":before!=after,"environment":environment_state(repo),
      "stdout":{"path":str(stdout_path),"sha256":sha256_bytes(c.stdout),"bytes":len(c.stdout)},
      "stderr":{"path":str(stderr_path),"sha256":sha256_bytes(c.stderr),"bytes":len(c.stderr)}})
    print(receipt_path); return c.returncode
if __name__=="__main__": raise SystemExit(main())
