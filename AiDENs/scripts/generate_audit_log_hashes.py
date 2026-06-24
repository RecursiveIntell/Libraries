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


def evidence_roots(root: Path) -> list[Path]:
    return [
        root / "target" / "verify-current",
        root / "target" / "verify-release",
        root / "docs" / "codex-runs",
    ]


def main() -> int:
    ap = argparse.ArgumentParser(description="Generate PHASE_15_AUDIT_LOG_HASHES.json from real AiDENs evidence logs.")
    ap.add_argument("--root", default=".")
    ap.add_argument("--run", default="P32")
    ap.add_argument("--out", default="target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    out = (root / args.out).resolve()
    suffixes = {".log", ".json", ".jsonl", ".txt"}
    entries = []

    for base in evidence_roots(root):
        if not base.exists():
            continue
        for path in sorted(p for p in base.rglob("*") if p.is_file()):
            if path.resolve() == out:
                continue
            if path.suffix not in suffixes:
                continue
            # Avoid source-like JSON fixtures under archive source packages; this manifest is for evidence logs/receipts.
            rel = path.resolve().relative_to(root).as_posix()
            if "/source-packages/archive/" in rel:
                continue
            entries.append({
                "path": rel,
                "sha256": sha256_file(path),
                "bytes": path.stat().st_size,
            })

    data = {
        "artifact_kind": "aidens.audit_log_hash_manifest.v1",
        "run": args.run,
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "entry_count": len(entries),
        "entries": entries,
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out} entries={len(entries)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
