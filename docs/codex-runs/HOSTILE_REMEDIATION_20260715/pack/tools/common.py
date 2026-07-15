from __future__ import annotations
import hashlib, json, os, platform, subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Sequence

def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00","Z")

def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def sha256_file(path: Path) -> str | None:
    return sha256_bytes(path.read_bytes()) if path.is_file() else None

def run_capture(argv: Sequence[str], cwd: Path) -> tuple[int,str,str]:
    try:
        c=subprocess.run(list(argv),cwd=cwd,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
        return c.returncode,c.stdout,c.stderr
    except FileNotFoundError as e:
        return 127,"",str(e)

def git_value(repo: Path,*args: str) -> str | None:
    code,out,_=run_capture(["git",*args],repo)
    return out.strip() if code==0 else None

def source_state(repo: Path) -> dict[str,Any]:
    status=git_value(repo,"status","--porcelain=v1")
    return {
        "branch":git_value(repo,"branch","--show-current"),
        "commit":git_value(repo,"rev-parse","HEAD"),
        "tree":git_value(repo,"rev-parse","HEAD^{tree}"),
        "dirty":bool(status) if status is not None else None,
        "status_sha256":sha256_bytes((status or "").encode()) if status is not None else None,
        "cargo_lock_sha256":sha256_file(repo/"Cargo.lock"),
    }

def environment_state(repo: Path) -> dict[str,Any]:
    _,rustc,re=run_capture(["rustc","-Vv"],repo)
    _,cargo,ce=run_capture(["cargo","-V"],repo)
    return {"platform":platform.platform(),"machine":platform.machine(),"python":platform.python_version(),
            "rustc":rustc.strip() or re.strip(),"cargo":cargo.strip() or ce.strip(),"pid":os.getpid()}

def atomic_write_json(path: Path,data: Any) -> None:
    path.parent.mkdir(parents=True,exist_ok=True)
    tmp=path.with_suffix(path.suffix+".tmp")
    tmp.write_text(json.dumps(data,indent=2,sort_keys=False)+"\n",encoding="utf-8")
    tmp.replace(path)
