#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser(description="Generate an AiDENs release gate manifest from verifier logs.")
    ap.add_argument("--root", default=".")
    ap.add_argument("--run", default="P32")
    ap.add_argument("--log-dir", default=None)
    ap.add_argument("--out", default=None)
    args = ap.parse_args()

    root = Path(args.root).resolve()
    log_dir = (root / (args.log_dir or f"target/verify-release/{args.run}")).resolve()
    out = (root / (args.out or f"target/verify-release/{args.run}/RELEASE_GATE_MANIFEST.json")).resolve()

    suffixes = {".log", ".json", ".jsonl", ".txt"}
    entries = []
    if log_dir.exists():
        for path in sorted(p for p in log_dir.rglob("*") if p.is_file()):
            if path.resolve() == out:
                continue
            if path.suffix not in suffixes:
                continue
            rel = path.resolve().relative_to(root).as_posix()
            entries.append({
                "path": rel,
                "sha256": sha256_file(path),
                "bytes": path.stat().st_size,
            })

    data = {
        "artifact_kind": "aidens.release_gate_manifest.v1",
        "run": args.run,
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "log_dir": log_dir.relative_to(root).as_posix() if log_dir.is_relative_to(root) else str(log_dir),
        "entry_count": len(entries),
        "entries": entries,
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out} entries={len(entries)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
