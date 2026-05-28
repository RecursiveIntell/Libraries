#!/usr/bin/env python3
"""Hash super-pass audit logs into a machine-readable manifest."""

from pathlib import Path
import argparse
import hashlib
import json
from datetime import datetime, timezone


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--audit-dir", default="target/super-pass/audit")
    parser.add_argument("--out", default="target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json")
    args = parser.parse_args()

    audit_dir = Path(args.audit_dir)
    entries = []
    for path in sorted(audit_dir.glob("*.log")):
        if not path.is_file():
            continue
        entries.append(
            {
                "path": str(path),
                "bytes": path.stat().st_size,
                "sha256": sha256(path),
            }
        )

    output = {
        "artifact_kind": "super_pass_audit_log_hash_manifest",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "audit_dir": str(audit_dir),
        "entry_count": len(entries),
        "entries": entries,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(output, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"hashed {len(entries)} audit logs -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
