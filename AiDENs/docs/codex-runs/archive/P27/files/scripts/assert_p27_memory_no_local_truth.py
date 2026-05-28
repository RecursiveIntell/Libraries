#!/usr/bin/env python3
"""Assert P27 memory grounding stays on canonical adapter routes."""

from __future__ import annotations

import re
import sys
from pathlib import Path


REQUIRED_OWNER_TOKENS = [
    "semantic-memory-forge",
    "forge-memory-bridge",
    "semantic-memory",
    "knowledge-runtime",
]

FORBIDDEN_PATTERNS = [
    re.compile(r"struct\s+(AiDENs|Aidens|Local).*Memory.*(Truth|Store|Db|Database)\b"),
    re.compile(r"enum\s+(AiDENs|Aidens|Local).*Memory.*(Truth|Store|Db|Database)\b"),
    re.compile(r"local_truth_store\s*:\s*true"),
    re.compile(r"memory_truth_owner\"\s*:\s*\"AiDENs", re.IGNORECASE),
]


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    source_paths = [
        root / "crates/aidens-memory-kit/src/lib.rs",
        root / "crates/aidens-runner/src/lib.rs",
        root / "crates/aidens-cli/src/lib.rs",
    ]
    missing = [str(path) for path in source_paths if not path.exists()]
    if missing:
        print(f"FAIL: missing memory seam source files: {missing}", file=sys.stderr)
        return 2

    combined = "\n".join(path.read_text(encoding="utf-8") for path in source_paths)
    for token in REQUIRED_OWNER_TOKENS:
        if token not in combined:
            print(f"FAIL: missing canonical memory owner token: {token}", file=sys.stderr)
            return 3
    for pattern in FORBIDDEN_PATTERNS:
        match = pattern.search(combined)
        if match:
            print(
                f"FAIL: forbidden local memory truth pattern matched: {match.group(0)}",
                file=sys.stderr,
            )
            return 4
    if "MemoryGroundingEvidenceV1" not in combined:
        print("FAIL: typed memory grounding evidence receipt is absent", file=sys.stderr)
        return 5
    if "local_truth_store: false" not in combined and '"local_truth_store": false' not in combined:
        print("FAIL: memory grounding evidence does not explicitly deny local truth storage", file=sys.stderr)
        return 6
    print("memory no-local-truth guard OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
