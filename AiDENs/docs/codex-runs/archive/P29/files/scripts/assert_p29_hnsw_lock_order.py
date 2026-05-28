#!/usr/bin/env python3
# Static smoke assertion; Codex should strengthen this with targeted tests.
from pathlib import Path
import sys

roots = [Path("."), Path("../semantic-memory")]
files = []
for root in roots:
    if root.exists():
        files.extend(root.glob("**/*hnsw*.rs"))
        files.extend(root.glob("**/lib.rs"))

text = "\n".join(p.read_text(encoding="utf-8", errors="ignore") for p in files)
required = [
    "deleted_snapshot",
    "saturating_add",
    "sync_pending_hnsw_sidecar",
    "with_read_conn",
    "with_write_conn",
]
missing = [marker for marker in required if marker not in text]
if missing:
    print("Missing HNSW lock-order/snapshot evidence markers:", ", ".join(missing))
    sys.exit(1)
print("HNSW lock-order/snapshot markers present")
