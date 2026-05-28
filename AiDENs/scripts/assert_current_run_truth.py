#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
LEDGER = ROOT / "docs" / "codex-runs" / "CURRENT_RUN.json"
MIRROR = ROOT / "docs" / "codex-runs" / "CURRENT_RUN.md"
PROTECTED = ["README.md", "STATUS.md", "SOURCE_BASIS.md", "SUPPORT_PROFILE.md"]
RUN_RE = re.compile(r"\bP\d+[A-Z]?\b")


def fail(msg: str) -> int:
    print(f"FAIL: {msg}", file=sys.stderr)
    return 2


def load() -> dict:
    if not LEDGER.exists():
        raise FileNotFoundError(f"missing {LEDGER.relative_to(ROOT)}")
    return json.loads(LEDGER.read_text(encoding="utf-8"))


def main() -> int:
    try:
        data = load()
    except Exception as e:
        return fail(str(e))
    active = str(data.get("active_run", "")).upper()
    last = str(data.get("last_certified_run", "")).upper()
    parent = str(data.get("parent_run", "")).upper()
    allowed = {active, last, parent}
    errors: list[str] = []

    for path in [MIRROR] + [ROOT / p for p in PROTECTED]:
        if not path.exists():
            errors.append(f"missing protected run-truth file: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        if "CURRENT_RUN.json" not in text:
            errors.append(f"{path.relative_to(ROOT)} does not cite CURRENT_RUN.json")
        if active not in text:
            errors.append(f"{path.relative_to(ROOT)} does not mention active_run {active}")
        if last not in text:
            errors.append(f"{path.relative_to(ROOT)} does not mention last_certified_run {last}")
        # Protected docs should be concise mirrors, not historical run catalogs.
        found = {m.group(0).upper() for m in RUN_RE.finditer(text)}
        unexpected = sorted(found - allowed)
        if unexpected:
            errors.append(f"{path.relative_to(ROOT)} mentions unexpected run IDs {unexpected}; move history to archive/deferred docs")

    if errors:
        for e in errors:
            print(f"FAIL: {e}", file=sys.stderr)
        return 2
    print(f"PASS: current-run truth matches ledger active={active} last_certified={last}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
