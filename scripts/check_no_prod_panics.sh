#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

if ! command -v jq >/dev/null 2>&1; then
  echo "supported-lane panic audit failed: jq is required" >&2
  exit 1
fi

allowlist_path="scripts/prod_panic_allowlist.json"
if [[ ! -f "$allowlist_path" ]]; then
  echo "supported-lane panic audit failed: missing allowlist file $allowlist_path" >&2
  exit 1
fi

python3 - <<'PY'
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

allowlist_path = Path("scripts/prod_panic_allowlist.json")
allowlist = set(json.loads(allowlist_path.read_text()))

supported = subprocess.check_output(
    [sys.executable, "scripts/print_supported_lane.py"], text=True
).splitlines()
if not supported:
    print("supported-lane panic audit failed: supported lane is empty", file=sys.stderr)
    sys.exit(1)

pattern = re.compile(
    r"\.unwrap\s*\(|\.expect\s*\(|\bpanic!\s*\(|\btodo!\s*\(|\bunimplemented!\s*\("
)

errors = 0
allowlisted_hits = 0


def strip_line_comment(line: str) -> str:
    # Good enough for the panic audit: preserve code before // comments. This
    # avoids counting documentation/comment examples without pretending to be a
    # full Rust lexer.
    return line.split("//", 1)[0]


def brace_delta(line: str) -> int:
    # Attribute/test-block skipping only needs coarse brace balancing. Strings
    # containing braces may overcount, but supported-lane test modules are
    # conventional enough for this release gate and failures remain visible.
    code = strip_line_comment(line)
    return code.count("{") - code.count("}")


def audit_file(path: Path) -> None:
    global errors, allowlisted_hits
    lines = path.read_text(errors="ignore").splitlines()
    skip_depth = 0
    pending_test_attr = False
    pending_cfg_test = False

    for idx, line in enumerate(lines, start=1):
        stripped = line.strip()

        if skip_depth > 0:
            skip_depth += brace_delta(line)
            if skip_depth <= 0:
                skip_depth = 0
            continue

        if stripped.startswith("#[test"):
            pending_test_attr = True
            continue

        if stripped.startswith("#[cfg") and "test" in stripped:
            pending_cfg_test = True
            continue

        if pending_test_attr or pending_cfg_test:
            delta = brace_delta(line)
            if "{" in strip_line_comment(line):
                skip_depth = delta
                if skip_depth <= 0:
                    skip_depth = 0
                pending_test_attr = False
                pending_cfg_test = False
                continue
            # Keep skipping attribute plumbing until the annotated item starts.
            continue

        code = strip_line_comment(line)
        if pattern.search(code):
            entry = f"{path.as_posix()}:{idx}"
            if entry in allowlist:
                allowlisted_hits += 1
            else:
                print(f"supported-lane panic audit failed: {entry}", file=sys.stderr)
                errors = 1


for crate in supported:
    src_dir = Path("living-memory/living-memory/src") if crate == "forge-engine" else Path(crate) / "src"
    if not src_dir.is_dir():
        print(f"supported-lane panic audit failed: missing source directory {src_dir}", file=sys.stderr)
        sys.exit(1)
    for target in sorted(src_dir.rglob("*.rs")):
        if target.name in {"tests.rs", "lib_tests.rs"} or target.name.endswith("_tests.rs"):
            continue
        audit_file(target)

if errors:
    sys.exit(1)
print(f"supported-lane panic audit passed ({allowlisted_hits} allowlisted compatibility shortcut(s))")
PY
