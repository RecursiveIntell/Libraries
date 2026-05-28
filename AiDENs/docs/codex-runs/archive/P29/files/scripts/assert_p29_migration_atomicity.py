#!/usr/bin/env python3
from pathlib import Path
import sys
roots = [Path("."), Path("../semantic-memory")]
files = []
for root in roots:
    if root.exists():
        files.extend(root.glob("**/*migration*.rs"))
        files.extend(root.glob("**/db.rs"))
        files.extend(root.glob("**/config.rs"))

text = "\n".join(p.read_text(encoding="utf-8", errors="ignore") for p in files)
required = [
    "failed to read PRAGMA user_version",
    "with_transaction(conn",
    "run_migration_v9(tx)",
    "run_migration_v16(tx)",
    "run_migration_v17(tx)",
]
missing = [marker for marker in required if marker not in text]
if missing:
    print("Missing migration atomicity evidence markers:", ", ".join(missing))
    sys.exit(1)
print("migration atomicity marker present")
