#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, sys
from pathlib import Path
REQUIRED={"commit_sha","tree_sha","cargo_lock_sha256","toolchain","platform","workspace_inventory_sha256","command_receipts"}
def main():
    p=argparse.ArgumentParser(); p.add_argument("--repo",required=True); p.add_argument("--strict",action="store_true"); a=p.parse_args()
    repo=Path(a.repo).expanduser().resolve(); ep=repo/"STATUS_EVIDENCE_MANIFEST.json"; rp=repo/"release/closeout_receipt_v1.json"; findings=[]
    if not ep.is_file(): findings.append("missing STATUS_EVIDENCE_MANIFEST.json")
    if not rp.is_file(): findings.append("missing release/closeout_receipt_v1.json")
    if findings: print(json.dumps({"findings":findings},indent=2)); return 1
    e=json.loads(ep.read_text()); r=json.loads(rp.read_text())
    if e.get("snapshot")!=r.get("snapshot"): findings.append(f"snapshot mismatch: {e.get('snapshot')!r} vs {r.get('snapshot')!r}")
    if e.get("captured_at")!=r.get("captured_at"): findings.append(f"captured_at mismatch: {e.get('captured_at')!r} vs {r.get('captured_at')!r}")
    er={x.get("command"):x.get("result") for x in e.get("proof_results",[]) if isinstance(x,dict)}
    if er!=r.get("gate_results",{}): findings.append("gate result mismatch")
    if a.strict:
        missing=sorted(REQUIRED-set(r.get("source_binding",{})))
        if missing: findings.append("missing source_binding fields: "+", ".join(missing))
    print(json.dumps({"schema_version":"libraries.evidence-consistency-report.v1","findings":findings},indent=2))
    return 1 if findings else 0
if __name__=="__main__": raise SystemExit(main())
